import { TypeRegistry } from '@polkadot/types/create';
import type { TypeDecl, Type, IStructField } from 'sails-js-types-v2';

export class TypeResolver {
  registry: TypeRegistry;
  private _userTypes: Record<string, Type> = {};

  constructor(types: Type[]) {
    const scaleTypes: Record<string, any> = {};
    const userTypes: Record<string, Type> = {};
    for (const type of types) {
      userTypes[type.name] = type;
      if (!type.type_params?.length) {
        // register non-generic by name
        scaleTypes[type.name] = this.getTypeDef(type);
      }
    }
    this._userTypes = userTypes;

    this.registry = new TypeRegistry();
    this.registry.setKnownTypes({ types: scaleTypes });
    this.registry.register(scaleTypes);
  }

  getTypeDeclString(type: TypeDecl, generics: Record<string, TypeDecl> = {}): string {
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
      // Generic param
      let generic = generics[type];
      if (generic) return this.getTypeDeclString(generic, generics);
    }
    if (type.kind === "slice") {
      return `Vec<${this.getTypeDeclString(type.item, generics)}>`;
    }
    if (type.kind === "array") {
      return `[${this.getTypeDeclString(type.item, generics)}; ${type.len}]`;
    }
    if (type.kind === "tuple") {
      return `(${type.types.map((t: TypeDecl) => this.getTypeDeclString(t, generics)).join(", ")})`;
    }
    if (type.kind === "named") {
      if (type.name === "Option") {
        return `Option<${this.getTypeDeclString(type.generics[0], generics)}>`;
      }
      if (type.name === "Result") {
        return `Result<${this.getTypeDeclString(type.generics[0], generics)}, ${this.getTypeDeclString(type.generics[1], generics)}>`;
      }
      if (type.name === "BTreeMap") {
        return `BTreeMap<${this.getTypeDeclString(type.generics[0], generics)}, ${this.getTypeDeclString(type.generics[1], generics)}>`;
      }
      if (type.name === "NonZeroU8") return "u8";
      if (type.name === "NonZeroU16") return "u16";
      if (type.name === "NonZeroU32") return "u32";
      if (type.name === "NonZeroU64") return "u64";
      if (type.name === "NonZeroU128") return "u128";
      if (type.name === "NonZeroU256") return "U256";
      if (type.generics?.length) {
        const userType = this._userTypes[type.name];
        if (!userType) {
          throw new Error('Unknown type :: ' + JSON.stringify(type));
        }
        if (!userType?.type_params?.length || userType.type_params.length !== type.generics.length) {
          throw new Error('Unknown generic type :: ' + JSON.stringify(type));
        }
        // type name with resolved generics, i.e. `MyType<String,Option<u8>>`
        const typeName = `${type.name}<${type.generics.map((t: TypeDecl) => this.getTypeDeclString(t)).join(",")}>`;
        // generics map, i.e, { "T": "String", "U": { "kind": "named", "name": "Option", "generics": ["u32"]} }
        const generics: Record<string, TypeDecl> = {};
        for (let i = 0; i < userType.type_params.length; i++) {
          generics[userType.type_params[i].name] = type.generics[i];
        }
        const scaleTypes: Record<string, any> = {};
        scaleTypes[typeName] = this.getTypeDef(userType, generics);
        // register resolved generyc type
        console.log(scaleTypes);
        this.registry.register(scaleTypes);
        return typeName;
      }
      // Generic param
      let generic = generics[type.name];
      if (generic) return this.getTypeDeclString(generic, generics);
      // Non-generic resolve as registered type
      return type.name;
    }
    throw new Error('Unknown type :: ' + JSON.stringify(type));
  };

  getTypeDef(type: Type, generics: Record<string, TypeDecl> = {}): object | string {
    if (type.kind === "struct") {
      return this.getStructDef(type.fields, generics);
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
        result[variant.name] = this.getStructDef(variant.fields, generics);
      }
      return { _enum: result };
    }
    throw new Error('Unknown type :: ' + JSON.stringify(type));
  };

  getStructDef(fields: { name?: string; type: TypeDecl }[], generics: Record<string, TypeDecl> = {}, stringify = false): object | string {
    if (fields.length === 0) return "Null";
    let isTuple = true;
    for (const field of fields) {
      if (field.name) {
        isTuple = false;
        break;
      }
    }
    if (isTuple) {
      if (fields.length === 1) {
        return this.getTypeDeclString(fields[0].type, generics)
      }
      return `(${fields.map((f: IStructField) => this.getTypeDeclString(f.type, generics)).join(", ")})`;
    }
    const result = {};
    for (const field of fields) {
      result[field.name] = this.getTypeDeclString(field.type, generics);
    }
    return stringify ? JSON.stringify(result) : result;
  };
}

