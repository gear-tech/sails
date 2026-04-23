import { TypeRegistry } from '@polkadot/types/create';

import type { TypeDecl, Type, IStructField } from './types.js';

/**
 * Naming strategy for rendering a `TypeDecl` as a string.
 * - `generic`: human-readable generic syntax, e.g. `MyType<Option<u32>>`.
 * - `canonical`: registry-safe, punctuation-free name, e.g. `MyTypeOfOptionOfu32`.
 * - `field`: used when emitting struct/tuple field types; generally uses `canonical` names,
 *   but keeps known generic wrappers (`Option<>`, `Vec<>`, `Result<>`) in generic form,
 *   e.g. `field: Option<Vec<MyTypeOfOptionOfu32>>`.
 */
export type NameKind = 'generic' | 'field' | 'canonical';

export class TypeResolver {
  registry: TypeRegistry;
  private _userTypes: Record<string, Type> = {};

  /**
   * @param types service-local (or program-level) user types. Takes precedence over `ambientTypes` on name collision.
   * @param ambientTypes program-level types visible to every service. Shadowed by `types` when names collide.
   */
  constructor(types: Type[], ambientTypes: Type[] = []) {
    this.registry = new TypeRegistry();

    const scaleTypes: Record<string, any> = {};
    const userTypes: Record<string, Type> = {};
    for (const type of [...ambientTypes, ...types]) {
      userTypes[type.name] = type;
    }
    this._userTypes = userTypes;
    for (const type of Object.values(userTypes)) {
      if (!type.type_params?.length) {
        scaleTypes[type.name] = this.getTypeDef(type);
      }
    }
    this.registry.setKnownTypes({ types: scaleTypes });
    this.registry.register(scaleTypes);
  }

  /**
   * Resolve a `TypeDecl`'s named user type to its `Type` definition.
   *
   * Returns `undefined` for primitives, slices, arrays, tuples, unknown names, and bare
   * type parameters (a `{ kind: 'named', name: 'T' }` that isn't a registered user type).
   * Does not recurse into generics — callers that want the substituted inner shape should
   * pair this with {@link substituteGenerics}.
   *
   * The returned `Type` is shared with the resolver's internal state — do not mutate it.
   */
  resolveNamed(type: TypeDecl): Type | undefined {
    if (typeof type === 'string') return undefined;
    if (type.kind !== 'named') return undefined;
    return this._userTypes[type.name];
  }

  /**
   * Recursively substitute type parameters through a `TypeDecl` tree.
   *
   * Pure: does not mutate inputs. Idempotent: passing an already-substituted tree yields an
   * equivalent tree. Only bare `{ kind: 'named', name: 'T' }` leaves whose `name` appears in
   * `substitutions` are replaced; wrapper shapes (`Option<T>`, `Vec<T>`, `Result<T, E>`, custom
   * generics) are preserved and their inner types substituted in place.
   *
   * The function recurses through replacement chains (`{ T: U, U: u32 }` resolves `T` to `u32`).
   * Cyclic maps (`{ T: { kind: 'named', name: 'T' } }` or `{ T: U, U: T }`) are detected at
   * runtime and cause an error to be thrown rather than an unbounded recursion. Maps produced by
   * {@link genericsSubstitutions} from a parsed IDL cannot create cycles.
   */
  substituteGenerics(type: TypeDecl, substitutions: Record<string, TypeDecl> = {}): TypeDecl {
    return this._substituteGenerics(type, substitutions, new Set());
  }

