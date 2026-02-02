import { GearApi, HexString, UserMessageSent } from '@gear-js/api';
import { u8aToHex } from '@polkadot/util';
import type {
  TypeDecl,
  IIdlDoc,
  IServiceExpo,
  IServiceIdent,
  IServiceUnit,
  IFuncParam,
  IServiceEvent,
} from 'sails-js-types-v2';
import { SailsMessageHeader, InterfaceId } from 'sails-js-parser-v2';

import { TransactionBuilder } from './transaction-builder-v2.js';
import { TypeResolver } from './type-resolver-v2.js';
import { ZERO_ADDRESS } from './consts.js';
import { QueryBuilder } from './query-builder-v2.js';

interface ISailsService {
  readonly functions: Record<string, SailsServiceFunc>;
  readonly queries: Record<string, SailsServiceQuery>;
  readonly events: Record<string, ISailsServiceEvent>;
  readonly extends: Record<string, SailsService>;
  readonly routeIdx: number;
}

interface ISailsFuncArg {
  /** ### Argument name */
  name: string;
  /** ### Argument type */
  type: any;
  /** ### Argument type definition */
  typeDef: TypeDecl;
}

interface ISailsServiceFuncParams {
  /** ### List of argument names and types  */
  readonly args: ISailsFuncArg[];
  /** ### Function return type */
  readonly returnType: any;
  /** ### Function return type definition */
  readonly returnTypeDef: TypeDecl;
  /** ### Encode payload to hex string */
  readonly encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  readonly decodePayload: <T = any>(bytes: HexString) => T;
  /** ### Decode function result */
  readonly decodeResult: <T = any>(result: HexString) => T;
  /** ### Docs from the IDL file */
  readonly docs?: string;
}

type SailsServiceQuery = ISailsServiceFuncParams & (<T = any>(...args: unknown[]) => QueryBuilder<T>);

type SailsServiceFunc = ISailsServiceFuncParams & (<T = any>(...args: unknown[]) => TransactionBuilder<T>);

interface ISailsServiceEvent {
  /** ### Event type */
  readonly type: any;
  /** ###  */
  readonly typeDef: IServiceEvent;
  /** ### Check if event is of this type */
  readonly is: (event: UserMessageSent) => boolean;
  /** ### Decode event payload */
  readonly decode: (payload: HexString) => any;
  /** ### Subscribe to event
   * @returns Promise with unsubscribe function
   */
  readonly subscribe: <T = any>(cb: (event: T) => void | Promise<void>) => Promise<() => void>;
  /** ### Docs from the IDL file */
  readonly docs?: string;
}

interface ISailsCtorFuncParams {
  /** ### List of argument names and types  */
  readonly args: ISailsFuncArg[];
  /** ### Encode payload to hex string */
  readonly encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  readonly decodePayload: <T = any>(bytes: HexString) => T;
  /** ### Create transaction builder from code */
  readonly fromCode: (code: Uint8Array | Buffer, ...args: unknown[]) => TransactionBuilder<any>;
  /** ### Create transaction builder from code id */
  readonly fromCodeId: (codeId: HexString, ...args: unknown[]) => TransactionBuilder<any>;
  /** ### Docs from the IDL file */
  readonly docs?: string;
}

const _getParamsForTxBuilder = (params: ISailsFuncArg[]) => {
  if (params.length === 0) return null;
  if (params.length === 1) return params[0].type;
  return `(${params.map((p) => p.type).join(', ')})`;
}

const _getArgsForTxBuilder = (args: any[], params: ISailsFuncArg[]) => {
  if (params.length === 0) return null;
  if (params.length === 1) return args[0];
  return args.slice(0, params.length);
}

export class SailsProgram {
  private _doc: IIdlDoc;
  private _typeResolver: TypeResolver;
  private _api?: GearApi;
  private _programId?: HexString;
  private _services: Map<bigint, IServiceUnit> = new Map();
  private _resolveServiceUnit: (ident: IServiceIdent) => IServiceUnit | undefined;


  /**
   * ### Crate program from parser IDL document
   * @param doc parser and normalized IDL document
   */
  constructor(doc: IIdlDoc) {
    this._doc = doc;
    if (this._doc.program) {
      this._typeResolver = new TypeResolver(this._doc.program.types);
    }

    // this.generateScaleCodeTypes();
    this._services = this._initServices();
    this._resolveServiceUnit = (ident: IServiceIdent) => {
      if (!ident.interface_id) {
        throw new Error(`Service "${ident.name}" is missing interface_id in IDL`);
      }
      const interfaceId = InterfaceId.from(ident.interface_id).asU64();
      return this._services.get(interfaceId);
    };
  }

  /** ### Set api to use for transactions */
  setApi(api: GearApi) {
    this._api = api;
    return this;
  }

  /** ### Set program id to interact with */
  setProgramId(programId: HexString) {
    this._programId = programId;
    return this;
  }

