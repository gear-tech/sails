## Intro

There are two ways of generating IDL - with & without program section. The former one is useful for the **end-dapps** while the latter one is more suitable for describing general standards, like VFT and etc.

By the **end-dapp** we mean a sails program which exposes services for the communication to the other programs or users.
When end-dapp IDL is generated, a program name is provided as an input to the generator. Otherwise - it's not.
The IDL consists of 3 big sections:
- program section
- service(s) section(s)
- global annotations

For the clarification it must be stated that `idl-gen` is referred to the actual implementation of the described standard in the doc.

## Global annotations

Global annotations section currently consists only from `!@sails` annotation, which tells the version of the IDL-generator currently used.
It has been planned to include the `!@include` annotation which is intended to include services from other IDLs, but that's currently under research, because of IDL's registry project.

## Program section

An IDL has a program section, if:
1. end-dapp IDL-generation function was called
2. end-dapp actually has any of constructors.

It must be stated, that `sails_macro::program` always generates a default constructor for the program in case it doesn't have any (create constructor). However idl-gen must not rely anyhow on the macro implementation.

Program section consists of:
1. Constructors sub-section which is basically functions that are called in init based on payload data provided by the user. The functions doesn't have returning value, but can have params.
2. Types sub-section which consists of user defined types used in params for constructors
3. Services sub-section


### Services sub-section

Services sub-section is an enumeration of the exposed services. The exposed service is the top-level service, which program declared as the one that users/program can call. From the program development perspective, exposed service is the one, which is returned by the method of the program in program's impl covered with `sails_macro::program`, like this:

```rust
#[sails_macro::program]
impl Program {
    fn svc1(&self) -> Svc1 { .. }
}
```

In the code snippet the `Svc1` is an exposed service. Beside exposed services there are base services, which are also possible to be invoked by users/programs when interacting with a program.

#### Service name
Before the Sails binary protocol `svc1` was considered as an exposed name of the `Svc1` service. So routing model of the sails programs communication required defining bytes of `svc1` in message payload in order to call `Svc1` service methods. In Sails binary protocol it now doesn't matter what's the name of the method under `sails_macro::program`, but the only information of the method that's still relevant is the returning `Svc1` name. 

#### Base services
Although it's possible to invoke the base service of the program, the IDL doesn't state this information in services sub-section of the program section. The reader of the IDL can understand by the declaration of the service in the separate service section (the one described below) whether the service is an extension of some other base service. The latter gives a hint to the reader about additional functionality available besides the exposed one. The IDL parser parses the IDL and creates a client which is able to call those base services.

#### Duplicates
There could be the cases when services with same names or even same services are exposed many times in the program section. See, for example, a program like this:
```rust
#[sails_macro::program]
impl MyProgram {
    fn svc1_1(&self) -> Svc1 { .. }
    fn svc1_2(&self) -> Svc1 { .. }
    fn svc1_other(&self) -> other_mod::Svc1 { .. }
}
```

**TODO [future] Make clarification on how it's going to be in IDL and resolved in the program under the hood (interface ids, route ids)**
In this case the `idl-gen` will generate services section the following way:
```
services {
    Svc1,
    Svc1,
    Svc1
}
```

## Service section
Program can have multiple services or no services at all.

Every sub-section of the service is only related to that service, i.e. functions, events and types of the service are only entities of the service. It's impossible to define events or functions of the other service in the current service - there won't be any reference to the other service. So these functions, events, types are only of the current service. That's an important point in understanding the extends sub-section which will be described later.

### Types sub-section
Service section itself consists of other sub-sections. The type sub-section here is similar to the type sub-section in program section. It consists only of types that are user defined, i.e. not types provided by Rust, like primitives, strings, map (`BTreeMap`), vectors, arrays or tuples.

There are some types that are not included into types sub-section and come for the users out of the box:
- `ActorId`
- `MessageId`
- `H160`
- `H256`
- `u256`, which is, basically, parity's `primitive_types::U256`.
- `Option<T>`
- `Result<T, E>` 
- `NonZero<T>`, where `T` stands for `u8`, `u16`, `u32`, `u64`, `u128`. The `NonZero` is used only as a concrete type with no generic params, like `Option` and `Result` could have. So in IDL source code you can define (or `idl-gen` can generate) `NonZero<u8>`, `NonZero<u16>`, `NonZero<u32>`, `NonZero<u64>`, `NonZero<u128>`, `NonZero<u256>, `but no `NonZero<T>`.

#### Declaration
Types are declared the Rust way. So types fields can be both user defined types or types provided by the Rust itself (like tuples, arrays, primitives and etc).  User defined types are `struct` and `enum`. So there could be:
- unit structs or no variant enums
- tuple structs and named fields structs
- enums with unit variants, tuple variants like:
```rust
enum TypesSubSectionEnum {
    TupleVariant(u32, bool),
    NamedFieldsVariant {
        field1: u32,
        field2: [bool; 32],
    }
}
```

#### Docs
Structs and enums in types sub-section can have doc strings:
```rust
/// This is unit struct
struct UnitStruct;

/// This is tuple struct
struct TupleStruct(u32);

/// This is named fields struct
struct NamedFieldsStruct {
    /// Field comment
    f1: u32
}

/// Extravagant - tuple field docs
struct TupleFieldDocsStruct (
    /// Field docs
    u32,
)

/// This is enum doc
enum SomeEnum {
    /// This is unit variant
    Unit,
    /// This is tuple variant
    Tuple(u32, u32),
    /// This is named fields variant
    NamedFields {
        /// Field 1 docs
        f1: u32,
        /// Field 2 docs
        f2: u32,
    }
    /// This is tuple fields docs
    Tuple2 (
        /// Tuple field docs
        u32
    )
}
```