  private _substituteGenerics(
    type: TypeDecl,
    substitutions: Record<string, TypeDecl>,
    visited: Set<string>,
  ): TypeDecl {
    if (typeof type === 'string') return type;
    if (type.kind === 'slice') {
      const item = this._substituteGenerics(type.item, substitutions, visited);
      return item === type.item ? type : { kind: 'slice', item };
    }
    if (type.kind === 'array') {
      const item = this._substituteGenerics(type.item, substitutions, visited);
      return item === type.item ? type : { kind: 'array', item, len: type.len };
    }
    if (type.kind === 'tuple') {
      const next = type.types.map((t) => this._substituteGenerics(t, substitutions, visited));
      return next.every((t, i) => t === type.types[i]) ? type : { kind: 'tuple', types: next };
    }
    if (type.kind === 'named') {
      if (type.generics?.length) {
        const next = type.generics.map((g) => this._substituteGenerics(g, substitutions, visited));
        return next.every((g, i) => g === type.generics![i])
          ? type
          : { kind: 'named', name: type.name, generics: next };
      }
      // Bare named reference may be a type parameter. Track visited names so a cyclic map
      // (`{ T: T }`, `{ T: U, U: T }`) throws instead of stack-overflowing.
      const replacement = substitutions[type.name];
      if (replacement !== undefined) {
        if (visited.has(type.name)) {
          throw new Error(
            `Cyclic substitution detected while resolving type parameter "${type.name}" — ` +
              `substitution chain: ${[...visited, type.name].join(' → ')}`,
          );
        }
        const nextVisited = new Set(visited);
        nextVisited.add(type.name);
        return this._substituteGenerics(replacement, substitutions, nextVisited);
      }
      return type;
    }
    throw new Error('Unknown TypeDecl kind :: ' + JSON.stringify(type));
  }

