# Sails Type Registry

`sails-type-registry` provides portable type metadata for Sails.

It turns Rust type information into portable metadata that can be collected in
a deduplicated `Registry` and consumed by IDL generation and related tooling.
This crate is not a general-purpose reflection system.

## What It Provides

- `TypeInfo`: trait for exposing a Rust type as portable metadata.
- `Registry`: nominal-type interner plus concrete binding cache. It stores
  shared nominal definitions and records concrete generic arguments at use
  sites.
- `sails_idl_ast`: portable metadata model for primitives, composites,
  variants, collections, generic parameters, and applied generic types.
- Builder API: manual construction of nominal `Type` values for explicit
  `TypeInfo` implementations.

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
let user_ref = registry.register_type::<User>().expect("User is nominal");
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

## Manual TypeInfo Implementations

For synthetic or manually assembled metadata, implement `TypeInfo` and return a
nominal `Type` from `type_def`. The registry owns interning and unique-name
disambiguation.

```rust
use sails_type_registry::ast::{PrimitiveType, Type, TypeDecl};
use sails_type_registry::{Registry, TypeBuilder, TypeInfo};

struct Pair;

impl TypeInfo for Pair {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        registry.register_named_type(Self::META, "Pair".into(), Vec::new(), |_| {})
    }

    fn type_def(_registry: &mut Registry) -> Option<Type> {
        Some(
            TypeBuilder::new()
                .name("Pair")
                .composite()
                .field("left")
                .ty(TypeDecl::Primitive(PrimitiveType::U32))
                .field("right")
                .ty(TypeDecl::Primitive(PrimitiveType::U32))
                .build(),
        )
    }
}

let mut registry = Registry::new();
let pair_ref = registry.register_type::<Pair>().expect("Pair is nominal");
assert!(registry.get_type(pair_ref).is_some());
```

## Metadata Model

Each registry entry stores:

- `name`: human-readable type name.
- `type_params`: declared generic parameters and optional defaults.
- `def`: structural type definition.
- `docs`: captured documentation lines.
- `annotations`: captured custom metadata annotations.

Concrete bindings store per-instantiation generic arguments separately from the
shared nominal definition.

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
