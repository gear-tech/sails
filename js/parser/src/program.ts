import { ISailsCtor, ISailsCtorFunc, ISailsProgram } from 'sails-js-types';
import { FuncParam, Service } from './service.js';
import { Type, WithDef } from './types.js';
import { getName } from './util.js';
import { Base } from './visitor.js';

export class Program implements ISailsProgram {
  private _services: Service[];
  private _types: Map<number, Type>;
  private _context: Map<number, WithDef>;
  private _ctor: Ctor;

  constructor() {
    this._services = [];
    this._types = new Map();
    this._context = new Map();
  }

  addService(service: Service) {
    this._services.push(service);
  }

  addType(type: Type) {
    const id = type.rawPtr;
    this._types.set(id, type);
    this._context.set(id, type);
    return id;
  }

  get services(): Service[] {
    return this._services;
  }

  get ctor(): Ctor {
    return this._ctor;
  }

  getType(id: number): Type {
    return this._types.get(id);
  }

  getContext(id: number): any {
    return this._context.get(id);
  }

  addContext(id: number, ctx: any) {
    this._context.set(id, ctx);
  }

  get types(): Type[] {
    return Array.from(this._types.values());
  }

  getTypeByName(name: string): Type {
    const types = this.types.filter((type) => type.name === name);
    if (types.length > 1) throw new Error(`multiple types found with name ${name}`);
    if (types.length === 0) throw new Error(`no type found with name ${name}`);

    return types[0];
  }

  addCtor(ctor: Ctor) {
    this._ctor = ctor;
  }
}

export class Ctor extends Base implements ISailsCtor {
  public readonly funcs: CtorFunc[];

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    this.funcs = [];
  }

  addFunc(func: CtorFunc) {
    this.funcs.push(func);
  }
}

export class CtorFunc extends Base implements ISailsCtorFunc {
  private _params: Map<number, FuncParam>;
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);
    this.name = name;
    this.offset = offset;

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
