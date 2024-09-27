import { ISailsTypeDef } from 'sails-js-types';
import { PayloadMethod } from './payload-method.js';

export const getJsTypeDef = (
  typeDef: ISailsTypeDef,
  payloadMethod?: PayloadMethod,
): { type: string; imports: string[] } => {
  if (typeDef.isPrimitive) {
    if (payloadMethod === PayloadMethod.toNumber) {
      return { type: 'number', imports: [] };
    }
    if (payloadMethod === PayloadMethod.toBigInt) {
      return { type: 'bigint', imports: [] };
    }

    const primitive = typeDef.asPrimitive;

    if (primitive.isBool) return { type: 'boolean', imports: [] };
    if (primitive.isChar) return { type: 'string', imports: [] };
    if (primitive.isNull) return { type: 'null', imports: [] };
    if (primitive.isStr) return { type: 'string', imports: [] };
    if (primitive.isI8 || primitive.isI16 || primitive.isI32 || primitive.isU8 || primitive.isU16 || primitive.isU32)
      return { type: 'number', imports: [] };
    if (primitive.isI64 || primitive.isI128 || primitive.isU64 || primitive.isU128 || primitive.isU256)
      return { type: 'number | string | bigint', imports: [] };
    if (primitive.isActorId) return { type: 'ActorId', imports: ['ActorId'] };
    if (primitive.isCodeId) return { type: 'CodeId', imports: ['CodeId'] };
    if (primitive.isMessageId) return { type: 'MessageId', imports: ['MessageId'] };
    if (primitive.isH256) return { type: 'H256', imports: ['H256'] };
    if (primitive.isH160) return { type: 'H160', imports: ['H160'] };
    if (primitive.isNonZeroU8) return { type: 'NonZeroU8', imports: ['NonZeroU8'] };
    if (primitive.isNonZeroU16) return { type: 'NonZeroU16', imports: ['NonZeroU16'] };
    if (primitive.isNonZeroU32) return { type: 'NonZeroU32', imports: ['NonZeroU32'] };
    if (primitive.isNonZeroU64) return { type: 'NonZeroU64', imports: ['NonZeroU64'] };
    if (primitive.isNonZeroU128) return { type: 'NonZeroU128', imports: ['NonZeroU128'] };
    if (primitive.isNonZeroU256) return { type: 'NonZeroU256', imports: ['NonZeroU256'] };
  }
  if (typeDef.isOptional) {
    const inner = getJsTypeDef(typeDef.asOptional.def);
    return { type: `${inner.type} | null`, imports: inner.imports };
  }
  if (typeDef.isResult) {
    const ok = getJsTypeDef(typeDef.asResult.ok.def);
    const err = getJsTypeDef(typeDef.asResult.err.def);
    return { type: `{ ok: ${ok.type} } | { err: ${err.type} }`, imports: [...ok.imports, ...err.imports] };
  }
  if (typeDef.isVec) {
    if (typeDef.asVec.def.isPrimitive && typeDef.asVec.def.asPrimitive.isU8) {
      return { type: '`0x${string}`', imports: [] };
    }

    const inner = getJsTypeDef(typeDef.asVec.def);

    return { type: `Array<${inner.type}>`, imports: inner.imports };
  }
  if (typeDef.isFixedSizeArray) {
    const inner = getJsTypeDef(typeDef.asFixedSizeArray.def);
    return { type: `Array<${inner.type}>`, imports: inner.imports };
  }
  if (typeDef.isMap) {
    const key = getJsTypeDef(typeDef.asMap.key.def);
    const value = getJsTypeDef(typeDef.asMap.value.def);
    return { type: `Record<${key.type}, ${value.type}>`, imports: [...key.imports, ...value.imports] };
  }
  if (typeDef.isUserDefined) {
    return { type: typeDef.asUserDefined.name, imports: [] };
  }
  if (typeDef.isStruct) {
    if (typeDef.asStruct.isTuple) {
      const fields = typeDef.asStruct.fields.map((f) => getJsTypeDef(f.def));
      return { type: `[${fields.map((f) => f.type).join(', ')}]`, imports: fields.flatMap((f) => f.imports) };
    } else {
      const imports = [];
      const def = typeDef.asStruct.fields
        .map((f) => {
          const fType = getJsTypeDef(f.def);
          imports.push(...fType.imports);
          return `${f.name}: ${fType.type}`;
        })
        .join('; ');
      return { type: `{ ${def} }`, imports };
    }
  }

  throw new Error('Unknown type :: ' + JSON.stringify(typeDef));
};

export const getScaleCodecDef = (type: ISailsTypeDef, asString = false) => {
  if (type.isPrimitive) {
    const primitive = type.asPrimitive;
    if (primitive.isBool) return 'bool';
    if (primitive.isChar) return 'char';
    if (primitive.isNull) return 'Null';
    if (primitive.isStr) return 'String';
    if (primitive.isI8) return 'i8';
    if (primitive.isI16) return 'i16';
    if (primitive.isI32) return 'i32';
    if (primitive.isI64) return 'i64';
    if (primitive.isI128) return 'i128';
    if (primitive.isU8 || primitive.isNonZeroU8) return 'u8';
    if (primitive.isU16 || primitive.isNonZeroU16) return 'u16';
    if (primitive.isU32 || primitive.isNonZeroU32) return 'u32';
    if (primitive.isU64 || primitive.isNonZeroU64) return 'u64';
    if (primitive.isU128 || primitive.isNonZeroU128) return 'u128';
    if (primitive.isU256 || primitive.isNonZeroU256) return 'U256';
    if (primitive.isActorId || primitive.isCodeId || primitive.isMessageId) return '[u8;32]';
    if (primitive.isH256) return 'H256';
    if (primitive.isH160) return 'H160';
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
