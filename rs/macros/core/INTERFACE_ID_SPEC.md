# Sails Service Interface ID Specification

This document specifies how interface IDs are generated for Sails services at compile time.

## Overview

Each Sails service has a unique 32-byte **Interface ID** that is computed deterministically at compile time based on the service's structural definition: its functions, their signatures, events, and inherited base services. The Interface ID provides **unique service fingerprinting**, so two services with different structures will have different IDs, even if they have the same name

## Terminology

- **HASH()**: Keccak256 hash function
- **REFLECT_HASH**: A type's 32-byte structural hash computed via the `ReflectHash` trait
- **||**: Byte concatenation operator
- **bytes(s)**: UTF-8 byte representation of string `s`

## Interface ID Computation

The Interface ID is computed as:

```
INTERFACE_ID = HASH(FUNCTIONS_HASH || EVENTS_HASH || BASE_SERVICES_HASH)
```

Where:
- `FUNCTIONS_HASH`: Hash of all service functions
- `EVENTS_HASH`: Hash of the service's event type (if present)
- `BASE_SERVICES_HASH`: Hash of extended base services (if present)

### Functions Hash

Functions are sorted lexicographically by their route name (case-insensitive), then hashed:

```
FUNCTIONS_HASH = HASH(FN_HASH_1 || FN_HASH_2 || ... || FN_HASH_N)
```

Note: Functions are sorted by the **lowercase** version of their route names, but the original case is preserved in the hash computation.

Each function's hash is computed as:

```
FN_HASH = HASH(FN_TYPE || bytes(FN_NAME) || ARG_HASH_1 || ... || ARG_HASH_N || RES_HASH)
```

#### Function Type (FN_TYPE)

The function type distinguishes between queries and commands:

```
FN_TYPE = bytes("query")    if function takes &self (immutable reference)
FN_TYPE = bytes("command")  if function takes &mut self (mutable reference)
```

Note: The bytes are the UTF-8 representation of the string literals.

#### Function Name (FN_NAME)

The function's route name bytes. By default, the route name is the function identifier converted to PascalCase (e.g., `get_value` → `GetValue`). However, it can be overridden with `#[export(route = "custom_name")]`, which is also converted to PascalCase (e.g., `"custom_name"` → `CustomName`).

Note: The bytes are the UTF-8 representation of the string literals.

#### Argument Hash (ARG_HASH)

Each function argument is hashed individually:

```
ARG_HASH = HASH(bytes("arg") || REFLECT_HASH)
```

Where `REFLECT_HASH` is the 32-byte structural hash of the argument's type.

Arguments are processed in their declaration order.

#### Result Hash (RES_HASH)

The result hash depends on whether the return type is a `Result<T, E>`:

**For non-Result types:**
```
RES_HASH = HASH(bytes("res) || REFLECT_HASH)
```

**For CommandReply<T> result:**
If a function returns `CommandReply<T>`, the type `T` is extracted and used for hashing. The `CommandReply` wrapper itself is not included in the hash since it's a protocol-level concern, not a logical interface concern.

```
RES_HASH = HASH(bytes("res") || T::REFLECT_HASH)
```

**For Result<T, E> types:**
```
RES_HASH = HASH(bytes("res") || T::REFLECT_HASH || bytes("throws") || E::REFLECT_HASH)
```

### Events Hash

If the service declares an events type using `#[service(events = EventType)]`:

```
EVENTS_HASH = EventType::REFLECT_HASH
```

If no events type is declared, this component is omitted from the interface ID computation.

### Base Services Hash

If the service extends base services, their interface IDs are sorted lexicographically by the base service type name (case-insensitive), then hashed:

```
BASE_SERVICES_HASH = HASH(BASE_ID_1 || BASE_ID_2 || ... || BASE_ID_N)
```

Where each `BASE_ID_i` is the 32-byte Interface ID of a base service.

Base services are sorted by their type identifier (the last segment of their type path) in ascending case-insensitive lexicographical order. The sorting uses the **lowercase** version of type names, but the actual Interface IDs are used in the hash (not the names themselves).

So if a service is declared as:

```rust
#[service(base = [BaseServiceC, ServiceD, BaseServiceB, ServiceA])]
```

The base services would be sorted as: `[BaseServiceB, BaseServiceC, ServiceC, ServiceD]` for hashing purposes.

If no base services exist, this component is omitted from the interface ID computation.

## Type Structural Hashing (ReflectHash)

The `ReflectHash` trait computes a 32-byte structural hash for types at compile time.

### Primitives

Primitive types are hashed by their type name:

```rust
// Examples:
u32::HASH = HASH(b"u32")
bool::HASH = HASH(b"bool")
String::HASH = HASH(b"String")
str::HASH = HASH(b"String")  // str hashes same as String
```

### Compound Types

#### Tuples
```
HASH = HASH(b"(" || T1::HASH || T2::HASH || ... || TN::HASH || b")")
```

#### Arrays
```
HASH = HASH(b"[" || T::HASH || b";" || bytes(stringify!(N)) || b"]")
```

#### Vectors
```
HASH = HASH(b"Vec<" || T::HASH || b">")
```

#### Options
```
HASH = HASH(b"Option<" || T::HASH || b">")
```

#### Results
```
HASH = HASH(b"Result<" || T::HASH || b"," || E::HASH || b">")
```

### Structs

For structs, the hash includes the type name and field types (but NOT field names):

```
HASH = HASH(TypeName || T1::HASH || T2::HASH || ... || TN::HASH)
```

Where:
- `TypeName` is the UTF-8 bytes of the struct's identifier
- Field types are included in declaration order
- Field names are **excluded** to maintain structural equivalence

