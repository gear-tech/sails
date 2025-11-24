# ReflectHash Specification

This document specifies how structural hashes are computed for Rust types at compile time using the `ReflectHash` trait.

## Overview

`ReflectHash` is a trait that provides a **deterministic, compile-time, structural hash** for Rust types. Each type implementing `ReflectHash` has a constant 32-byte hash computed using Keccak256 based solely on the type's structure, independent of field names.

The structural hash enables:
- **Type fingerprinting**: Unique identifiers for types based on their structure
- **Interface stability tracking**: Detect when type structures change across versions
- **Structural equivalence**: Types with the same structure produce the same hash, regardless of field naming

## Terminology

- **HASH()**: Keccak256 hash function
- **||**: Byte concatenation operator
- **bytes(s)**: UTF-8 byte representation of string `s`
- **TypeName**: The identifier of a type (struct, enum, variant) as a string

## Core Trait

```rust
pub trait ReflectHash {
    /// The 256-bit structural hash of this type, computed at compile time.
    const HASH: [u8; 32];
}
```

## Deriving ReflectHash

The easiest way to implement `ReflectHash` is to derive it:

```rust
#[derive(ReflectHash)]
struct MyStruct {
    field: u32,
}
```

The derive macro automatically generates the correct implementation based on the type's structure.

### Custom Crate Path

If `sails-reflect-hash` is re-exported under a different name, specify the crate path:

```rust
use sails_reflect_hash as my_hash;

#[derive(ReflectHash)]
#[reflect_hash(crate = my_hash)]
struct MyType {
    field: u32,
}
```

## Hashing Rules

### Primitive Types

Primitive types are hashed by their type name:

```
HASH = HASH(bytes(TypeName))
```

**Examples:**
```
u8::HASH     = HASH(b"u8")
u32::HASH    = HASH(b"u32")
bool::HASH   = HASH(b"bool")
String::HASH = HASH(b"String")
str::HASH    = HASH(b"String")  // str and String are structurally equivalent
```

