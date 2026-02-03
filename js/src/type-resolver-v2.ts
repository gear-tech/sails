import { TypeRegistry } from '@polkadot/types/create';
import type { TypeDecl, Type, IStructField } from 'sails-js-types-v2';

/**
 * Naming strategy for rendering a `TypeDecl` as a string.
 * - `generic`: human-readable generic syntax, e.g. `MyType<Option<u32>>`.
 * - `canonical`: registry-safe, punctuation-free name, e.g. `MyTypeOfOptionOfu32`.
 * - `field`: used when emitting struct/tuple field types; generally uses `canonical` names,
 *   but keeps known generic wrappers (`Option<>`, `Vec<>`, `Result<>`) in generic form,
 *   e.g. `field: Option<Vec<MyTypeOfOptionOfu32>>`.
 */
export type NameKind = "generic" | "field" | "canonical";

export class TypeResolver {
  registry: TypeRegistry;
  private _userTypes: Record<string, Type> = {};

  constructor(types: Type[]) {
    this.registry = new TypeRegistry();

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
    this.registry.setKnownTypes({ types: scaleTypes });
    this.registry.register(scaleTypes);
  }

  /**
   * Convert a `TypeDecl` into a concrete string name, resolving generic parameters.
   *
   * When a parameterized user type is encountered, the fully-resolved definition is
   * registered under both its `generic` and `canonical` names so lookups by either
   * representation succeed.
   *
   * For `nameKind: "field"`, known generic wrappers keep the generic syntax
   * (`Option<>`, `Vec<>`, `Result<>`) while their inner types are rendered canonically.
   */
  getTypeDeclString(type: TypeDecl, generics: Record<string, TypeDecl> = {}, nameKind: NameKind = "generic"): string {
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
      if (type === "ActorId" || type === "CodeId" || type === "MessageId") {
        if (nameKind === "canonical") {
          return type as string;
        }
        return "[u8;32]";
      }
      if (type === "H256") return "H256";
      if (type === "H160") return "H160";
      if (type === "U256") return "U256";
    }
    if (type.kind === "slice") {
      if (nameKind === "canonical") {
        return `VecOf${this.getTypeDeclString(type.item, generics, nameKind)}`;
      }
      return `Vec<${this.getTypeDeclString(type.item, generics, nameKind)}>`;
    }
    if (type.kind === "array") {
      if (nameKind === "canonical") {
        return `ArrayOf${this.getTypeDeclString(type.item, generics, nameKind)}Len${type.len}`;
      }
      return `[${this.getTypeDeclString(type.item, generics, nameKind)};${type.len}]`;
    }
    if (type.kind === "tuple") {
      if (nameKind === "canonical") {
        return `TupleOf${type.types
          .map((t: TypeDecl) => this.getTypeDeclString(t, generics, nameKind))
          .join("And")}`;
      }
      return `(${type.types.map((t: TypeDecl) => this.getTypeDeclString(t, generics, nameKind)).join(",")})`;
    }
    if (type.kind === "named") {
      if (type.name === "Option") {
        if (nameKind === "canonical") {
          return `OptionOf${this.getTypeDeclString(type.generics[0], generics, nameKind)}`;
        }
        return `Option<${this.getTypeDeclString(type.generics[0], generics, nameKind)}>`;
      }
      if (type.name === "Result") {
        if (nameKind === "canonical") {
          return `ResultOk${this.getTypeDeclString(type.generics[0], generics, nameKind)}Err${this.getTypeDeclString(type.generics[1], generics, nameKind)}`;
        }
        return `Result<${this.getTypeDeclString(type.generics[0], generics, nameKind)},${this.getTypeDeclString(type.generics[1], generics, nameKind)}>`;
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
        // genericName with resolved generics, i.e. `MyType<String,Option<u8>>`
        const genericName = `${type.name}<${type.generics
          .map((t: TypeDecl) => this.getTypeDeclString(t, generics, "generic"))
          .join(",")}>`;
        // canonicalName with resolved generics, i.e. `MyTypeOfStringOptionOfu8>`
        const canonicalName = `${type.name}Of${type.generics
          .map((t: TypeDecl) => this.getTypeDeclString(t, generics, "canonical"))
          .join("")}`;
        if (!this.registry.hasType(canonicalName)) {
          // type param to generic map, i.e, { "T": "String", "U": { "kind": "named", "name": "Option", "generics": ["u32"]} }
          const generics_map: Record<string, TypeDecl> = {};
          for (let i = 0; i < userType.type_params.length; i++) {
            generics_map[userType.type_params[i].name] = type.generics[i];
          }
          const typeDef = this.getTypeDef(userType, generics_map);
          /// When a user type with generics is resolved, the resolver constructs two names:
          // - genericName: readable, type-like syntax (example MyType<Option<u32>>).
          // - canonicalName: registry-safe, punctuation-free string (example MyTypeOfOptionOfu32).
          // The registry is keyed by string name. Field/tuple contexts use nameKind: "field",
          // which is treated like canonical to avoid <, >, and , in field type strings.
          // To ensure both references resolve to the same underlying definition,
          // the code registers the same typeDef under both genericName and canonicalName.
          const scaleTypes: Record<string, any> = { [genericName]: typeDef, [canonicalName]: typeDef, };
          this.registry.register(scaleTypes);
        }
        return nameKind == "generic" ? genericName : canonicalName;
      }
      // Generic param
      const generic = generics[type.name];
      if (generic) {
        return this.getTypeDeclString(generic, generics, nameKind);
      }

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
        return this.getTypeDeclString(fields[0].type, generics, "field");
      }
      return `(${fields.map((f: IStructField) => this.getTypeDeclString(f.type, generics, "field")).join(",")})`;
    }
    const result = {};
    for (const field of fields) {
      result[field.name] = this.getTypeDeclString(field.type, generics, "field");
    }
    return stringify ? JSON.stringify(result) : result;
  };
}