#### Generics
Another feature of the `idl-gen` is that it recognizes the generic params of types and declares user defined types in types sub-section with generics, but uses concrete types intead of generics in functions, events or other types - same way it was done in the program at the time of it's coding in Rust. For example:
```rust
struct GenericComplex<T> {
    field1: GenericStruct<ActorId>,
    field2: GenericStruct<Result<bool, String>>,
    field3: GenericStruct<Option<T>>,
}

struct GenericStruct<T> {
    field: T
}
```

The generated code in IDL actually means that the original program code had `GenericComplex` struct with fields defined exactly same way: first two fields have concerete types for generic params, the last field has generic param wrapped in `Option`, which itself is a type param for `GenericStruct`.

#### Finalized form
Most of types are dumped into IDL the same way they were defined in the program source code. Like `u32` is written as `u32`, user defined `MyStruct<T>` will be the same `MyStruct<T>`. However, that's not the case for vectors (dynamic arrays) and maps:
- `[T]` - definition of the generic dynamic array, i.e., vector of type `T`.
- `[(K, V)]` - definition of the generic map, which is actually a dynamic array of tuples with 2 values of type `K` and `V`.

So `idl-gen` generates dynamic arrays and maps the described above way.

### Functions sub-section
Functions sub-section consists of functions definitions. The definitions here are the same as in program section except for constructors in the latter do not have returning values.

There are two types of functions: `common` and `query` ones. The query ones state that calling them shouldn't result in state changes. At least, it's not expected.

Basically, functions look that way:
```
functions {
    /// Function accepts no args, returns no value
    VoidFunction();

    /// Function accepts args, returns no value
    /// See, param2 accepts type with generic params being defined as concrete ones. There are no generics when type is used.
    Function1(param1: bool, param2: GenericStruct<Option<String>>);

    /// Function accepts args, returns a value
    Function2(param1: bool) -> bool;

    /// Function returns an error (in Rust it's `Result<(), String>`)
    Function3() throws String;

    /// Function returns a value or an error (in Rust it's `Result<bool, String>`)
    Function4() -> bool throws String;

    /// Complete example
    /// In Rust that's:
    /// `fn complete(
    ///    p1: ActorId,
    ///    p2: SomeGeneric<
    ///      [u8; 32],
    ///      (MessageId, H256, u256),
    ///      BTreeMap<GenericStruct<bool>, Result<(u16,), String>>
    ///    >,
    /// ) -> Result<Option<NonZeroU16>, MyErrorType>`
    Complete(p1: ActorId, p2: SomeGeneric<[u8; 32], (MessageId, H256, u256), [(GenericStruct<bool>, Result<Option<(u16,)>, String>)]>) -> Option<NonZero<u16> throws MyErrorType;

    /// Queries are completely the same, but they have `@query` over them
    @query
    Noop();

    /// Query return nothing
    @query
    Function5(param1: bool, param2: GenericStruct<Option<String>>);

    /// Query returns bool
    @query
    Function6(param1: bool) -> bool;

    /// Query returns an error (in Rust it's `Result<(), String>`)
    @query
    Function7() throws String;

    /// Query returns a value or an error (in Rust it's `Result<bool, String>`)
    @query
    Function8() -> bool throws String;
}
```

Functions both common ones and queries under one service section have unique names. Common and queries functions have the same "namespace".

### Events sub-section
Events sub-section definition is identical to the user defined enum declaration except there's no separate name like enum has. So events sub-section looks like this:
```
events {
    /// Unit field event docs
    Created,
    /// Tuple field event docs
    Initialized(ActorId, String)
    /// Struct field event docs
    Sent {
        /// `to` field of the event `Sent` docs
        to: ActorId,
        amount: u128
    },
    Dropped (
        /// Tuple field docs
        ActorId,
        String,
        bool
    )
}
```

### Extends sub-section
IDL services can be extended by other services. In this case the base services, which extended the current one, are mentioned in the extends sub-section the following way:
```
service BaseService { .. }
service ExtendedService {
    extends {
        BaseService
    }
    
    ...
}
```

#### Duplicates
Similar to the service sub-section of the program section, the extends sub-section can contain duplicates. Say, the service has defined it's base services the following way:
```rust 
#[sails_macro::service(extends = [Base, other_mod::Base])]
impl Extension { .. }
```

**TODO [future] Make clarification on how it's going to be in IDL and resolved in the program under the hood (interface ids, route ids). The basic sense of the clarification - services in extends section are unique by their ids**

In this case the generated IDL will be like this:
```
extends {
    Base,
    Base
}
```

However the following example with the same service defined more than one time in the extends argument of the macro will not compile and also will fail generating the IDL:
```rust 
#[sails_macro::service(extends = [Base, Base])]
impl Extension { .. }
```

#### One-time generation
Services with the same interface id are generated only once regardless of the relationships between these services and their exposure status. 
For example, there are two exposed services both are extensions for the same base service. In this case, the IDL will have 3 services:
```
service BaseService { .. }
service ExposedService1 {
    extends {
        BaseService
    }
    ..
}
service ExposedService2 {
    extends {
        BaseService
    }
    ..
}
```

