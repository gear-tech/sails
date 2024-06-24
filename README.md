# Sails &emsp;

Sails is a library for bringing your experience of writing applications utilizing
[Gear Protocol](https://gear-tech.io/) to the next level of simplicity and
clarity. It deals with things like:
- eliminating
the necessity of writing some low-level boilerplate code and letting you to stay
focused on your bussiness problem
- generated [IDL](https://en.wikipedia.org/wiki/Interface_description_language) file for
your application
- generated client allowing to interact with your application from code written in
different languages and executed in different runtimes

---

## Getting started

Add the following to your `Cargo.toml`
```toml
[dependencies]
sails-rtl = { git = "https://github.com/gear-tech/sails" }
gstd = { git = "https://github.com/gear-tech/gear", features = ["debug"] }
```

And then:

```rust
#![no_std]

use gstd::debug;
use sails_rtl::{gstd::gservice, prelude::*};

struct MyPing;

#[gservice]
impl MyPing {
    pub const fn new() -> Self {
        Self
    }

    pub async fn ping(&mut self) -> bool {
        debug!("Ping called");
        true
    }
}

#[derive(Default)]
struct MyProgram;

#[gprogram]
impl MyProgram {
    #[groute("ping")]
    pub fn ping_svc(&self) -> MyPing {
        MyPing::new()
    }
}
```

## Details

The entire idea of the [Gear Protocol](https://gear-tech.io/) is based on the
asynchronous version of the [Request-Response Pattern](https://en.wikipedia.org/wiki/Request%E2%80%93response).
On-chain applications loaded onto Gear-based network receive and handle messages from the
other on-chain or off-chain applications. Both can be treated as external consumers
of services provided by your application, and the latter can represent ordinary people
interacting with the network.

### Application

Sails architecture for applications is based on a few key concepts.
<br/>

The first one is *__service__* which is represented by an impl of some Rust struct
marked with the `#[gservice]` attribute. The service main responsibility is
implementing some aspect of application business logic. A set of its __public__
methods defined by the impl is essentially a set of remote calls the service exposes
to external consumers. Each such method working over a `&mut self` is treated as
a command changing some state, whereas each method working over a `&self` is
treated as a query keeping everything unchanged and returning some data. Both
types of methods can accept some parameters to be passed by a client. Both types of
methods can be synchronous or asynchronous. All the other service's methods and
associated functions are treated as implementation details and ignored. The code
generated behind the service by the `#[gservice]` macro decodes an incoming request
message and dispatches it to the appropriate method based on the method's name.
On the method's completion, its result is encoded and returned as a response to a caller.

```rust
#[gservice]
impl MyService {
    // This is a command
    pub fn do_something(&mut self, p1: u32, p2: String) -> &'static [u8] {
        ...
    }

    // This is a query
    pub fn some_value(&self, p1: Option<bool>) -> String {
        ...
    }
}
```

<br/>

The second key concept is *__program__* which is similarly to service represented
by an impl of some Rust struct marked with the `#[gprogram]` attribute. The program
main responsibility is hosting one or more services and exposing them to the external
consumers. A set of its __public__ associated functions returning `Self` are
treated as application constructors. These functions can accept some parameters
to be passed by a client. They can be synchronous or asynchronous. One of them will
be called once per application lifetime when the application is loaded onto the network.
A set of program's __public__ methods working over `&self` and having no other parameters
are treated as exposed service constructors and are called each time when an incoming
request message needs be dispatched to a selected service. All the other methods and
associated functions are treated as implementation details and ignored. The code
generated behind the program by the `#[gprogram]` macro receives an incoming request
message from the network, decodes it and dispatches it to a matching service for actual
processing. After that, the result is encoded and returned as a response to a caller.
Only one program is allowed per application.

```rust
#[gprogram]
impl MyProgram {
    // Application constructor
    pub fn new() -> Self {
        ...
    }

    // Yet another application constructor
    pub fn from_u32(p1: u32) -> Self {
        ...
    }

    // Service constructor
    pub fn ping_svc(&self) -> MyPing {
        ...
    }
}
```

<br/>

And the final key concept is message *__routing__*. This concept doesn't have a
mandatory representation in code, but can be altered by using the `#[groute]`
attribute applied to those public methods and associated functions described above.
The concept itself is about rules for dispatching an incoming request message to
a specific service's method using service and method names. By default, every
service exposed via program is exposed using the name of the service constructor
method converted into *PascalCase*. For example:

```rust
#[gprogram]
impl MyProgram {
    // The `MyPing` service is exposed as `PingSvc`
    pub fn ping_svc(&self) -> MyPing {
        ...
    }
}
```

This behavior can be changed by applying the `#[groute]` attribute:

```rust
#[gprogram]
impl MyProgram {
    // The `MyPing` service is exposed as `Ping`
    #[groute("ping")] // The specified name will be converted into PascalCase
    pub fn ping_svc(&self) -> MyPing {
        ...
    }
}
```

The same rules are applicable to service method names:

```rust
#[gservice]
impl MyPing {
    // The `do_ping` method is exposed as `Ping`
    #[groute("ping")]
    pub fn do_ping(&mut self) {
        ...
    }

    // The `ping_count` method is exposed as `PingCount`
    pub fn ping_count(&self) -> u64 {
        ...
    }
}
```

### Payload Encoding

An application written with Sails uses [SCALE Codec](https://github.com/paritytech/parity-scale-codec) to encode/decode data
at its base.
<br/>

Every incoming request message is expected to have the following format:
<br/>

__|__ *SCALE encoded service name* __|__ *SCALE encoded method name* __|__ *SCALE encoded parameters* __|__
<br/>

Every outgoing response message has the following format:
<br/>

__|__ *SCALE encoded service name* __|__ *SCALE encoded method name* __|__ *SCALE encoded result* __|__

### Service Extending

[TBD]

### Client

[TBD???]

### Examples

[TBD]

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
