import { GearApi, HexString, UserMessageSent, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types/create';
import { u8aToHex } from '@polkadot/util';
import { ISailsIdlParser, ISailsProgram, ISailsService, ISailsTypeDef } from 'sails-js-types';

import { getFnNamePrefix, getServiceNamePrefix } from './prefix.js';
import { TransactionBuilder } from './transaction-builder.js';
import { getScaleCodecDef } from 'sails-js-util';
import { ZERO_ADDRESS } from './consts.js';

interface SailsService {
  readonly functions: Record<string, SailsServiceFunc>;
  readonly queries: Record<string, SailsServiceQuery>;
  readonly events: Record<string, SailsServiceEvent>;
}

interface ISailsFuncArg {
  /** ### Argument name */
  name: string;
  /** ### Argument type */
  type: any;
  /** ### Argument type definition */
  typeDef: ISailsTypeDef;
}

interface ISailsServiceFuncParams {
  /** ### List of argument names and types  */
  readonly args: ISailsFuncArg[];
  /** ### Function return type */
  readonly returnType: any;
  /** ### Function return type definition */
  readonly returnTypeDef: ISailsTypeDef;
  /** ### Encode payload to hex string */
  readonly encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  readonly decodePayload: <T = any>(bytes: HexString) => T;
  /** ### Decode function result */
  readonly decodeResult: <T = any>(result: HexString) => T;
}

type SailsServiceQuery = ISailsServiceFuncParams &
  (<T>(origin: string, value?: bigint, atBlock?: HexString, ...args: unknown[]) => Promise<T>);

type SailsServiceFunc = ISailsServiceFuncParams & (<T>(...args: unknown[]) => TransactionBuilder<T>);

interface SailsServiceEvent {
  /** ### Event type */
  readonly type: any;
  /** ###  */
  readonly typeDef: ISailsTypeDef;
  /** ### Check if event is of this type */
  readonly is: (event: UserMessageSent) => boolean;
  /** ### Decode event payload */
  readonly decode: (payload: HexString) => any;
  /** ### Subscribe to event
   * @returns Promise with unsubscribe function
   */
  readonly subscribe: <T>(cb: (event: T) => void | Promise<void>) => Promise<() => void>;
}

interface ISailsCtorFuncParams {
  /** ### List of argument names and types  */
  readonly args: ISailsFuncArg[];
  /** ### Encode payload to hex string */
  readonly encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  readonly decodePayload: <T>(bytes: HexString) => T;
  /** ### Create transaction builder from code */
  readonly fromCode: (code: Uint8Array | Buffer, ...args: unknown[]) => TransactionBuilder<any>;
  /** ### Create transaction builder from code id */
  readonly fromCodeId: (codeId: HexString, ...args: unknown[]) => TransactionBuilder<any>;
}

export class Sails {
  private _parser: ISailsIdlParser;
  private _program: ISailsProgram;
  private _scaleTypes: Record<string, any>;
  private _registry: TypeRegistry;
  private _api?: GearApi;
  private _programId?: HexString;

  constructor(parser?: ISailsIdlParser) {
    this._parser = parser;
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

  /**
   * ### Parse IDL from string
   * @param idl - IDL string
   */
  parseIdl(idl: string) {
    if (!this._parser) {
      throw new Error(
        'Parser not set. Use sails-js-parser package to initialize the parser and pass it to the Sails constructor.',
      );
    }
    this._program = this._parser.parse(idl);
    this.generateScaleCodeTypes();
    return this;
  }

  private generateScaleCodeTypes() {
    const scaleTypes: Record<string, any> = {};

    for (const type of this._program.types) {
      scaleTypes[type.name] = getScaleCodecDef(type.def);
    }

    this._registry = new TypeRegistry();
    this._registry.setKnownTypes({ types: scaleTypes });
    this._registry.register(scaleTypes);

    this._scaleTypes = scaleTypes;
  }

  /** #### Scale code types from the parsed IDL */
  get scaleCodecTypes() {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    return this._scaleTypes;
  }

  /** #### Registry with registered types from the parsed IDL */
  get registry() {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    return this._registry;
  }

  private _getFunctions(service: ISailsService): {
    funcs: Record<string, SailsServiceFunc>;
    queries: Record<string, SailsServiceQuery>;
  } {
    const funcs: Record<string, SailsServiceFunc> = {};
    const queries: Record<string, SailsServiceQuery> = {};

    for (const func of service.funcs) {
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def), typeDef: p.def }));
      const returnType = getScaleCodecDef(func.def);
      if (func.isQuery) {
        queries[func.name] = (async <T = any>(
          origin: string,
          value: bigint = 0n,
          atBlock?: HexString,
          ...args: unknown[]
        ): Promise<T> => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }
          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }
          const payload = this.registry
            .createType(`(String, String, ${params.map((p) => p.type).join(', ')})`, [service.name, func.name, ...args])
            .toHex();

          const reply = await this._api.message.calculateReply({
            destination: this.programId,
            origin: decodeAddress(origin),
            payload,
            value,
            gasLimit: this._api.blockGasLimit.toBigInt(),
            at: atBlock || null,
          });

          if (!reply.code.isSuccess) {
            throw new Error(this.registry.createType('String', reply.payload).toString());
          }

          const result = this.registry.createType(`(String, String, ${returnType})`, reply.payload.toHex());
          return result[2].toJSON() as T;
        }) as SailsServiceQuery;
      } else {
        funcs[func.name] = (<T = any>(...args: any): TransactionBuilder<T> => {
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
            [service.name, func.name, ...args],
            `(String, String, ${params.map((p) => p.type).join(', ')})`,
            returnType,
            this._programId,
          );
        }) as SailsServiceFunc;
      }

      Object.assign(func.isQuery ? queries[func.name] : funcs[func.name], {
        args: params,
        returnType,
        returnTypeDef: func.def,
        encodePayload: (...args: unknown[]): HexString => {
          if (args.length !== args.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          const payload = this.registry.createType(`(String, String, ${params.map((p) => p.type).join(', ')})`, [
            service.name,
            func.name,
            ...args,
          ]);

          return payload.toHex();
        },
        decodePayload: <T = any>(bytes: HexString) => {
          const payload = this.registry.createType(`(String, String, ${params.map((p) => p.type).join(', ')})`, bytes);
          const result = {} as Record<string, any>;
          params.forEach((param, i) => {
            result[param.name] = payload[i + 2].toJSON();
          });
          return result as T;
        },
        decodeResult: <T = any>(result: HexString) => {
          const payload = this.registry.createType(`(String, String, ${returnType})`, result);
          return payload[2].toJSON() as T;
        },
      });
    }

    return { funcs, queries };
  }

  private _getEvents(service: ISailsService): Record<string, SailsServiceEvent> {
    const events: Record<string, SailsServiceEvent> = {};

    for (const event of service.events) {
      const t = event.def ? getScaleCodecDef(event.def) : 'Null';
      const typeStr = event.def ? getScaleCodecDef(event.def, true) : 'Null';
      events[event.name] = {
        type: t,
        typeDef: event.def,
        is: ({ data: { message } }: UserMessageSent) => {
          if (!message.destination.eq(ZERO_ADDRESS)) {
            return false;
          }

          if (getServiceNamePrefix(message.payload.toHex()) !== service.name) {
            return false;
          }

          if (getFnNamePrefix(message.payload.toHex()) !== event.name) {
            return false;
          }

          return true;
        },
        decode: (payload: HexString) => {
          const data = this.registry.createType(`(String, String, ${typeStr})`, payload);
          return data[2].toJSON();
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
            const payload = message.payload.toHex();

            if (getServiceNamePrefix(payload) === service.name && getFnNamePrefix(payload) === event.name) {
              cb(this.registry.createType(`(String, String, ${typeStr})`, message.payload)[2].toJSON() as T);
            }
          });
        },
      };
    }

    return events;
  }

  /** #### Services with functions and events from the parsed IDL */
  get services(): Record<string, SailsService> {
    if (!this._program) {
      throw new Error('IDL is not parsed');
    }

    const services = {};

    for (const service of this._program.services) {
      const { funcs, queries } = this._getFunctions(service);
      services[service.name] = {
        functions: funcs,
        queries,
        events: this._getEvents(service),
      };
    }

    return services;
  }

  /** #### Constructor functions with arguments from the parsed IDL */
  get ctors() {
    if (!this._program) {
      throw new Error('IDL not parsed');
    }

    const ctor = this._program.ctor;

    if (!ctor) {
      return null;
    }

    const funcs: Record<string, ISailsCtorFuncParams> = {};

    for (const func of ctor.funcs) {
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def), typeDef: p.def }));
      funcs[func.name] = {
        args: params,
        encodePayload: (...args): HexString => {
          if (args.length !== args.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          if (params.length === 0) {
            return u8aToHex(this.registry.createType('String', func.name).toU8a());
          }

          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, [
            func.name,
            ...args,
          ]);

          return payload.toHex();
        },
        decodePayload: <T = any>(bytes: Uint8Array | string) => {
          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, bytes);
          const result = {} as Record<string, any>;
          params.forEach((param, i) => {
            result[param.name] = payload[i + 1].toJSON();
          });
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
            [func.name, ...args],
            `(String, ${params.map((p) => p.type).join(', ')})`,
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
            [func.name, ...args],
            `(String, ${params.map((p) => p.type).join(', ')})`,
            'String',
            codeId,
          );

          this._programId = builder.programId;
          return builder;
        },
      };
    }

    return funcs;
  }

  /** #### Parsed IDL */
  get program() {
    if (!this._program) {
      throw new Error('IDL is not parsed');
    }

    return this._program;
  }

  /** #### Get type definition by name */
  getTypeDef(name: string): ISailsTypeDef {
    return this.program.getTypeByName(name).def;
  }
}
