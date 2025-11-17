# IDL V2 Spec

## Annotations

### Global

String represented value, starting from the beginning of file

- `!@<ident>[: <value>]`

Examples

- `!@sails: 0.1.0` // version of IDL
- `!@author: me` // author of the IDL
- `!@git: ...` // source code
- `!@version: 0.2.0` // protocol version

### @include (TBD)

- Syntax: @include: <path_to_idl>
- Possibly path can be a git-path (url)
- First iteration impl can be the same as idl-v1 (just include the text, no include key-words).
- Impl notes: when parsed `@include: <path_to_idl>`, then just include text to the file immediately

### Local

Local annotations goes before (above) related token.

- `@<ident>[: <value>]`

Examples

- `@doc: some comments` (shortcut: `///`)
- `@indexed`
- `@query`

## Comments

- Comments start from `//` and end where the line ends.
- Used for some IDL clarifications or details.
- Ignored on the IDL parsing.

## Types

### Common

- `bool`
- `char`
- `String`

### Numerics

- `u8`, `i8`
- `u16`, `i16`
- `u32`, `i32`
- `u64`, `i64`
- `u128`, `i128`

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
- `()` - not used in the "return" part of the function's signature,
  but used as an independent type

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

### Type Aliases (TBD)

`alias <ident> = <ident>`

#### Generic support (TBD)

- `alias NewType = Type<u8>`,
- `alias NewType<T> = Type<T>`.

#### Builtins (TBD)

- void
  - `aliad void = ();`
- list
  - `alias list<T> = [T]`
- map
  - `Vec<(K, V)>`
  - `alias map<K, V> = [(K, V)]`
- set
  - `alias set<T> = [T]`
- byte
  - `alias byte = u8`
- bytes
  - `alias bytes = [u8]`
- string
  - `alias string = [u8]`
- actor
  - `struct ActorId([u8; 32])`
  - `alias actor = ActorId`
- code
  - `struct CodeId([u8; 32])`
  - `alias code = CodeId`
- u256
  - `struct U256([u64; 4])`
  - `alias u256 = U256`
- h160
  - `struct H160([u8; 20])`
  - `alias h160 = H160`
- h256
  - `struct H256([u8; 32])`
  - `alias h256 = H256`

## Service

Service definition

- `extends` List of services to extend
- `events` List of service events
- `functions` Service functions
- `types` Service types

```js
service <ident> {
    extends {}
    events {}
    functions {}
    types {}
}
```

### Service Events

Service event is represented as an enum variant with an associated payload.

Events in `events { ... }` are modeled as `Enum Variant` describing fields of the event,
so the same machinery as for enums can be reused.

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
- may contain documentation comments and annotations.

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
            bits: bitvec,
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

Service definition

- `constructors` List of program constructors
- `services` List of exported services
- `types` Service types

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
        DemoCanvas: Canvas;
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
>    - `functions` override decalaration (TBD)
> 2. Namespaces are not included in the specification
>    - reference to service type `<service_ident>::<type_ident>` (TBD)
