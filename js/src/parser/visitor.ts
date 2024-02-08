const getText = (ptr: number, len: number, memory: WebAssembly.Memory): string => {
  const buf = new Uint8Array(memory.buffer.slice(ptr, ptr + len));
  return new TextDecoder().decode(buf);
};

const getName = (ptr: number, offset: number, memory: WebAssembly.Memory): { name: string; offset: number } => {
  const name_ptr_buf = new Uint8Array(memory.buffer.slice(ptr + offset, ptr + offset + 4));
  offset += 4;
  const name_ptr_dv = new DataView(name_ptr_buf.buffer, 0);
  const name_ptr = name_ptr_dv.getUint32(0, true);

  const name_len_buf = new Uint8Array(memory.buffer.slice(ptr + offset, ptr + offset + 4));
  offset += 4;
  const name_len_dv = new DataView(name_len_buf.buffer, 0);
  const name_len = name_len_dv.getUint32(0, true);

  const name = getText(name_ptr, name_len, memory);

  return { name, offset };
};

class Base {
  protected offset: number;
  public readonly rawPtr: number;

  constructor(public ptr: number, memory: WebAssembly.Memory) {
    const rawPtrBuf = new Uint8Array(memory.buffer.slice(ptr, ptr + 4));
    const rawPtrDv = new DataView(rawPtrBuf.buffer, 0);
    this.rawPtr = rawPtrDv.getUint32(0, true);
    this.offset = 4;
  }
}

export class Program {
  private _service: Service;
  private _types: Map<number, Type>;
  private _context: Map<number, WithDef>;

  constructor() {
    this._service = null;
    this._types = new Map();
    this._context = new Map();
  }

  addService(service: Service) {
    this._service = service;
  }

  addType(type: Type) {
    const id = type.rawPtr;
    this._types.set(id, type);
    this._context.set(id, type);
    return id;
  }

  get service(): Service {
    return this._service;
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
}

class WithDef extends Base {
  private _def: TypeDef;

  setDef(def: TypeDef): void {
    this._def = def;
  }

  get def(): TypeDef {
    return this._def;
  }
}

export class Type extends WithDef {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;
  }
}

export enum DefKind {
  Struct,
  Enum,
  Optional,
  Primitive,
  Result,
  Vec,
  UserDefined,
  FixedSizeArray,
  Map,
}

type DefVariants =
  | StructDef
  | EnumDef
  | OptionalDef
  | PrimitiveDef
  | ResultDef
  | VecDef
  | UserDefinedDef
  | MapDef
  | FixedSizeArrayDef;

export class TypeDef {
  private _def: DefVariants;
  private _kind: DefKind;

  constructor(def: DefVariants, kind: DefKind) {
    this._def = def;
    this._kind = kind;
  }

  get isStruct(): boolean {
    return this._kind === DefKind.Struct;
  }

  get isEnum(): boolean {
    return this._kind === DefKind.Enum;
  }

  get isOptional(): boolean {
    return this._kind === DefKind.Optional;
  }

  get isPrimitive(): boolean {
    return this._kind === DefKind.Primitive;
  }

  get isResult(): boolean {
    return this._kind === DefKind.Result;
  }

  get isVec(): boolean {
    return this._kind === DefKind.Vec;
  }

  get isMap(): boolean {
    return this._kind === DefKind.Map;
  }

  get isFixedSizeArray(): boolean {
    return this._kind === DefKind.FixedSizeArray;
  }

  get isUserDefined(): boolean {
    return this._kind === DefKind.UserDefined;
  }

  get asStruct(): StructDef {
    if (!this.isStruct) throw new Error('not a struct');

    return this._def as StructDef;
  }

  get asEnum(): EnumDef {
    if (!this.isEnum) throw new Error('not a enum');

    return this._def as EnumDef;
  }

  get asOptional(): OptionalDef {
    if (!this.isOptional) throw new Error('not a optional');

    return this._def as OptionalDef;
  }

  get asPrimitive(): PrimitiveDef {
    if (!this.isPrimitive) throw new Error('not a primitive');

    return this._def as PrimitiveDef;
  }

  get asResult(): ResultDef {
    if (!this.isResult) throw new Error('not a result');

    return this._def as ResultDef;
  }

