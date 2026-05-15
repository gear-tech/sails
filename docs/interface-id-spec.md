# Sails Service Interface ID Specification

This document specifies how interface IDs are generated for Sails services at compile time.

## Overview

Each Sails service has a unique 8-byte **Interface ID** that is computed deterministically at compile time based on the service's structural definition: its functions, their signatures, events, and inherited base services. The Interface ID provides **unique service fingerprinting**, so two services with different structures will have different IDs, even if they have the same name

## Terminology

- **HASH()**: Keccak256 hash function
- **REFLECT_HASH**: A type's 32-byte structural hash computed via the `ReflectHash` trait
- **||**: Byte concatenation operator
- **bytes(s)**: UTF-8 byte representation of string `s`

## Interface ID Computation

The Interface ID is computed as:

```
PRE_INTERFACE_ID = HASH(FN_HASH_1 || FN_HASH_2 || ... || FN_HASH_N || EVENTS_HASH || BASE_SERVICE_HASH_1 || ... || BASE_SERVICE_HASH_M)

# First 8 bytes of the hash
INTERFACE_ID = PRE_INTERFACE_ID[0..8]
```

Where:
- `FN_HASH_N`: Hash of service's function *N*
- `EVENTS_HASH`: Hash of the service's event type (if present)
- `BASE_SERVICE_HASH_M`: Hash of extended base services (if present)

### Functions Hash

Functions are sorted lexicographically by their route name (case-insensitive). Functions are sorted by the **lowercase** version of their route names, but the original case is preserved in the hash computation.

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

Each function argument hash is just a structural hash of its type returned from `ReflectHash` trait implementation:
```
ARG_HASH = REFLECT_HASH
```

Arguments are processed in their declaration order.

#### Result Hash (RES_HASH)

The result hash depends on whether the return type is a `Result<T, E>`:

**For non-Result types:**
```
RES_HASH = bytes("res") || REFLECT_HASH
```
It's a concatenation of the UTF-8 bytes of the string "res" and the structural hash of the return type.

**For CommandReply<T> result:**
If a function returns `CommandReply<T>`, the type `T` is extracted and used for hashing. The `CommandReply` wrapper itself is not included in the hash since it's a protocol-level concern, not a logical interface concern.

```
RES_HASH = bytes("res") || T::REFLECT_HASH
```
It's a concatenation of the UTF-8 bytes of the string "res" and the structural hash of the inner type `T`.

**For Result<T, E> types:**
```
RES_HASH = bytes("res") || T::REFLECT_HASH || bytes("throws") || E::REFLECT_HASH
```
It's a concatenation of:
- UTF-8 bytes of the string "res"
- Structural hash of the success type `T`
- UTF-8 bytes of the string "throws"
- Structural hash of the error type `E`

### Events Hash

If the service declares an events type using `#[service(events = EventType)]`:

```
EVENTS_HASH = EventType::REFLECT_HASH
```

If no events type is declared, this component is omitted from the interface ID computation.

### Base Services Hash

If the service extends base services, their interface IDs are sorted lexicographically by the base service type name (case-insensitive), and concatenated:

```
BASE_ID_1 || BASE_ID_2 || ... || BASE_ID_N
```

Where each `BASE_ID_i` is the 8-byte Interface ID of a base service.

Base services are sorted by their type identifier (the last segment of their type path) in ascending case-insensitive lexicographical order. The sorting uses the **lowercase** version of type names, but the actual Interface IDs are used in the hash (not the names themselves).

So if a service is declared as:

```rust
#[service(base = [BaseServiceC, ServiceD, BaseServiceB, ServiceA])]
```

The base services would be sorted as: `[BaseServiceB, BaseServiceC, ServiceA, ServiceD]` for hashing purposes.

If no base services exist, this component is omitted from the interface ID computation.

## Determinism Guarantees

The interface ID computation is **deterministic** and **stable** under:

1. **Reordering fields**: Changing field order in structs/enums changes the hash
2. **Adding/removing functions**: Changes the FUNCTIONS_HASH
3. **Changing function types**: query ↔ command changes FN_TYPE and thus the hash
4. **Adding/removing events**: Changes EVENTS_HASH presence
5. **Adding/removing base services**: Changes final hash because added/removed base service IDs affects the final computation
6. **Renaming fields**: Field name changes do NOT affect the hash (structural hashing)
7. **Reordering functions**: Does NOT change FUNCTIONS_HASH (functions are sorted)
8. **Reordering base services**: Does NOT change the final hash (base services are sorted)

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
    bytes("query") ||
    bytes("Get") ||
    bytes("res") || u32::HASH
)

INCREMENT_FN_HASH = HASH(
    bytes("command") ||
    bytes("Increment") ||
    u32::HASH ||
    bytes("res") || u32::HASH
)

INTERFACE_ID = HASH(GET_FN_HASH || INCREMENT_FN_HASH)
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
EVENTS_HASH = CounterEvents::HASH
INTERFACE_ID = HASH(INCREMENT_FN_HASH || EVENTS_HASH)
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
    bytes("command") ||
    bytes("Transfer") ||
    ActorId::HASH ||
    u128::HASH ||
    bytes("res") || unit::HASH || bytes("throws") || Error::HASH
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

AUDITOR_ID = Auditor::INTERFACE_ID  // 8 bytes
LOGGER_ID = Logger::INTERFACE_ID     // 8 bytes

INCREMENT_FN_HASH = HASH(
    bytes("command") ||
    bytes("Increment") ||
    bytes("res") || u32::HASH
)

INTERFACE_ID = HASH(INCREMENT_FN_HASH || AUDITOR_ID || LOGGER_ID)
```

Note: If Logger and Auditor themselves extend other services or have events, those are already included in their respective INTERFACE_IDs, creating a recursive dependency tree.
