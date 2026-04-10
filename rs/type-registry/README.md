# Sails Type Registry

`sails-type-registry` provides portable type metadata for Sails.

It turns Rust type information into portable metadata that can be collected in
a deduplicated `Registry` and consumed by IDL generation and related tooling.
This crate is not a general-purpose reflection system.

## What It Provides

- `TypeInfo`: trait for exposing a Rust type as portable metadata.
- `Registry`: registry that interns type descriptions and returns stable
  `TypeRef` handles.
- `Type` / `TypeDef`: portable metadata model for primitives, composites,
  variants, collections, generic parameters, and applied generic types.
- Builder API: manual construction of `Type` values for synthetic definitions.

## Derive-Based Usage

Enable the `derive` feature and derive `TypeInfo` on your types:

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
let user_ty = registry.get_type(user_ref).unwrap();

assert_eq!(user_ty.name, "User");
```

If the crate is re-exported under another name, the derive can be pointed at
that path explicitly:

```rust
use sails_type_registry as registry;

#[derive(registry::TypeInfo)]
#[type_info(crate = registry)]
struct User {
    id: u64,
}
```

## Manual Registry Construction

For synthetic or manually assembled metadata, construct `Type` values with the
builder API and register them directly:

```rust
use sails_type_registry::{Registry, Type};

let mut registry = Registry::new();
let u32_ref = registry.register_type::<u32>();

let pair = Type::builder()
    .name("Pair")
    .composite()
    .field("left")
    .ty(u32_ref)
    .field("right")
    .ty(u32_ref)
    .build();

let pair_ref = registry.register_type_def(pair);
assert!(registry.get_type(pair_ref).is_some());
```

`register_type_def` normalizes definitions before insertion and deduplicates
them by module path, name, type parameters, and normalized definition.

## Metadata Model

Each registry entry stores:

- `module_path`: recorded module path for display and disambiguation.
- `name`: human-readable type name.
- `type_params`: declared generic parameters and their assigned arguments.
- `def`: structural type definition.
- `docs`: captured documentation lines.
- `annotations`: captured custom metadata annotations.

`TypeDef` covers:

- primitives
- composites and variants
- sequences, arrays, tuples, and maps
- `Option` and `Result`
- generic parameters
- applied generic types

Aliases are not represented as a separate metadata kind. The registry stores
portable type descriptions, not source-level alias declarations.

## Features

- `derive`: enables `#[derive(TypeInfo)]` via `sails-type-registry-derive`
- `gprimitives`: adds Gear-specific primitive support from `gprimitives`