  get asVec(): VecDef {
    if (!this.isVec) throw new Error('not a vec');

    return this._def as VecDef;
  }

  get asUserDefined(): UserDefinedDef {
    if (!this.isUserDefined) throw new Error('not a user defined');

    return this._def as UserDefinedDef;
  }

  get asMap(): MapDef {
    if (!this.isMap) throw new Error('not a map');

    return this._def as MapDef;
  }

  get asFixedSizeArray(): FixedSizeArrayDef {
    if (!this.isFixedSizeArray) throw new Error('not a fixed size array');

    return this._def as FixedSizeArrayDef;
  }
}

export enum EPrimitiveType {
  Null,
  Bool,
  Char,
  Str,
  U8,
  U16,
  U32,
  U64,
  U128,
  I8,
  I16,
  I32,
  I64,
  I128,
}

export class PrimitiveDef {
  constructor(private value: number) {}

  get isNull(): boolean {
    return this.value === EPrimitiveType.Null;
  }

  get isBool(): boolean {
    return this.value === EPrimitiveType.Bool;
  }

  get isChar(): boolean {
    return this.value === EPrimitiveType.Char;
  }

  get isStr(): boolean {
    return this.value === EPrimitiveType.Str;
  }

  get isU8(): boolean {
    return this.value === EPrimitiveType.U8;
  }

  get isU16(): boolean {
    return this.value === EPrimitiveType.U16;
  }

  get isU32(): boolean {
    return this.value === EPrimitiveType.U32;
  }

  get isU64(): boolean {
    return this.value === EPrimitiveType.U64;
  }

  get isU128(): boolean {
    return this.value === EPrimitiveType.U128;
  }

  get isI8(): boolean {
    return this.value === EPrimitiveType.I8;
  }

  get isI16(): boolean {
    return this.value === EPrimitiveType.I16;
  }

  get isI32(): boolean {
    return this.value === EPrimitiveType.I32;
  }

  get isI64(): boolean {
    return this.value === EPrimitiveType.I64;
  }

  get isI128(): boolean {
    return this.value === EPrimitiveType.I128;
  }
}

export class OptionalDef extends WithDef {}

export class VecDef extends WithDef {}

export class ResultDef {
  public readonly ok: WithDef;
  public readonly err: WithDef;

  constructor(ok_ptr: number, err_ptr: number, memory: WebAssembly.Memory) {
    this.ok = new WithDef(ok_ptr, memory);
    this.err = new WithDef(err_ptr, memory);
  }
}

export class MapDef {
  public readonly key: WithDef;
  public readonly value: WithDef;

  constructor(keyPtr: number, valuePtr: number, memory: WebAssembly.Memory) {
    this.key = new WithDef(keyPtr, memory);
    this.value = new WithDef(valuePtr, memory);
  }
}

export class StructDef extends Base {
  private _fields: Map<number, StructField>;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    this._fields = new Map();
  }

  addField(field: StructField) {
    const id = field.rawPtr;
    this._fields.set(id, field);
    return id;
  }

  get fields(): StructField[] {
    return Array.from(this._fields.values());
  }

  get isTuple(): boolean {
    return this.fields.every((f) => f.name === '');
  }
}

export class EnumDef extends Base {
  private _variants: Map<number, EnumVariant>;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    this._variants = new Map();
  }

  addVariant(variant: EnumVariant) {
    const id = variant.rawPtr;
    this._variants.set(id, variant);
    return id;
  }

  get variants(): EnumVariant[] {
    return Array.from(this._variants.values());
  }

  get isNesting(): boolean {
    return this.variants.some((v) => !!v.def);
  }
}

export class StructField extends WithDef {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;
  }
}

export class EnumVariant extends WithDef {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;
  }
}

export class FixedSizeArrayDef extends WithDef {
  public readonly len: number;

  constructor(ptr: number, len: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    this.len = len;
  }
}

export class UserDefinedDef {
  public readonly name: string;

  constructor(ptr: number, len: number, memory: WebAssembly.Memory) {
    this.name = getText(ptr, len, memory);
  }
}

export class Service extends Base {
  public readonly funcs: Func[];

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    this.funcs = [];
  }

  addFunc(func: Func) {
    this.funcs.push(func);
  }
}

export class Func extends WithDef {
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

export class FuncParam extends WithDef {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;
  }
}
