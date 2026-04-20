# Sails Type Registry

`sails-type-registry` provides portable type metadata for Sails.

It turns Rust type information into portable metadata that can be collected in
a deduplicated `Registry` and consumed by IDL generation and related tooling.
This crate is not a general-purpose reflection system.

## What It Provides

- `TypeInfo`: trait for exposing a Rust type as portable metadata.
- `Registry`: interns type descriptions and returns stable `TypeRef` handles.
- `MetaType`: lets you keep a reference to a `TypeInfo` type without making the
  surrounding struct generic. Useful when you need to store several different
  types in one place (for example, a list of service commands/queries/events).
- `TypeBuilder` / `CompositeBuilder` / `VariantDefBuilder`: builder API for
  constructing synthetic `Type` values.
- Types (`Type`, `TypeDecl`, `TypeDef`, …) are re-exported from
  [`sails-idl-ast`] — that crate is the canonical IDL AST and model.

## Two Forms: Declaration vs Definition

Every type has two portable forms, produced by two separate methods of the
`TypeInfo` trait:

```rust
pub trait TypeInfo: 'static {
    type Identity: ?Sized + 'static;

    /// Usage/reference form — returned for ALL types.
    /// Primitives return `TypeDecl::Primitive(_)`;
    /// custom types return `TypeDecl::Named { name, generics, .. }`.
    fn type_decl(registry: &mut Registry) -> TypeDecl;

    /// Full structural definition — only for user-defined named types
    /// (struct/enum). Primitives, sequences, tuples, etc. return `None`
    /// via the default implementation.
    fn type_def(_registry: &mut Registry) -> Option<Type> { None }
}
```

`TypeDecl` is the shape that appears at *use sites* (field types, function
parameters). `Type` (via `TypeDef::Struct` / `TypeDef::Enum` / `TypeDef::Alias`)
is the shape used at *definition sites*.

## Derive-Based Usage

Enable the `derive` feature (on by default) and derive `TypeInfo`:

```toml
[dependencies]
sails-type-registry = { version = "x.y.z", features = ["derive"] }
```

```rust
use sails_type_registry::{Registry, TypeInfo};

#[derive(TypeInfo)]
struct User {
    id: u64,
    name: String,
}

let mut registry = Registry::new();
let user_ref = registry.register_type::<User>();

// `get_type_decl` returns the usage form (TypeDecl::Named { name: "User", .. }).
let decl = registry.get_type_decl(user_ref).unwrap();

// `get_type` returns the full definition (struct fields / enum variants).
let def = registry.get_type(user_ref).unwrap();
assert_eq!(def.name, "User");
```

If the crate is re-exported under a different path, the derive can be pointed
there explicitly:

```rust
use sails_type_registry as registry;

#[derive(registry::TypeInfo)]
#[type_info(crate = registry)]
struct User {
    id: u64,
}
```

## Manual Registry Construction

For synthetic metadata, build `Type` values with `TypeBuilder` and register
them via the `TypeInfo` trait (no direct registration of pre-built `Type`
values — the registry is always driven by `TypeInfo`).

```rust
use sails_type_registry::{Registry, TypeBuilder, TypeDecl, TypeInfo};
use sails_type_registry::prelude::*;

struct Pair;

impl TypeInfo for Pair {
    type Identity = Self;

    fn type_decl(_registry: &mut Registry) -> TypeDecl {
        TypeDecl::named("Pair".into())
    }

    fn type_def(_registry: &mut Registry) -> Option<Type> {
        Some(
            TypeBuilder::new()
                .name("Pair")
                .composite()
                .field("left").ty(TypeDecl::Primitive(PrimitiveType::U32))
                .field("right").ty(TypeDecl::Primitive(PrimitiveType::U32))
                .build(),
        )
    }
}

let mut registry = Registry::new();
let pair_ref = registry.register_type::<Pair>();
assert!(registry.get_type(pair_ref).is_some());
```

Registration deduplicates by `TypeId::of::<T::Identity>()`, so multiple calls
for the same type return the same `TypeRef`. Recursive and mutually-recursive
definitions are handled via an internal placeholder pass.

## Iterating the Registry

- `registry.types()` — `(TypeRef, &TypeDecl)` for every registered type
  (usage form, includes primitives).
- `registry.named_types()` — `&Type` for every registered user-defined type
  (definition form, struct/enum/alias only).
- `registry.get_type_decl(type_ref)` — usage form for a specific ref.
- `registry.get_type(type_ref)` — definition form, `None` for non-user types.

## Metadata Model (from `sails-idl-ast`)

Each user-defined `Type` carries:

- `name` — human-readable type name.
- `type_params` — declared generic parameters (including const-generic ones).
- `def` — `TypeDef::Struct` / `TypeDef::Enum` / `TypeDef::Alias`.
- `docs` — documentation lines captured from `///` comments.
- `annotations` — custom `@key` / `@key("value")` metadata.

`TypeDecl` (usage form) covers:

- `Primitive(PrimitiveType)` — primitives (`bool`, integers, `String`, gear
  primitives, …).
- `Named { name, generics, param }` — references to user types and generic
  wrappers like `Option` / `Result` / `Range`.
- `Tuple { types }`, `Slice { item }`, `Array { item, len }` — structural
  forms without names.

### Module Path via `@path`

User-defined types record their originating Rust module path via a reserved
`@path` annotation. The constant `PATH_ANNOTATION` is exported for consumers
(IDL generation uses it to disambiguate identically-named types from
different modules). The annotation is stripped before emitting IDL.

## Features

- `derive` *(default)* — enables `#[derive(TypeInfo)]` via
  `sails-type-registry-derive`.
- `gprimitives` — adds `TypeInfo` for Gear primitives (`ActorId`, `MessageId`,
  `CodeId`, `H160`, `H256`, `U256`, `NonZeroU256`).
- `alloy-primitives` — adds `TypeInfo` for `alloy_primitives::Address` and
  `alloy_primitives::B256` (requires `gprimitives`).

[`sails-idl-ast`]: ../idl-ast