This applies to:
- **Unit structs**: `struct Foo;` → `HASH(b"Foo")`
- **Tuple structs**: `struct Foo(u32, u64);` → `HASH(b"Foo" || u32::HASH || u64::HASH)`
- **Named structs**: `struct Foo { x: u32, y: u64 }` → `HASH(b"Foo" || u32::HASH || u64::HASH)`

### Enums

For enums, the hash includes the enum name and all variant hashes:

```
HASH = HASH(EnumName || VARIANT_HASH_1 || VARIANT_HASH_2 || ... || VARIANT_HASH_N)
```

Each variant is hashed as:

```
VARIANT_HASH = HASH(VariantName || T1::HASH || T2::HASH || ... || TN::HASH)
```

Where:
- Variants are processed in declaration order
- Variant field types are included in declaration order
- Field names in named variants are **excluded**

Examples:
- Unit variant: `A` → `HASH(b"A")`
- Tuple variant: `B(u32, u64)` → `HASH(b"B" || u32::HASH || u64::HASH)`
- Named variant: `C { x: u32, y: u64 }` → `HASH(b"C" || u32::HASH || u64::HASH)`

## Determinism Guarantees

The interface ID computation is **deterministic** and **stable** under:

1. **Reordering fields**: Changing field order in structs/enums changes the hash
2. **Adding/removing functions**: Changes the FUNCTIONS_HASH
3. **Changing function types**: query ↔ command changes FN_TYPE and thus the hash
4. **Adding/removing events**: Changes EVENTS_HASH presence
5. **Adding/removing base services**: Changes BASE_SERVICES_HASH
6. **Renaming fields**: Field name changes do NOT affect the hash (structural hashing)
7. **Reordering functions**: Does NOT change FUNCTIONS_HASH (functions are sorted)
8. **Reordering base services**: Does NOT change BASE_SERVICES_HASH (base services are sorted)

## Examples

### Simple Service

```rust
#[service]
impl Counter {
    #[export]
    pub fn increment(&mut self, delta: u32) -> u32 { ... }
    
    #[export]
    pub fn get(&self) -> u32 { ... }
}
```

Interface ID computation:
```
// Functions sorted: ["Get", "Increment"]

GET_FN_HASH = HASH(
    b"query" ||
    b"Get" ||
    HASH(b"res" || u32::HASH)
)

INCREMENT_FN_HASH = HASH(
    b"command" ||
    b"Increment" ||
    HASH(b"arg" || u32::HASH) ||
    HASH(b"res" || u32::HASH)
)

FUNCTIONS_HASH = HASH(GET_FN_HASH || INCREMENT_FN_HASH)

INTERFACE_ID = HASH(FUNCTIONS_HASH)
```

### Service with Events

```rust
#[derive(ReflectHash)]
enum CounterEvents {
    Incremented(u32),
}

#[service(events = CounterEvents)]
impl Counter {
    #[export]
    pub fn increment(&mut self) { ... }
}
```

Interface ID computation:
```
FUNCTIONS_HASH = HASH(INCREMENT_FN_HASH)
EVENTS_HASH = CounterEvents::HASH
INTERFACE_ID = HASH(FUNCTIONS_HASH || EVENTS_HASH)
```

### Service with Result Type

```rust
#[service]
impl Wallet {
    #[export]
    pub fn transfer(&mut self, to: ActorId, amount: u128) -> Result<(), Error> { ... }
}
```

Function hash includes error type:
```
TRANSFER_FN_HASH = HASH(
    b"command" ||
    b"Transfer" ||
    HASH(b"arg" || ActorId::HASH) ||
    HASH(b"arg" || u128::HASH) ||
    HASH(b"res" || unit::HASH || b"throws" || Error::HASH)
)
```

### Service with Base Services

```rust
// Base service 1
#[service]
impl Logger {
    #[export]
    pub fn log(&self, message: String) { ... }
}

// Base service 2
#[service]
impl Auditor {
    #[export]
    pub fn audit(&self, action: String) { ... }
}

// Extended service
#[service(extends = [Logger, Auditor])]
impl SecureCounter {
    #[export]
    pub fn increment(&mut self) -> u32 { ... }
}
```

Interface ID computation with base services:
```
// Base services sorted case-insensitively: ["Auditor", "Logger"]
// (sorted by lowercase: "auditor" < "logger")

AUDITOR_ID = Auditor::INTERFACE_ID  // 32 bytes
LOGGER_ID = Logger::INTERFACE_ID     // 32 bytes

BASE_SERVICES_HASH = HASH(AUDITOR_ID || LOGGER_ID)

INCREMENT_FN_HASH = HASH(
    b"command" ||
    b"Increment" ||
    HASH(b"res" || u32::HASH)
)

FUNCTIONS_HASH = HASH(INCREMENT_FN_HASH)

INTERFACE_ID = HASH(FUNCTIONS_HASH || BASE_SERVICES_HASH)
```

Note: If Logger and Auditor themselves extend other services or have events, those are already included in their respective INTERFACE_IDs, creating a recursive dependency tree.


### Review results:
1. Calculate function's hashes differently: first calculate commands hashes, then queries. Inside commands and queries are sorted by route names.

```
INTERFACE_ID = HASH(COMMAND1_HASH || COMMAND2_HASH || QUERY1_HASH || EVENTS_HASH || BASE1_SERVICE_HASH)
COMMAND_HASH = FN_HASH
FN_HASH = HASH(bytes("command") || bytes(FN_NAME) || REFLECT_HASH || ... || ARG_HASH_N || RES_HASH)
ARG_HASH = REFLECT_HASH
RES_HASH = (REFLECT_HASH) OR (REFLECT_HASH || bytes("throws") || REFLECT_HASH)
```