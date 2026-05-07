# Sails Type Registry

`sails-type-registry` provides portable type metadata for Sails.

It turns Rust type information into portable metadata that can be collected in
a deduplicated `Registry` and consumed by IDL generation and related tooling.
This crate is not a general-purpose reflection system.

## What It Provides

- `TypeInfo`: trait for exposing a Rust type as portable metadata.
- `Registry`: Named-type interner plus concrete binding cache. It stores
  shared named definitions and records concrete generic arguments at use
  sites.
- `sails_idl_ast`: portable metadata model for primitives, composites,
  variants, collections, generic parameters, and applied generic types.
- Builder API: manual construction of named `Type` values for explicit
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
let user_ref = registry.register_type::<User>().expect("User is named");
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
named `Type` from `type_def`. The registry owns interning and unique-name
disambiguation.

```rust
use sails_type_registry::ast::{PrimitiveType, Type, TypeDecl};
use sails_type_registry::{Registry, TypeBuilder, TypeInfo};

struct Pair;

impl TypeInfo for Pair {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        registry.register_named_type(Self::META, "Pair", Vec::new())
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
let pair_ref = registry.register_type::<Pair>().expect("Pair is named");
assert!(registry.get_type(pair_ref).is_some());
```

When a manual named type needs to register field dependencies after caching
its own concrete `TypeId`, use `register_named_type_with_dependencies`:

```rust
fn type_decl(registry: &mut Registry) -> TypeDecl {
    registry.register_named_type_with_dependencies(
        Self::META,
        "Node",
        Vec::new(),
        |registry| {
            let _ = <Box<Node> as TypeInfo>::type_decl(registry);
        },
    )
}
```

Most hand-written implementations should use `register_named_type`; the
dependency-aware form is primarily for derive-generated recursive-safe code.

## Metadata Model

Each registry entry stores:

- `name`: human-readable type name.
- `type_params`: declared generic parameters and optional defaults.
- `def`: structural type definition.
- `docs`: captured documentation lines.
- `annotations`: captured custom metadata annotations.

Concrete bindings store per-instantiation generic arguments separately from the
shared named definition.

For a generic named type, the stored definition stays abstract and field
references to declared generic parameters are represented as
`TypeDecl::Generic { name }`. Concrete use sites carry the applied generic
arguments:

```rust
// Stored named definition field:
TypeDecl::Generic { name: "T".into() }

// Concrete use site:
TypeDecl::Named {
    name: "Wrapper".into(),
    generics: vec![TypeDecl::Primitive(PrimitiveType::String)],
}
```

This lets `Wrapper<String>` and `Wrapper<u32>` share one named `Type`
definition while keeping concrete arguments available for IDL export.

The metadata model covers:

- primitives
- composites and variants
- sequences, arrays, tuples, and maps
- `Option` and `Result`
- generic parameter references
- applied generic types

Aliases are not represented as a separate metadata kind. The registry stores
portable type descriptions, not source-level alias declarations.

## Registry Naming

The derive macro owns const-generic suffixes in base names, for example
`Wrapper<T, const N: usize>` can register as `WrapperN32` for `N = 32`.
The registry then handles only collision disambiguation between named types
with the same base name from different module paths. It first tries the base
name, then prepends module path segments in PascalCase, and finally appends a
numeric suffix if needed.

## Features

- `derive`: enables `#[derive(TypeInfo)]` via `sails-type-registry-derive`
- `gprimitives`: adds Gear-specific primitive support from `gprimitives`