// export const getTypeDeclDef = (type: TypeDecl, userTypes: Record<string, Type> = {}) => {
//   if (typeof type === "string") {
//     if (type === "()") return "Null";
//     if (type === "bool") return "bool";
//     if (type === "char") return "char";
//     if (type === "String") return "String";
//     if (type === "i8") return "i8";
//     if (type === "i16") return "i16";
//     if (type === "i32") return "i32";
//     if (type === "i64") return "i64";
//     if (type === "i128") return "i128";
//     if (type === "u8") return "u8";
//     if (type === "u16") return "u16";
//     if (type === "u32") return "u32";
//     if (type === "u64") return "u64";
//     if (type === "u128") return "u128";
//     if (type === "ActorId" || type === "CodeId" || type === "MessageId") return "[u8;32]";
//     if (type === "H256") return "H256";
//     if (type === "H160") return "H160";
//     if (type === "U256") return "U256";
//   }
//   if (type.kind === "slice") {
//     return `Vec<${getTypeDeclDef(type.item, userTypes)}>`;
//   }
//   if (type.kind === "array") {
//     return `[${getTypeDeclDef(type.item, userTypes)}; ${type.len}]`;
//   }
//   if (type.kind === "tuple") {
//     return `(${type.types.map((t: TypeDecl) => getTypeDeclDef(t, userTypes)).join(", ")})`;
//   }
//   if (type.kind === "named") {
//     if (type.name === "Option") {
//       return `Option<${getTypeDeclDef(type.generics[0], userTypes)}>`;
//     }
//     if (type.name === "Result") {
//       return `Result<${getTypeDeclDef(type.generics[0], userTypes)}, ${getTypeDeclDef(type.generics[1], userTypes)}>`;
//     }
//     if (type.name === "NonZeroU8") return "u8";
//     if (type.name === "NonZeroU16") return "u16";
//     if (type.name === "NonZeroU32") return "u32";
//     if (type.name === "NonZeroU64") return "u64";
//     if (type.name === "NonZeroU128") return "u128";
//     if (type.name === "NonZeroU256") return "U256";
//     if (type.generics?.length) {
//       const userType = userTypes[type.name];
//       if (userType?.type_params?.length && userType.type_params.length === type.generics.length) {
//         const typeParamMap: Record<string, TypeDecl> = {};
//         for (let i = 0; i < userType.type_params.length; i++) {
//           typeParamMap[userType.type_params[i].name] = type.generics[i];
//         }

//         const resolveTypeParams = (decl: TypeDecl): TypeDecl => {
//           if (typeof decl === "string") {
//             return typeParamMap[decl] ?? decl;
//           }
//           if (decl.kind === "slice") {
//             return { kind: "slice", item: resolveTypeParams(decl.item) };
//           }
//           if (decl.kind === "array") {
//             return { kind: "array", item: resolveTypeParams(decl.item), len: decl.len };
//           }
//           if (decl.kind === "tuple") {
//             return { kind: "tuple", types: decl.types.map(resolveTypeParams) };
//           }
//           if (decl.kind === "named") {
//             const mapped = typeParamMap[decl.name];
//             if (mapped && !decl.generics?.length) {
//               return mapped;
//             }
//             return {
//               kind: "named",
//               name: decl.name,
//               generics: decl.generics?.map(resolveTypeParams),
//             };
//           }
//           return decl;
//         };

