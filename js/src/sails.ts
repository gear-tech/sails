import { TypeRegistry } from '@polkadot/types/create';
import { u8aToHex } from '@polkadot/util';
import { HexString, UserMessageSent } from '@gear-js/api';

import { Program, Service, TypeDef, WasmParser } from './parser/index.js';
import { getScaleCodecDef } from './utils/types.js';
import { getFnNamePrefix, getServiceNamePrefix } from 'utils/prefix.js';

const ZERO_ADDRESS = u8aToHex(new Uint8Array(32));

interface SailsService {
  functions: Record<string, SailsServiceFunc>;
  events: Record<string, SailsServiceEvent>;
}

interface SailsServiceFunc {
  args: { name: string; type: any }[];
  returnType: any;
  isQuery: boolean;
  encodePayload: (...args: any[]) => HexString;
  decodePayload: <T>(bytes: HexString) => T;
  decodeResult: <T>(result: HexString) => T;
}

interface SailsServiceEvent {
  type: any;
  is: (event: UserMessageSent) => boolean;
  decode: (payload: HexString) => any;
}

interface SailsCtorFunc {
  args: { name: string; type: any }[];
  encodePayload: (...args: any[]) => Uint8Array;
  decodePayload: <T>(bytes: HexString) => T;
}

export class Sails {
  private _parser: WasmParser;
  private _program: Program;
  private _scaleTypes: Record<string, any>;
  private _registry: TypeRegistry;

  constructor(parser: WasmParser) {
    this._parser = parser;
  }

  /** #### Create new Sails instance */
  static async new() {
    const parser = new WasmParser();
    return new Sails(await parser.init());
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

  private _getFunctions(service: Service): Record<string, SailsServiceFunc> {
    const funcs: Record<string, SailsServiceFunc> = {};

    for (const func of service.funcs) {
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def) }));
      const returnType = getScaleCodecDef(func.def);
      funcs[func.name] = {
        args: params,
        returnType,
        isQuery: func.isQuery,
        encodePayload: (...args): HexString => {
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
      };
    }

    return funcs;
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
      services[service.name] = {
        functions: this._getFunctions(service),
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

    const funcs: Record<string, SailsCtorFunc> = {};

    for (const func of ctor.funcs) {
      const params = func.params.map((p) => ({ name: p.name, type: getScaleCodecDef(p.def) }));
      funcs[func.name] = {
        args: params,
        encodePayload: (...args): Uint8Array => {
          if (args.length !== args.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, [
            func.name,
            ...args,
          ]);

          return payload.toU8a();
        },
        decodePayload: <T = any>(bytes: Uint8Array | string) => {
          const payload = this.registry.createType(`(String, ${params.map((p) => p.type).join(', ')})`, bytes);
          return payload[1].toJSON() as T;
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
