import { PrimitiveDef, TypeDef } from '../parser/types.js';
import { PaylodMethod } from './payload-method.js';

export const getJsTypeDef = (type: TypeDef, payloadMethod?: PaylodMethod): string => {
  if (type.isPrimitive) {
    if (payloadMethod === PaylodMethod.toNumber) {
      return 'number';
    }
    if (payloadMethod === PaylodMethod.toBigInt) {
      return 'bigint';
    }
    return getPrimitiveTypeName(type.asPrimitive, true);
  }
  if (type.isOptional) {
    return `${getJsTypeDef(type.asOptional.def)} | null`;
  }
  if (type.isResult) {
    return `{ ok: ${getJsTypeDef(type.asResult.ok.def)} } | { err: ${getJsTypeDef(type.asResult.err.def)} }`;
  }
  if (type.isVec) {
    return `Array<${getJsTypeDef(type.asVec.def)}>`;
  }
  if (type.isFixedSizeArray) {
    return `Array<${getJsTypeDef(type.asFixedSizeArray.def)}>`;
  }
  if (type.isMap) {
    return `Record<${getJsTypeDef(type.asMap.key.def)}, ${getJsTypeDef(type.asMap.value.def)}>`;
  }
  if (type.isUserDefined) {
    return type.asUserDefined.name;
  }
  if (type.isStruct) {
    if (type.asStruct.fields[0].name === '') {
      return `[${type.asStruct.fields.map(({ def }) => getJsTypeDef(def)).join(', ')}]`;
    }
    const def = type.asStruct.fields.map((f) => `${f.name}: ${getJsTypeDef(f.def)}`).join('; ');
    return `{ ${def} }`;
  }

  throw new Error('Unknown type :: ' + JSON.stringify(type));
};

export const getPrimitiveTypeName = (type: PrimitiveDef, forTs = false): string => {
  if (type.isBool) return forTs ? 'boolean' : 'bool';
  if (type.isChar) return forTs ? 'string' : 'char';
  if (type.isNull) return forTs ? 'null' : 'Null';
  if (type.isStr) return forTs ? 'string' : 'String';
  if (type.isI8) return forTs ? 'number' : 'i8';
  if (type.isI16) return forTs ? 'number' : 'i16';
  if (type.isI32) return forTs ? 'number' : 'i32';
  if (type.isI64) return forTs ? 'number | string' : 'i64';
  if (type.isI128) return forTs ? 'number | string' : 'i128';
  if (type.isU8) return forTs ? 'number' : 'u8';
  if (type.isU16) return forTs ? 'number' : 'u16';
  if (type.isU32) return forTs ? 'number' : 'u32';
  if (type.isU64) return forTs ? 'number | string' : 'u64';
  if (type.isU128) return forTs ? 'number | string' : 'u128';
  if (type.isU256) return forTs ? 'number | string' : 'U256';
  if (type.isActorId || type.isCodeId || type.isMessageId) return forTs ? 'string' : '[u8;32]';
  if (type.isH256) return forTs ? 'string' : 'H256';

  throw new Error('Unknown primitive type');
};

export const getScaleCodecDef = (type: TypeDef, asString = false) => {
  if (type.isPrimitive) {
    return getPrimitiveTypeName(type.asPrimitive);
  }
  if (type.isOptional) {
    return `Option<${getScaleCodecDef(type.asOptional.def)}>`;
  }
  if (type.isResult) {
    return `Result<${getScaleCodecDef(type.asResult.ok.def)}, ${getScaleCodecDef(type.asResult.err.def)}>`;
  }
  if (type.isVec) {
    return `Vec<${getScaleCodecDef(type.asVec.def)}>`;
  }
  if (type.isFixedSizeArray) {
    return `[${getScaleCodecDef(type.asFixedSizeArray.def)}; ${type.asFixedSizeArray.len}]`;
  }
  if (type.isMap) {
    return `BTreeMap<${getScaleCodecDef(type.asMap.key.def)}, ${getScaleCodecDef(type.asMap.value.def)}>`;
  }
  if (type.isUserDefined) {
    return type.asUserDefined.name;
  }
  if (type.isStruct) {
    if (type.asStruct.isTuple) {
      return `(${type.asStruct.fields.map(({ def }) => getScaleCodecDef(def)).join(', ')})`;
    }
    const result = {};
    for (const field of type.asStruct.fields) {
      result[field.name] = getScaleCodecDef(field.def);
    }
    return asString ? JSON.stringify(result) : result;
  }
  if (type.isEnum) {
    if (!type.asEnum.isNesting) {
      return { _enum: type.asEnum.variants.map((v) => v.name) };
    }
    const result = {};
    for (const variant of type.asEnum.variants) {
      result[variant.name] = variant.def ? getScaleCodecDef(variant.def) : 'Null';
    }
    return { _enum: result };
  }

  throw new Error('Unknown type :: ' + JSON.stringify(type));
};
