import { ISailsDocs } from './idl';

export interface IWithDefEntity {
  readonly def: ISailsTypeDef;
}

export interface ISailsType extends IWithDefEntity, ISailsDocs {
  readonly name: string;
}

export interface ISailsTypeDef {
  readonly isPrimitive: boolean;
  readonly asPrimitive: ISailsPrimitiveDef;
  readonly isStruct: boolean;
  readonly asStruct: ISailsStructDef;
  readonly isEnum: boolean;
  readonly asEnum: ISailsEnumDef;
  readonly isOptional: boolean;
  readonly asOptional: ISailsOptionalDef;
  readonly isResult: boolean;
  readonly asResult: ISailsResultDef;
  readonly isVec: boolean;
  readonly asVec: ISailsVecDef;
  readonly isMap: boolean;
  readonly asMap: ISailsMapDef;
  readonly isFixedSizeArray: boolean;
  readonly asFixedSizeArray: ISailsFixedSizeArrayDef;
  readonly isUserDefined: boolean;
  readonly asUserDefined: ISailsUserDefinedDef;
}

export interface ISailsPrimitiveDef {
  readonly isNull: boolean;
  readonly isBool: boolean;
  readonly isChar: boolean;
  readonly isStr: boolean;
  readonly isU8: boolean;
  readonly isU16: boolean;
  readonly isU32: boolean;
  readonly isU64: boolean;
  readonly isU128: boolean;
  readonly isI8: boolean;
  readonly isI16: boolean;
  readonly isI32: boolean;
  readonly isI64: boolean;
  readonly isI128: boolean;
  readonly isActorId: boolean;
  readonly isCodeId: boolean;
  readonly isMessageId: boolean;
  readonly isH256: boolean;
  readonly isU256: boolean;
  readonly isH160: boolean;
  readonly isNonZeroU8: boolean;
  readonly isNonZeroU16: boolean;
  readonly isNonZeroU32: boolean;
  readonly isNonZeroU64: boolean;
  readonly isNonZeroU128: boolean;
  readonly isNonZeroU256: boolean;
}

export interface ISailsStructDef {
  readonly fields: ISailsStructField[];
  readonly isTuple: boolean;
}

export interface ISailsStructField extends IWithDefEntity, ISailsDocs {
  readonly name: string;
}

export interface ISailsEnumDef {
  readonly variants: ISailsEnumVariant[];
  readonly isNesting: boolean;
}

export interface ISailsEnumVariant extends IWithDefEntity, ISailsDocs {
  readonly name: string;
}

export type ISailsOptionalDef = IWithDefEntity;

export interface ISailsResultDef {
  readonly ok: IWithDefEntity;
  readonly err: IWithDefEntity;
}

export type ISailsVecDef = IWithDefEntity;

export interface ISailsMapDef {
  readonly key: IWithDefEntity;
  readonly value: IWithDefEntity;
}

export interface ISailsFixedSizeArrayDef extends IWithDefEntity {
  readonly len: number;
}

export interface ISailsUserDefinedDef {
  readonly name: string;
}
