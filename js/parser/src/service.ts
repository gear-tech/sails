import { ISailsFuncParam, ISailsService, ISailsServiceEvent, ISailsServiceFunc } from 'sails-js-types';
import { EnumVariant, WithDef } from './types.js';
import { getName } from './util.js';
import { Base } from './visitor.js';

export class Service extends Base implements ISailsService {
  public readonly funcs: ServiceFunc[];
  public readonly events: ServiceEvent[];
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name || 'Service';
    this.offset = offset;

    this.funcs = [];
    this.events = [];
  }

  addFunc(func: ServiceFunc) {
    this.funcs.push(func);
  }

  addEvent(event: ServiceEvent) {
    this.events.push(event);
  }
}

export class ServiceEvent extends EnumVariant implements ISailsServiceEvent {}

export class ServiceFunc extends WithDef implements ISailsServiceFunc {
  public readonly name: string;
  public readonly isQuery: boolean;
  private _params: Map<number, FuncParam>;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;

    const is_query_buf = new Uint8Array(memory.buffer.slice(ptr + this.offset, ptr + this.offset + 1));
    const is_query_dv = new DataView(is_query_buf.buffer, 0);
    this.isQuery = is_query_dv.getUint8(0) === 1;

    this._params = new Map();
  }

  addFuncParam(ptr: number, param: FuncParam) {
    this._params.set(ptr, param);
  }

  get params(): FuncParam[] {
    if (this._params.size === 0) return [];

    return Array.from(this._params.values());
  }
}

export class FuncParam extends WithDef implements ISailsFuncParam {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;
  }
}
