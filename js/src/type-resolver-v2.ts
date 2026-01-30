import type { TypeDecl, Type } from 'sails-js-types-v2';

export const getScaleCodecDef = (type: TypeDecl, stringify = false) => {
  if (typeof type === "string") {
    if (type === "()") return "Null";
    if (type === "bool") return "bool";
    if (type === "char") return "char";
    if (type === "String") return "String";
    if (type === "i8") return "i8";
    if (type === "i16") return "i16";
    if (type === "i32") return "i32";
    if (type === "i64") return "i64";
    if (type === "i128") return "i128";
    if (type === "u8") return "u8";
    if (type === "u16") return "u16";
    if (type === "u32") return "u32";
    if (type === "u64") return "u64";
    if (type === "u128") return "u128";
    if (type === "ActorId" || type === "CodeId" || type === "MessageId") return "[u8;32]";
    if (type === "H256") return "H256";
    if (type === "H160") return "H160";
    if (type === "U256") return "U256";
  }
  if (type.kind === "slice") {
    return `Vec<${getScaleCodecDef(type.item)}>`;
  }
  if (type.kind === "array") {
    return `[${getScaleCodecDef(type.item)}; ${type.len}]`;
  }
  if (type.kind === "tuple") {
    return `(${type.types.map((t: TypeDecl) => getScaleCodecDef(t)).join(", ")})`;
  }
  if (type.kind === "named") {
    if (type.name === "Option") {
      return `Option<${getScaleCodecDef(type.generics[0])}>`;
    }
    if (type.name === "Result") {
      return `Result<${getScaleCodecDef(type.generics[0])}, ${getScaleCodecDef(type.generics[1])}>`;
    }
    if (type.name === "NonZeroU8") return "u8";
    if (type.name === "NonZeroU16") return "u16";
    if (type.name === "NonZeroU32") return "u32";
    if (type.name === "NonZeroU64") return "u64";
    if (type.name === "NonZeroU128") return "u128";
    if (type.name === "NonZeroU256") return "U256";
    if (type.generics?.length) {
      return `${type.name}<${type.generics.map((t: TypeDecl) => getScaleCodecDef(t)).join(", ")}>`;
    }
    return type.name;
  }
  throw new Error('Unknown type :: ' + JSON.stringify(type));
};

export const getScaleCodecTypeDef = (type: Type, stringify = false) => {
  if (type.kind === "struct") {
    return getStructDef(type.fields, stringify);
  }
  if (type.kind === "enum") {
    let isNesting = false;
    for (const variant of type.variants) {
      if (variant.fields.length > 0) {
        isNesting = true;
        break;
      }
    }
    if (!isNesting) {
      return { _enum: type.variants.map((v) => v.name) };
    }
    const result = {};
    for (const variant of type.variants) {
      result[variant.name] = getStructDef(variant.fields);
    }
    return { _enum: result };
  }
  throw new Error('Unknown type :: ' + JSON.stringify(type));
};

export const getStructDef = (fields: { name?: string; type: TypeDecl }[], stringify = false): object | string => {
  if (fields.length === 0) return "Null";
  let isTuple = true;
  for (const field of fields) {
    if (field.name) {
      isTuple = false;
      break;
    }
  }
  if (isTuple) {
    return `(${fields.map((f) => getScaleCodecDef(f.type)).join(", ")})`;
  }
  const result = {};
  for (const field of fields) {
    result[field.name] = getScaleCodecDef(field.type);
  }
  return stringify ? JSON.stringify(result) : result;
};