import {
  ISailsEnumDef,
  ISailsEnumVariant,
  ISailsFixedSizeArrayDef,
  ISailsMapDef,
  ISailsOptionalDef,
  ISailsPrimitiveDef,
  ISailsResultDef,
  ISailsStructDef,
  ISailsStructField,
  ISailsType,
  ISailsTypeDef,
  ISailsUserDefinedDef,
  ISailsVecDef,
  IWithDefEntity,
} from 'sails-js-types';
import { getName, getText } from './util.js';
import { Base } from './visitor.js';

export class WithDef extends Base implements IWithDefEntity {
  private _def: TypeDef;

  setDef(def: TypeDef): void {
    if (this._def) throw new Error('def already set');

    this._def = def;
  }

  get def(): TypeDef {
    return this._def;
  }
}

export class Type extends WithDef implements ISailsType {
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

export class TypeDef implements ISailsTypeDef {
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
  ActorId,
  CodeId,
  MessageId,
  H256,
  U256,
  H160,
  NonZeroU8,
  NonZeroU16,
  NonZeroU32,
  NonZeroU64,
  NonZeroU128,
  NonZeroU256,
}

export class PrimitiveDef implements ISailsPrimitiveDef {
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

  get isActorId(): boolean {
    return this.value === EPrimitiveType.ActorId;
  }

  get isCodeId(): boolean {
    return this.value === EPrimitiveType.CodeId;
  }

  get isMessageId(): boolean {
    return this.value === EPrimitiveType.MessageId;
  }

  get isH256(): boolean {
    return this.value === EPrimitiveType.H256;
  }

  get isU256(): boolean {
    return this.value === EPrimitiveType.U256;
  }

  get isH160(): boolean {
    return this.value === EPrimitiveType.H160;
  }

  get isNonZeroU8(): boolean {
    return this.value === EPrimitiveType.NonZeroU8;
  }

  get isNonZeroU16(): boolean {
    return this.value === EPrimitiveType.NonZeroU16;
  }

  get isNonZeroU32(): boolean {
    return this.value === EPrimitiveType.NonZeroU32;
  }

  get isNonZeroU64(): boolean {
    return this.value === EPrimitiveType.NonZeroU64;
  }

  get isNonZeroU128(): boolean {
    return this.value === EPrimitiveType.NonZeroU128;
  }

  get isNonZeroU256(): boolean {
    return this.value === EPrimitiveType.NonZeroU256;
  }
}

export class OptionalDef extends WithDef implements ISailsOptionalDef {}

export class VecDef extends WithDef implements ISailsVecDef {}

export class ResultDef implements ISailsResultDef {
  public readonly ok: WithDef;
  public readonly err: WithDef;

  constructor(ok_ptr: number, err_ptr: number, memory: WebAssembly.Memory) {
    this.ok = new WithDef(ok_ptr, memory);
    this.err = new WithDef(err_ptr, memory);
  }
}

export class MapDef implements ISailsMapDef {
  public readonly key: WithDef;
  public readonly value: WithDef;

  constructor(keyPtr: number, valuePtr: number, memory: WebAssembly.Memory) {
    this.key = new WithDef(keyPtr, memory);
    this.value = new WithDef(valuePtr, memory);
  }
}

export class StructDef extends Base implements ISailsStructDef {
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

export class EnumDef extends Base implements ISailsEnumDef {
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

export class StructField extends WithDef implements ISailsStructField {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;
  }
}

export class EnumVariant extends WithDef implements ISailsEnumVariant {
  public readonly name: string;

  constructor(ptr: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    const { name, offset } = getName(ptr, this.offset, memory);

    this.name = name;
    this.offset = offset;
  }
}

export class FixedSizeArrayDef extends WithDef implements ISailsFixedSizeArrayDef {
  public readonly len: number;

  constructor(ptr: number, len: number, memory: WebAssembly.Memory) {
    super(ptr, memory);

    this.len = len;
  }
}

export class UserDefinedDef implements ISailsUserDefinedDef {
  public readonly name: string;

  constructor(ptr: number, len: number, memory: WebAssembly.Memory) {
    this.name = getText(ptr, len, memory);
  }
}
