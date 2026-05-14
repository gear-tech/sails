# IDL V2 Spec

## Document Structure

A Sails IDL document is a sequence of top-level items in any order:

- Global annotations (`!@<ident>[: <value>]`), including `!@include` directives.
- Zero or more `service <ident>[@<interface_id>] { ... }` declarations.
- At most one `program <ident> { ... }` declaration.

Whitespace and `//` line comments are ignored. The preprocessor expands `!@include` directives before grammar parsing (see [Include directive](#include-directive) below).

Validation rules enforced by the parser:

- Service `ident`s must be unique within the document.
- Service `interface_id`s must be unique across all services in the document.

## Annotations

### Global

String represented value, starting from the beginning of file

- `!@<ident>[: <value>]`

Examples

- `!@sails: 0.1.0` // version of IDL
- `!@author: me` // author of the IDL
- `!@git: ...` // source code
- `!@version: 0.2.0` // protocol version

### Include directive

The `!@include` global annotation splices another IDL document into the current one at parse time.

- Syntax: `!@include: <path_to_idl>`
- Path may be a local filesystem path (resolved by `FsLoader`) or a `git://` URL.
- Included content is inlined at the directive site; the included file may itself contain further `!@include` directives.

### Local

Local annotations goes before (above) related token.

- `@<ident>[: <value>]`

Examples

- `@doc: some comments` (shortcut: `///`)
- `@indexed`
- `@query`
- `@partial` (used for service subset generation)
- `@entry_id: <number>` (explicit entry identifier)

## Comments

- Comments start from `//` and end where the line ends.
- Used for some IDL clarifications or details.
- Ignored on the IDL parsing.

## Types

### Common

- `bool`
- `char`
- `string`, `String`

### Numerics

- `u8`, `i8`
- `u16`, `i16`
- `u32`, `i32`
- `u64`, `i64`
- `u128`, `i128`
- `u256`, `U256` — 256-bit unsigned integer (`struct U256([u64; 4])`)

### Identifiers & Hashes

Built-in primitive types backed by fixed-size byte arrays. Both the short lowercase form and the PascalCase form are accepted.

- `actor`, `ActorId` — `struct ActorId([u8; 32])`
- `code`, `CodeId` — `struct CodeId([u8; 32])`
- `messageid`, `MessageId` — `struct MessageId([u8; 32])`
- `h160`, `H160` — `struct H160([u8; 20])`
- `h256`, `H256` — `struct H256([u8; 32])`

### Slice

`[T]` Dynamic-length array (no Rusty `vec`)

- `[u8]`

### Array

`[T; u32]` Fixed-length array

- `[u8; 16]`

### Tuples

- `(T1, T2, ..)` Type tuple
  - `(u8, u16)`
  - `(u8, u16, u32)`
- `()` — the unit type. Conventionally the `-> <output>` clause is omitted for functions with no return value rather than written as `-> ()`.
  May appear anywhere a type is expected (parameters, fields, generic arguments).

### Structs

- Classic (named fields)

```rs
struct <ident> {
    <field>: <type>,
}
```

Example

```rs
    struct Type {
        f1: u64,
        f2: Type2,
    }
```

- Tuple-like (unnamed fields)

```rs
struct <ident>(<type>[,<type>]*)
```

Example

```rs
    struct Type(Type1, Type2);
```

- Unit-like (no fields)
  - `struct Type;`

### Enums (Unions)

Unified declaration (similar to Rust):

```rs
enum Type {
    Var1 {          // Struct-like
        f1: Type1,
        f2: Type2,
    },
    Var2,           // Unit-like
    Var3(Type3),    // Tuple-like
}
```

### Generics

Generic type definition

#### Generic struct

- Classic (named fields)

```rs
struct Type<T1, T2> {
    f1: T1,
    f2: T2,
}
```

- Tuple-like (unnamed fields)

```rs
struct Type<T>(Type1, T);
```

#### Generic enums

```rs
enum Type<T> {
    Var1 {
        f1: T,
    },
    Var2,
    Var3(T),
}
```

### Type Aliases

`alias <ident> = <type>;`

#### Generic support

- `alias NewType = Type<u8>;`
- `alias NewType<T> = Type<T>;`

#### Convenience aliases (TBD)

Short aliases for common patterns. Not yet built into the parser. Declare them in a `types` block if needed.

- `alias void = ();`
- `alias list<T> = [T];`
- `alias map<K, V> = [(K, V)];`
- `alias set<T> = [T];`
- `alias byte = u8;`
- `alias bytes = [u8];`

## Service

Service definition

- `extends` List of services to extend
- `events` List of service events
- `functions` Service functions
- `types` Service types

```js
service <ident>[@<interface_id>] {
    extends {}
    events {}
    functions {}
    types {}
}
```

Each block is optional and may appear in any order; the listing above is conventional, not required.

The optional `@<interface_id>` suffix pins the service's identifier to a
specific 8-byte value, written as `@0x` followed by 16 lowercase hex digits.
For non-`@partial` services it is **optional**: omit it and the parser
auto-computes the canonical id from the service signature; supply it and the
parser validates the canonical id matches and rejects the IDL on mismatch.

For `@partial` services it is **required** - the partial signature is a
subset, so the parser cannot recompute the original service's id from it
(see [Partial Service Subset](#partial-service-subset)).

Service IDL is self-contained. Types referenced by service functions, events,
and `throws` declarations are resolved from the service's own `types` block and
from explicitly extended service interfaces. Program-level `types` are not an
ambient scope for services, so a service can be distributed independently from a
particular program definition.

Program-level `types` are available to program constructors and other
program-level declarations. A program may expose services through `services`,
but that does not make program-local types visible inside those services.

Validation rules enforced by the parser:

- `@entry_id` values must be unique among a service's functions.
- `@entry_id` values must be unique among a service's events.

### Service Events

Service event is represented as an enum variant with an associated payload.

Events in `events { ... }` are modeled as `Enum Variant` describing fields of the event,
so the same machinery as for enums can be reused.

- `@entry_id: <number>` allows overriding the automatic positional index (which starts from 0 for the first member).

### Service functions

Service function entry

```js
/// Some documentation
[@query]
<ident>([<param_1>[, <param_n>]*]) [-> <output>] [throws <throws_type>];
```

- `params` is the ordered list of function parameters;
- `output` is the return type (use `PrimitiveType::Void` for `()` / no value);
- `throws` is an optional error type after the `throws` keyword;
- `@query` marks read-only / query functions as defined by the spec;
- `@entry_id: <number>` allows overriding the automatic positional index (which starts from 0 for the first member);
- may contain documentation comments and annotations.

### Partial Service Subset

The `@partial` annotation allows defining a subset of an original service. This is useful when generating a client for only specific methods of a large contract. When using `@partial`, the service **MUST** have an explicit `interface_id` (e.g., `service Name@0x...`).

Example of a partial IDL:

```js
@partial
service PartialService@0x1234567890abcdef {
    events {
        @entry_id: 2
        SomethingHappened(String);
    }
    functions {
        @entry_id: 5
        SomeMethod() -> bool;
    }
}
```

In this case, the generator will use the provided `interface_id` and `entry_id: 5`, ensuring compatibility with the original contract regardless of how many other methods it has.

### Example

```js
!@sails: 0.1.0
!@include: ownable.idl
!@include: git://github.com/some_repo/tippable.idl

/// Canvas service
service Canvas {
    // Merge `functions`, `events`, `types`, from Ownable, Tippable and Pausable services
    extends {
        Ownable,
        Tippable,
        Pausable,
    }

    // Canvas service events
    events {
        StatusChanged (Point<u32>),
        Jubilee {
            /// Amount of alive points.
            @indexed
            amount: u64,
            bits: [u8],
        },
        E1,
    }

    functions {
        /// Sets color for the point.
        /// app -> `fn color_point(&mut self, point: Point<u32>, color: Color) -> Result<(), ColorError>`
        /// On `Ok` - auto-reply. On `Err` -> app will encode error bytes of `ColorError` (`gr_panic_bytes`).
        ColorPoint(point: Point<u32>, color: Color) throws ColorError;

        /// Kills the point.
        /// app -> `fn kill_point(&mut self, point: Point<u32>) -> Result<bool, String>`
        KillPoint(point: Point<u32>) -> bool throws String;

        /// Returns known points.
        /// app -> `fn points(&self, ...) -> Result<BTreeMap<Point<u32>, PointStatus>, String>`
        @query
        Points(offset: u32, len: u32) -> map<Point<u32>, PointStatus> throws String;

        /// Returns status set for given point.
        @query
        PointStatus(point: Point<u32>) -> Option<PointStatus>;
    }

    types {
        struct Color {
            color: [u8; 4],
            space: ColorSpace,
        }

        /// Error happened during point setting.
        enum ColorError {
            InvalidSource,
            DeadPoint,
        }

        enum ColorSpace {
            RGB,
            HSV,
            CMYK,
        }

        // Point with two coordinates.
        struct Point<T> {
            /// Horizontal coordinate.
            x: T,
            /// Vertical coordinate.
            y: T,
        }

        /// Defines status of some point as colored by somebody or dead for some reason.
        enum PointStatus {
            /// Colored into some RGB.
            Colored {
                /// Who has colored it.
                author: actor,
                /// Color used.
                color: Color,
            },
            /// Dead point - won't be available for coloring anymore.
            Dead,
        }
    }
}

/// Pausable Service
service Pausable {
    events {
        Paused,
        Unpaused,
    }

    functions {
        // Client: `fn pause(&mut self) -> Result<(), SailsEnvError>`
        Pause();
        Unpause();
    }

    types {
        struct PausedError;
    }
}
```

## Programs

Program definition

- `constructors` List of program constructors
- `services` List of exported services
- `types` Program types

Each block is optional and may appear in any order. At most one `program` declaration is permitted per IDL document.

Validation rules enforced by the parser:

- `@entry_id` values must be unique among program constructors.

### Program constructors

Constructor entry

```js
/// Some documentation
<ident>([<param_1>[, <param_n>]*]) [throws <throws_type>];
```

- `params` is the ordered list of constructor parameters;
- `throws` is an optional error type after the `throws` keyword;
- constructors have no return type (they yield the program instance);
- `@entry_id: <number>` allows overriding the automatic positional index (which starts from 0 for the first member);
- may contain documentation comments and annotations.

### Program services

Service export entry

```js
/// Some documentation
<service_ident>[: <route>];
```

- `service_ident` is the service identifier, optionally suffixed with `@<interface_id>` (see [Service](#service));
- `route` is the optional routing name; when omitted, the service identifier is used as the route;
- `route_idx` is assigned positionally starting from 1 in declaration order;
- may contain documentation comments and annotations.

```js
!@sails: 0.1.0
!@include: canvas.idl

@author: me
/// This is my program that export Canvas service.
program DemoCanvas {
    constructors {
        Create();
        // app -> `fn with_owner(owner: ActorId) -> Result<Self, ZeroOwnerError>`;
        WithOwner(owner: actor) throws ZeroOwnerError;
    }

    services {
        Canvas: Canvas;

        /// Canvas service with route `DemoCanvas`.
        Canvas: DemoCanvas;
    }

    types {
        struct ZeroOwnerError;
        // Alias to imported type from included `canvas.idl`
        alias Paused = Pausable::PausedError;
    }
}
```

> NOTE
>
> 1. `extends` merge `events`, `functions` and `types` with service declaration according to
>    - `functions` override declaration
> 2. Namespaces are not included in the specification
>    - reference to service type `<service_ident>::<type_ident>` from program/root scopes (TBD)
>    - service declarations do not implicitly resolve names from `program.types`
> 3. Parsers expose the computed `interface_id` on each service in the AST, so tooling can recover canonical ids by parsing IDL with the suffix omitted.

## Codec Annotations and Dispatch Paths

The Sails header dispatch path uses SCALE-encoded payload bytes after the header.
Generic header-first tooling should decode this path as SCALE.

IDL v2 may also carry local `@codec` annotations on exported methods and events.
Current codec tokens are `scale` and `ethabi`.
The annotation describes which generated dispatch paths expose the item:

- No `@codec` annotation means both dispatch paths are available where the target supports them.
- `@codec: scale` means the item is available through SCALE/Gear dispatch.
- `@codec: ethabi` means the item is available through Solidity ABI dispatch.
- `@codec: scale,ethabi` means both paths are available.

Solidity ABI dispatch is a separate ethexe/Solidity-facing path and is not implied by the Sails header alone.
Tools that only implement the header-first Sails dispatcher must not silently decode `ethabi`-only entries as SCALE.