  /** ### Get program id */
  get programId() {
    return this._programId;
  }

  private _initServices(): Map<bigint, IServiceUnit> {
    const services = new Map<bigint, IServiceUnit>();
    for (const service of this._doc.services ?? []) {
      if (!service.interface_id) {
        throw new Error(`Service "${service.name}" is missing interface_id in IDL`);
      }

      const interfaceId = InterfaceId.from(service.interface_id).asU64();
      services.set(interfaceId, service);
    }

    return services;
  }

  /** #### Registry with registered types from the parsed IDL */
  get registry() {
    if (!this._doc.program) {
      throw new Error('Program not exists');
    }

    return this._typeResolver.registry;
  }

  /** #### TypeResolver with registered types from the parsed IDL */
  get typeResolver() {
    if (!this._doc.program) {
      throw new Error('Program not exists');
    }

    return this._typeResolver;
  }

  /** #### Services with functions and events from the parsed IDL */
  get services(): Record<string, SailsService> {
    const services: Record<string, SailsService> = {};
    const program = this._doc.program;

    if (!program?.services?.length || !this._resolveServiceUnit) {
      return services;
    }

    for (const expo of program.services as IServiceExpo[]) {
      const serviceUnit = this._resolveServiceUnit(expo);
      if (!serviceUnit) {
        throw new Error(`Service definition for "${expo.name}" not found in IDL`);
      }
      services[expo.name] = new SailsService(serviceUnit, this._api, this._programId, expo.route_idx, this._resolveServiceUnit);
    }
    return services;
  }

  /** #### Constructor functions with arguments from the parsed IDL */
  get ctors(): Record<string, ISailsCtorFuncParams> | null {
    if (!this._doc.program) {
      return null;
    }

    const program = this._doc.program;
    const funcs: Record<string, ISailsCtorFuncParams> = {};

    for (const [entry_id, func] of program.ctors.entries()) {
      const header = SailsMessageHeader.v1(InterfaceId.zero(), entry_id, 0);
      const params = func.params.map((p: IFuncParam) => ({ name: p.name, type: this._typeResolver.getTypeDeclString(p.type), typeDef: p.type }));
      funcs[func.name] = {
        args: params,
        encodePayload: (...args): HexString => {
          if (args.length !== params.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          if (params.length === 0) {
            return u8aToHex(header.toBytes());
          }

          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, [
            header.toBytes(),
            ...args,
          ]);

          return payload.toHex();
        },
        decodePayload: <T = any>(bytes: Uint8Array | string) => {
          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, bytes);
          const result = {} as Record<string, any>;
          for (const [i, param] of params.entries()) {
            result[param.name] = payload[i + 1].toJSON();
          }
          return result as T;
        },
        fromCode: (code: Uint8Array | Buffer, ...args: unknown[]) => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }

          const builder = new TransactionBuilder(
            this._api,
            this.registry,
            'upload_program',
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            'String',
            code,
          );

          this._programId = builder.programId;
          return builder;
        },
        fromCodeId: (codeId: HexString, ...args: unknown[]) => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }

          const builder = new TransactionBuilder(
            this._api,
            this.registry,
            'create_program',
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            'String',
            codeId,
          );

          this._programId = builder.programId;
          return builder;
        },
        docs: func.docs ? func.docs.join('\n') : undefined,
      };
    }

    return funcs;
  }

  /** #### Parsed IDL */
  get program() {
    if (!this._doc) {
      throw new Error('IDL is not parsed');
    }

    return this._doc.program;
  }
}

export class SailsService implements ISailsService {
  functions: Record<string, SailsServiceFunc>;
  queries: Record<string, SailsServiceQuery>;
  events: Record<string, ISailsServiceEvent>;
  routeIdx: number;

  private _service: IServiceUnit;
  private _typeResolver: TypeResolver;
  private _api?: GearApi;
  private _programId?: HexString;
  private _resolveServiceUnit?: (ident: IServiceIdent) => IServiceUnit | undefined;

  constructor(
    service: IServiceUnit,
    api?: GearApi,
    programId?: HexString,
    routeIdx = 0,
    resolveServiceUnit?: (ident: IServiceIdent) => IServiceUnit | undefined,
  ) {
    this._service = service;
    this._api = api;
    this._programId = programId;
    this.routeIdx = routeIdx;
    this._resolveServiceUnit = resolveServiceUnit;
    this._typeResolver = new TypeResolver(service.types);

    this.events = this._getEvents(service);
    const { funcs, queries } = this._getFunctions(service);
    this.functions = funcs;
    this.queries = queries;
  }

  withRouteIdx(routeIdx: number): SailsService {
    if (routeIdx === this.routeIdx) {
      return this;
    }

    return new SailsService(this._service, this._api, this._programId, routeIdx, this._resolveServiceUnit);
  }

