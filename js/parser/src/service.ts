import { ISailsFuncParam, ISailsService, ISailsServiceEvent, ISailsServiceFunc } from 'sails-js-types';
import { EnumVariant, WithDef } from './types.js';
import { getBool, getDocs, getName } from './util.js';
import { Base } from './visitor.js';

export class Service extends Base implements ISailsService {
  public readonly funcs: ISailsServiceFunc[];
  public readonly events: ServiceEvent[];
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const [name, nameOffset] = getName(ptr, this.offset, memory);

    this.name = name || 'Service';
    this.offset += nameOffset;

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
  public readonly docs?: string;
  private _params: Map<number, FuncParam>;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const [name, nameOffset] = getName(ptr, this.offset, memory);
    this.name = name;
    this.offset += nameOffset;

    const [isQuery, isQueryOffset] = getBool(ptr, this.offset, memory);
    this.isQuery = isQuery;
    this.offset += isQueryOffset;

    const [docs, docsOffset] = getDocs(ptr, this.offset, memory);
    this.docs = docs;
    this.offset += docsOffset;

    this._params = new Map();
  }

  addFuncParam(ptr: number, param: FuncParam) {
    this._params.set(ptr, param);
  }

  get params(): ISailsFuncParam[] {
    if (this._params.size === 0) return [];

    return [...this._params.values()];
  }
}

export class FuncParam extends WithDef implements ISailsFuncParam {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const [name, nameOffset] = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset += nameOffset;
  }
}