  /**
   * Build a substitution map from a user type's declared `type_params` and a concrete
   * generics list (typically the `generics` field of a `{ kind: 'named', generics: [...] }`
   * `TypeDecl`). Missing positions are omitted.
   */
  genericsSubstitutions(userType: Type, generics: TypeDecl[] = []): Record<string, TypeDecl> {
    const map: Record<string, TypeDecl> = {};
    const params = userType.type_params ?? [];
    const len = Math.min(params.length, generics.length);
    for (let i = 0; i < len; i++) {
      map[params[i].name] = generics[i];
    }
    return map;
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
  getTypeDeclString(type: TypeDecl, generics: Record<string, TypeDecl> = {}, nameKind: NameKind = 'generic'): string {
    if (typeof type === 'string') {
      if (type === '()') return 'Null';
      if (type === 'bool') return 'bool';
      if (type === 'char') return 'char';
      if (type === 'String') return 'String';
      if (type === 'i8') return 'i8';
      if (type === 'i16') return 'i16';
      if (type === 'i32') return 'i32';
      if (type === 'i64') return 'i64';
      if (type === 'i128') return 'i128';
      if (type === 'u8') return 'u8';
      if (type === 'u16') return 'u16';
      if (type === 'u32') return 'u32';
      if (type === 'u64') return 'u64';
      if (type === 'u128') return 'u128';
      if (type === 'ActorId' || type === 'CodeId' || type === 'MessageId') {
        if (nameKind === 'canonical') {
          return type as string;
        }
        return '[u8;32]';
      }
      if (type === 'H256') return 'H256';
      if (type === 'H160') return 'H160';
      if (type === 'U256') return 'U256';
    }
    if (type.kind === 'slice') {
      if (nameKind === 'canonical') {
        return `VecOf${this.getTypeDeclString(type.item, generics, nameKind)}`;
      }
      return `Vec<${this.getTypeDeclString(type.item, generics, nameKind)}>`;
    }
    if (type.kind === 'array') {
      if (nameKind === 'canonical') {
        return `ArrayOf${this.getTypeDeclString(type.item, generics, nameKind)}Len${type.len}`;
      }
      return `[${this.getTypeDeclString(type.item, generics, nameKind)};${type.len}]`;
    }
    if (type.kind === 'tuple') {
      if (nameKind === 'canonical') {
        return `TupleOf${type.types.map((t: TypeDecl) => this.getTypeDeclString(t, generics, nameKind)).join('And')}`;
      }
      return `(${type.types.map((t: TypeDecl) => this.getTypeDeclString(t, generics, nameKind)).join(',')})`;
    }
    if (type.kind === 'generic') {
      const generic = generics[type.name];
      if (generic) {
        return this.getTypeDeclString(generic, generics, nameKind);
      }

      return type.name;
    }
    if (type.kind === 'named') {
      if (type.name === 'Option') {
        if (nameKind === 'canonical') {
          return `OptionOf${this.getTypeDeclString(type.generics[0], generics, nameKind)}`;
        }
        return `Option<${this.getTypeDeclString(type.generics[0], generics, nameKind)}>`;
      }
      if (type.name === 'Result') {
        if (nameKind === 'canonical') {
          return `ResultOk${this.getTypeDeclString(type.generics[0], generics, nameKind)}Err${this.getTypeDeclString(type.generics[1], generics, nameKind)}`;
        }
        return `Result<${this.getTypeDeclString(type.generics[0], generics, nameKind)},${this.getTypeDeclString(type.generics[1], generics, nameKind)}>`;
      }
      if (type.name === 'NonZeroU8') return 'u8';
      if (type.name === 'NonZeroU16') return 'u16';
      if (type.name === 'NonZeroU32') return 'u32';
      if (type.name === 'NonZeroU64') return 'u64';
      if (type.name === 'NonZeroU128') return 'u128';
      if (type.name === 'NonZeroU256') return 'U256';
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
          .map((t: TypeDecl) => this.getTypeDeclString(t, generics, 'generic'))
          .join(',')}>`;
        // canonicalName with resolved generics, i.e. `MyTypeOfStringOptionOfu8>`
        const canonicalName = `${type.name}Of${type.generics
          .map((t: TypeDecl) => this.getTypeDeclString(t, generics, 'canonical'))
          .join('')}`;
        if (!this.registry.hasType(canonicalName)) {
          const generics_map = this.genericsSubstitutions(userType, type.generics);
          const typeDef = this.getTypeDef(userType, generics_map);
          /// When a user type with generics is resolved, the resolver constructs two names:
          // - genericName: readable, type-like syntax (example MyType<Option<u32>>).
          // - canonicalName: registry-safe, punctuation-free string (example MyTypeOfOptionOfu32).
          // The registry is keyed by string name. Field/tuple contexts use nameKind: "field",
          // which is treated like canonical to avoid <, >, and , in field type strings.
          // To ensure both references resolve to the same underlying definition,
          // the code registers the same typeDef under both genericName and canonicalName.
          const scaleTypes: Record<string, any> = { [genericName]: typeDef, [canonicalName]: typeDef };
          this.registry.register(scaleTypes);
        }
        return nameKind == 'generic' ? genericName : canonicalName;
      }

      // Non-generic resolve as registered type
      return type.name;
    }
    throw new Error('Unknown type :: ' + JSON.stringify(type));
  }

  getTypeDef(type: Type, generics: Record<string, TypeDecl> = {}): object | string {
    if (type.kind === 'struct') {
      return this.getStructDef(type.fields, generics);
    }
    if (type.kind === 'enum') {
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
    if (type.kind === 'alias') {
      return this.getTypeDeclString(type.target, generics, 'field');
    }
    throw new Error('Unknown type :: ' + JSON.stringify(type));
  }

  getStructDef(
    fields: { name?: string; type: TypeDecl }[],
    generics: Record<string, TypeDecl> = {},
    stringify = false,
  ): object | string {
    if (fields.length === 0) return 'Null';
    let isTuple = true;
    for (const field of fields) {
      if (field.name) {
        isTuple = false;
        break;
      }
    }
    if (isTuple) {
      if (fields.length === 1) {
        return this.getTypeDeclString(fields[0].type, generics, 'field');
      }
      return `(${fields.map((f: IStructField) => this.getTypeDeclString(f.type, generics, 'field')).join(',')})`;
    }
    const result = {};
    for (const field of fields) {
      result[field.name] = this.getTypeDeclString(field.type, generics, 'field');
    }
    return stringify ? JSON.stringify(result) : result;
  }
}