//         if (userType.kind === "struct") {
//           const fields = userType.fields.map((field) => ({
//             ...field,
//             type: resolveTypeParams(field.type),
//           }));
//           if (fields.length === 0) return "Null";
//           let isTuple = true;
//           for (const field of fields) {
//             if (field.name) {
//               isTuple = false;
//               break;
//             }
//           }
//           if (isTuple) {
//             return `(${fields.map((f: IStructField) => getTypeDeclDef(f.type, userTypes)).join(", ")})`;
//           }
//           const result = {};
//           for (const field of fields) {
//             result[field.name] = getTypeDeclDef(field.type, userTypes);
//           }
//           return JSON.stringify(result);
//         }

//         if (userType.kind === "enum") {
//           let isNesting = false;
//           for (const variant of userType.variants) {
//             if (variant.fields.length > 0) {
//               isNesting = true;
//               break;
//             }
//           }
//           if (!isNesting) {
//             return JSON.stringify({ _enum: userType.variants.map((v) => v.name) });
//           }
//           const result = {};
//           for (const variant of userType.variants) {
//             const fields = variant.fields.map((field) => ({
//               ...field,
//               type: resolveTypeParams(field.type),
//             }));
//             if (fields.length === 0) {
//               result[variant.name] = "Null";
//               continue;
//             }
//             let isTuple = true;
//             for (const field of fields) {
//               if (field.name) {
//                 isTuple = false;
//                 break;
//               }
//             }
//             if (isTuple) {
//               result[variant.name] = `(${fields.map((f: IStructField) => getTypeDeclDef(f.type, userTypes)).join(", ")})`;
//               continue;
//             }
//             const structResult = {};
//             for (const field of fields) {
//               structResult[field.name] = getTypeDeclDef(field.type, userTypes);
//             }
//             result[variant.name] = structResult;
//           }
//           return JSON.stringify({ _enum: result });
//         }
//       }

//       return `${type.name}<${type.generics.map((t: TypeDecl) => getTypeDeclDef(t, userTypes)).join(", ")}>`;
//     }
//     // Non-generic resolve as registered type
//     return type.name;
//   }
//   throw new Error('Unknown type :: ' + JSON.stringify(type));
// };

// export const getTypeDef = (type: Type, stringify = false) => {
//   if (type.kind === "struct") {
//     return getStructDef(type.fields, stringify);
//   }
//   if (type.kind === "enum") {
//     let isNesting = false;
//     for (const variant of type.variants) {
//       if (variant.fields.length > 0) {
//         isNesting = true;
//         break;
//       }
//     }
//     if (!isNesting) {
//       return { _enum: type.variants.map((v) => v.name) };
//     }
//     const result = {};
//     for (const variant of type.variants) {
//       result[variant.name] = getStructDef(variant.fields);
//     }
//     return { _enum: result };
//   }
//   throw new Error('Unknown type :: ' + JSON.stringify(type));
// };

// export const getStructDef = (fields: { name?: string; type: TypeDecl }[], stringify = false): object | string => {
//   if (fields.length === 0) return "Null";
//   let isTuple = true;
//   for (const field of fields) {
//     if (field.name) {
//       isTuple = false;
//       break;
//     }
//   }
//   if (isTuple) {
//     return `(${fields.map((f: IStructField) => getTypeDeclDef(f.type)).join(", ")})`;
//   }
//   const result = {};
//   for (const field of fields) {
//     result[field.name] = getTypeDeclDef(field.type);
//   }
//   return stringify ? JSON.stringify(result) : result;
// };