**Supported primitives:**
- Unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`
- Signed integers: `i8`, `i16`, `i32`, `i64`, `i128`
- Boolean: `bool`
- Character: `char`
- String types: `String`, `str` (both hash to `"String"`)

### Non-Zero Types

Non-zero wrapper types include both the wrapper name and the inner type:

```
NonZeroT::HASH = HASH(b"NonZeroT" || T::HASH)
```

**Examples:**
```
NonZeroU8::HASH   = HASH(b"NonZeroU8" || u8::HASH)
NonZeroU32::HASH  = HASH(b"NonZeroU32" || u32::HASH)
NonZeroI64::HASH  = HASH(b"NonZeroI64" || i64::HASH)
NonZeroU256::HASH = HASH(b"NonZeroU256" || U256::HASH)
```

### References

References are **transparent** - they have the same hash as the referent:

```
&T::HASH     = T::HASH
&mut T::HASH = T::HASH
```

This reflects that references are structurally equivalent to their referent in interface terms.

### Unit Type

The unit type `()` is hashed as:

```
()::HASH = HASH(b"()")
```

### Tuples

Tuples are hashed by concatenating the hashes of their elements:

```
(T1, T2, ..., TN)::HASH = HASH(T1::HASH || T2::HASH || ... || TN::HASH)
```

**Examples:**
```
(u32, u64)::HASH          = HASH(u32::HASH || u64::HASH)
(String, bool, u8)::HASH  = HASH(String::HASH || bool::HASH || u8::HASH)
```

**Note:** Tuples up to 12 elements are supported.

### Arrays

Arrays are hashed by the element type and the array length:

```
[T; N]::HASH = HASH(T::HASH || bytes(stringify!(N)))
```

**Examples:**
```
[u8; 32]::HASH  = HASH(u8::HASH || b"32")
[bool; 10]::HASH = HASH(bool::HASH || b"10")
```

**Note:** Arrays up to size 32 have implementations.

### Slices & Vec

Slices have the same hash as vectors of the same element type:

```
[T]::HASH = HASH(b"Vec" || T::HASH)
```

This reflects that slices and vectors are structurally equivalent in interface terms.

### Option

```
Option<T>::HASH = HASH(b"Option" || T::HASH)
```

**Example:**
```
Option<u32>::HASH = HASH(b"Option" || u32::HASH)
```

### Result

```
Result<T, E>::HASH = HASH(b"Result" || T::HASH || E::HASH)
```

**Example:**
```
Result<u32, String>::HASH = HASH(b"Result" || u32::HASH || String::HASH)
```

### BTreeMap

```
BTreeMap<K, V>::HASH = HASH(b"BTreeMap" || K::HASH || V::HASH)
```

**Example:**
```
BTreeMap<String, u64>::HASH = HASH(b"BTreeMap" || String::HASH || u64::HASH)
```

### Structs

For structs, the hash includes the type name and field types **in declaration order**, but **excludes field names**:

```
HASH = HASH(bytes(StructName) || T1::HASH || T2::HASH || ... || TN::HASH)
```

Where:
- `StructName` is the UTF-8 bytes of the struct's identifier
- `Ti::HASH` are the hashes of field types in declaration order
- Field names are **excluded**

#### Unit Structs

```rust
struct Empty;
```

```
Empty::HASH = HASH(b"Empty")
```

#### Tuple Structs

```rust
struct Point(u32, u64);
```

```
Point::HASH = HASH(b"Point" || u32::HASH || u64::HASH)
```

#### Named Structs

```rust
struct User {
    id: u64,
    name: String,
}
```

```
User::HASH = HASH(b"User" || u64::HASH || String::HASH)
```

**Important:** Field names (`id`, `name`) are **not** included in the hash. This means:

```rust
struct User1 { id: u64, name: String }
struct User2 { x: u64, y: String }
```

Both produce the same hash if they have the same field types in the same order:
```
User1::HASH = HASH(b"User1" || u64::HASH || String::HASH)
User2::HASH = HASH(b"User2" || u64::HASH || String::HASH)
```

The only difference is the struct name itself.

### Enums

For enums, the hash is computed by concatenating all variant hashes:

```
HASH = HASH(VARIANT_HASH_1 || VARIANT_HASH_2 || ... || VARIANT_HASH_N)
```

**Important:** The enum name itself is **NOT** included in the hash. Only the variant hashes are concatenated.

Variants are processed **in declaration order**.

Each variant is hashed like a struct:

```
VARIANT_HASH = HASH(bytes(VariantName) || T1::HASH || T2::HASH || ... || TN::HASH)
```

#### Unit Variants

```rust
enum Status {
    Active,
    Inactive,
}
```

```
Active_HASH   = HASH(b"Active")
Inactive_HASH = HASH(b"Inactive")
Status::HASH  = HASH(Active_HASH || Inactive_HASH)
```

#### Tuple Variants

```rust
enum Event {
    Transferred(ActorId, u128),
    Approved(ActorId, u128),
}
```

```
Transferred_HASH = HASH(b"Transferred" || ActorId::HASH || u128::HASH)
Approved_HASH    = HASH(b"Approved" || ActorId::HASH || u128::HASH)
Event::HASH      = HASH(Transferred_HASH || Approved_HASH)
```

#### Named Variants

```rust
enum Message {
    Transfer { from: ActorId, to: ActorId, amount: u128 },
    Burn { amount: u128 },
}
```

```
Transfer_HASH = HASH(b"Transfer" || ActorId::HASH || ActorId::HASH || u128::HASH)
Burn_HASH     = HASH(b"Burn" || u128::HASH)
Message::HASH = HASH(Transfer_HASH || Burn_HASH)
```

**Important:** Field names in named variants are **excluded**, just like in named structs.

#### Mixed Variants

```rust
enum Action {
    Stop,
    Move(i32, i32),
    SetName { name: String },
}
```

```
Stop_HASH     = HASH(b"Stop")
Move_HASH     = HASH(b"Move" || i32::HASH || i32::HASH)
SetName_HASH  = HASH(b"SetName" || String::HASH)
Action::HASH  = HASH(Stop_HASH || Move_HASH || SetName_HASH)
```

**Note:** The enum name "Action" is **not** included in the final hash - only the concatenated variant hashes.

### Gear Primitives

Special Gear types are hashed with their name and inner structure:

```
ActorId::HASH   = HASH(b"ActorId" || [u8; 32]::HASH)
MessageId::HASH = HASH(b"MessageId" || [u8; 32]::HASH)
CodeId::HASH    = HASH(b"CodeId" || [u8; 32]::HASH)
H256::HASH      = HASH(b"H256" || [u8; 32]::HASH)
H160::HASH      = HASH(b"H160" || [u8; 20]::HASH)
U256::HASH      = HASH(b"U256" || HASH(u64::HASH || b"4"))
```

## Determinism Guarantees

The hash computation is **deterministic** and **stable** under:

1. ✅ **Field/variant reordering**: Changing order CHANGES the hash
2. ✅ **Field/variant renaming**: Renaming does NOT change the hash (structural hashing)
3. ✅ **Adding/removing fields**: Changes the hash
4. ✅ **Adding/removing variants**: Changes the hash
5. ✅ **Type name changes**: Changing the type/variant name CHANGES the hash
6. ✅ **Generic instantiation**: Different generic arguments produce different hashes

## Misc Examples

### Nested Types

```rust
#[derive(ReflectHash)]
struct Inner {
    value: u32,
}

#[derive(ReflectHash)]
struct Outer {
    inner: Inner,
    extra: u64,
}
```

Hash computation:
```
Inner::HASH = HASH(b"Inner" || u32::HASH)
Outer::HASH = HASH(b"Outer" || Inner::HASH || u64::HASH)
```

### Generic Types

```rust
#[derive(ReflectHash)]
struct Container<T: ReflectHash> {
    data: T,
}
```

Each instantiation has a different hash:
```
Container<u32>::HASH = HASH(b"Container" || u32::HASH)
Container<String>::HASH = HASH(b"Container" || String::HASH)
```

## Limitations

1. **Unions are not supported**: `ReflectHash` cannot be derived for unions
2. **Generic bounds**: Generic types must have all type parameters implement `ReflectHash`
3. **Const generics**: Only const generics that can be stringified are supported
4. **Recursive types**: Self-referential types require manual implementation or indirection

## Implementation Notes

### Compile-Time Evaluation

All hash computations occur at compile time using `const` evaluation. The `HASH` constant is computed once during compilation and embedded in the binary.

### Keccak256

The hash function used is Keccak256 (not SHA3-256). The `keccak-const` crate provides compile-time compatible Keccak256.

### Crate Path Detection

The derive macro uses `proc-macro-crate` to automatically detect how `sails-reflect-hash` is imported. If detection fails or the crate is re-exported, use the `#[reflect_hash(crate = ...)]` attribute.
