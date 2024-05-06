import { GearApi, HexString, UserMessageSent, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types/create';
import { u8aToHex } from '@polkadot/util';

import { Program, Service, TypeDef, WasmParser } from './parser/index.js';
import { getFnNamePrefix, getServiceNamePrefix } from 'utils/prefix.js';
import { TransactionBuilder } from './transaction-builder.js';
import { getScaleCodecDef } from './utils/types.js';

const ZERO_ADDRESS = u8aToHex(new Uint8Array(32));

interface SailsService {
  functions: Record<string, SailsServiceFunc>;
  queries: Record<string, SailsServiceQuery>;
  events: Record<string, SailsServiceEvent>;
}

interface ISailsServiceFuncParams {
  /** ### List of argument names and types  */
  args: { name: string; type: any }[];
  /** ### Function return type */
  returnType: any;
  /** ### Encode payload to hex string */
  encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  decodePayload: <T extends any = any>(bytes: HexString) => T;
  /** ### Decode function result */
  decodeResult: <T extends any = any>(result: HexString) => T;
}

type SailsServiceQuery = ISailsServiceFuncParams &
  (<T>(origin: string, value?: bigint, atBlock?: HexString, ...args: unknown[]) => Promise<T>);

type SailsServiceFunc = ISailsServiceFuncParams & (<T>(...args: unknown[]) => TransactionBuilder<T>);

interface SailsServiceEvent {
  type: any;
  is: (event: UserMessageSent) => boolean;
  decode: (payload: HexString) => any;
  subscribe: <T>(cb: (event: T) => void | Promise<void>) => void;
}

interface ISailsCtorFuncParams {
  /** ### List of argument names and types  */
  args: { name: string; type: any }[];
  /** ### Encode payload to hex string */
  encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  decodePayload: <T>(bytes: HexString) => T;
  /** ### Create transaction builder from code */
  fromCode: (code: Uint8Array | Buffer, ...args: unknown[]) => TransactionBuilder<any>;
  /** ### Create transaction builder from code id */
  fromCodeId: (codeId: HexString, ...args: unknown[]) => TransactionBuilder<any>;
}

export class Sails {
  private _parser: WasmParser;
  private _program: Program;
  private _scaleTypes: Record<string, any>;
  private _registry: TypeRegistry;
  private _api?: GearApi;
  private _programId?: HexString;

  constructor(parser: WasmParser) {
    this._parser = parser;
  }

  /** #### Create new Sails instance */
  static async new() {
    const parser = new WasmParser();
    return new Sails(await parser.init());
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

  private _getFunctions(service: Service): {
    funcs: Record<string, SailsServiceFunc>;
    queries: Record<string, SailsServiceQuery>;
  } {
    const funcs: Record<string, SailsServiceFunc> = {};
    const queries: Record<string, SailsServiceQuery> = {};

    for (const func of service.funcs) {
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def) }));
      const returnType = getScaleCodecDef(func.def);
      if (func.isQuery) {
        queries[func.name] = (async <T extends any = any>(
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
        funcs[func.name] = (<T extends any = any>(...args: any): TransactionBuilder<T> => {
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
          return payload[2].toJSON() as T;
        },
        decodeResult: <T = any>(result: HexString) => {
          const payload = this.registry.createType(`(String, String, ${returnType})`, result);
          return payload[2].toJSON() as T;
        },
      });
    }

    return { funcs, queries };
  }

  private _getEvents(service: Service): Record<string, SailsServiceEvent> {
    const events: Record<string, SailsServiceEvent> = {};

    for (const event of service.events) {
      const t = event.def ? getScaleCodecDef(event.def) : 'Null';
      const typeStr = event.def ? getScaleCodecDef(event.def, true) : 'Null';
      events[event.name] = {
        type: t,
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
        subscribe: <T extends any = any>(cb: (eventData: T) => void | Promise<void>) => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }

          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }

          this._api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
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
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def) }));
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
          return payload[1].toJSON() as T;
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
  getTypeDef(name: string): TypeDef {
    return this.program.getTypeByName(name).def;
  }
}