  /** #### Registry with registered types from the ServiceUnit */
  get registry() {
    if (!this._service) {
      throw new Error('Service not set');
    }
    return this._typeResolver.registry;
  }

  private _getFunctions(service: IServiceUnit): {
    funcs: Record<string, SailsServiceFunc>;
    queries: Record<string, SailsServiceQuery>;
  } {
    const funcs: Record<string, SailsServiceFunc> = {};
    const queries: Record<string, SailsServiceQuery> = {};

    for (const [entry_id, func] of service.funcs.entries()) {
      const header = SailsMessageHeader.v1(InterfaceId.from(service.interface_id), entry_id, this.routeIdx);
      const params: ISailsFuncArg[] = func.params.map((p: IFuncParam) => ({
        name: p.name,
        type: this._typeResolver.getTypeDeclString(p.type),
        typeDef: p.type,
      }));
      const returnType = this._typeResolver.getTypeDeclString(func.output);
      if (func.kind == "query") {
        queries[func.name] = (<T = any>(...args: unknown[]): QueryBuilder<T> => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }
          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }

          return new QueryBuilder<T>(
            this._api,
            this.registry,
            this._programId,
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            returnType,
          );
        }) as SailsServiceQuery;
      } else {
        funcs[func.name] = (<T = any>(...args: unknown[]): TransactionBuilder<T> => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }
          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }
          return new TransactionBuilder(
            this._api,
            this.registry,
            'send_message',
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            returnType,
            this._programId,
          );
        }) as SailsServiceFunc;
      }

      Object.assign(func.kind == "query" ? queries[func.name] : funcs[func.name], {
        args: params,
        returnType,
        returnTypeDef: func.output,
        docs: func.docs ? func.docs.join('\n') : undefined,
        encodePayload: (...args: unknown[]): HexString => {
          if (args.length !== params.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          if (params.length === 0) {
            return u8aToHex(header.toBytes());
          }

          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, [
            header.toBytes(),
            ...args,
          ]);

          return payload.toHex();
        },
        decodePayload: <T = any>(bytes: HexString) => {
          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, bytes);
          const result = {} as Record<string, any>;
          for (const [i, param] of params.entries()) {
            result[param.name] = payload[i + 1].toJSON();
          }
          return result as T;
        },
        decodeResult: <T = any>(result: HexString) => {
          const payload = this.registry.createType(`([u8; 16], ${returnType})`, result);
          return payload[1].toJSON() as T;
        },
      });
    }

    return { funcs, queries };
  }

  private _getEvents(service: IServiceUnit): Record<string, ISailsServiceEvent> {
    const events: Record<string, ISailsServiceEvent> = {};
    const interface_id_u64: bigint = InterfaceId.from(service.interface_id).asU64();

    for (const [entry_id, event] of service.events.entries()) {
      const t = event.fields?.length ? this._typeResolver.getStructDef(event.fields) : 'Null';
      const typeStr = event.fields?.length ? this._typeResolver.getStructDef(event.fields, {}, true) : 'Null';
      events[event.name] = {
        type: t,
        typeDef: event,
        docs: event.docs ? event.docs.join('\n') : undefined,
        is: ({ data: { message } }: UserMessageSent) => {
          if (!message.destination.eq(ZERO_ADDRESS)) {
            return false;
          }

          const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
          if (ok && header.interface_id.asU64() === interface_id_u64 && header.entry_id === entry_id) {
            return true;
          }
          return false;
        },
        decode: (payload: HexString) => {
          const data = this.registry.createType(`([u8; 16], ${typeStr})`, payload);
          return data[1].toJSON();
        },
        subscribe: <T = any>(cb: (eventData: T) => void | Promise<void>): Promise<() => void> => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }

          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }

          return this._api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
            if (!message.source.eq(this._programId)) return;
            if (!message.destination.eq(ZERO_ADDRESS)) return;

            const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
            if (
              ok &&
              header.interface_id.asU64() === interface_id_u64 &&
              header.entry_id === entry_id
            ) {
              cb(this.registry.createType(`([u8; 16], ${typeStr})`, message.payload)[1].toJSON() as T);
              // const payload: Uint8Array = message.payload.slice(header.hlen);
              // cb(this._registry.createType(`${typeStr}`, payload).toJSON() as T);
            }
          });
        },
      };
    }

    return events;
  }

  get extends(): Record<string, SailsService> {
    const extended: Record<string, SailsService> = {};

    if (!this._service?.extends?.length || !this._resolveServiceUnit) {
      return extended;
    }

    for (const ident of this._service.extends as IServiceIdent[]) {
      const serviceUnit = this._resolveServiceUnit(ident);
      if (!serviceUnit) {
        throw new Error(`Service definition for "${ident.name}" not found in IDL`);
      }

      extended[ident.name] = new SailsService(
        serviceUnit,
        this._api,
        this._programId,
        this.routeIdx,
        this._resolveServiceUnit,
      );
    }

    return extended;
  }
}
