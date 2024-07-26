# Sails &emsp;

Sails is a library for bringing your experience of writing applications utilizing
[Gear Protocol](https://gear-tech.io/) to the next level of simplicity and
clarity. It deals with things like:
- eliminating
the necessity of writing some low-level boilerplate code and letting you to stay
focused on your business problem
- generated [IDL](https://en.wikipedia.org/wiki/Interface_description_language) file for
your application
- generated client allowing to interact with your application from code written in
different languages and executed in different runtimes

---

> [!NOTE]
> The Sails library is published under the name `sails-rs` on `crates-io`.
>
> Versions "<= 0.2.0" are pinned to v1.4.2 of gear libs.

## Getting started

Add the following to your `Cargo.toml`
```toml
[dependencies]
sails-rs = "*"
gstd = { version = "*", features = ["debug"] }
```

And then in your `lib.rs`:

```rust
#![no_std]

use sails_rs::prelude::*;
use gstd::debug;

struct MyPing;

#[service]
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

#[program]
impl MyProgram {
    #[route("ping")]
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

The first one is *__service__* which is represented by an impl of some Rust struct
marked with the `#[service]` attribute. The service main responsibility is
implementing some aspect of application business logic.

A set of service's __public__ methods defined by the impl is essentially a set of
remote calls the service exposes to external consumers. Each such method working
over a `&mut self` is treated as a command changing some state, whereas each method
working over a `&self` is treated as a query keeping everything unchanged and
returning some data. Both types of methods can accept some parameters passed by a
client and can be synchronous or asynchronous. All the other service's methods and
associated functions are treated as implementation details and ignored. The code
generated behind the service by the `#[service]` attribute decodes an incoming
request message and dispatches it to the appropriate method based on the method's
name. On the method's completion, its result is encoded and returned as a response
to a caller.

```rust
#[service]
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


The second key concept is *__program__* which is similarly to the service represented
by an impl of some Rust struct marked with the `#[program]` attribute. The program
main responsibility is hosting one or more services and exposing them to the external
consumers.

A set of its associated __public__ functions returning `Self` are treated as application
constructors. These functions can accept some parameters passed by a client and can be
synchronous or asynchronous. One of them will be called once at the very beginning of
the application lifetime, i.e. when the application is loaded onto the network. The
returned program instance will live until the application stays on the network. If there
are no such methods discovered, a default one with the following signature will be generated:

```rust
pub fn default() -> Self {
    Self
}
```

A set of program's __public__ methods working over `&self` and having no other parameters
are treated as exposed service constructors and are called each time when an incoming
request message needs be dispatched to a selected service. All the other methods and
associated functions are treated as implementation details and ignored. The code
generated behind the program by the `#[program]` attribute receives an incoming request
message from the network, decodes it and dispatches it to a matching service for actual
processing. After that, the result is encoded and returned as a response to a caller.
Only one program is allowed per application.

```rust
#[program]
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


And the final key concept is message *__routing__*. This concept doesn't have a
mandatory representation in code, but can be altered by using the `#[route]`
attribute applied to those public methods and associated functions described above.
The concept itself is about rules for dispatching an incoming request message to
a specific service's method using service and method names. By default, every
service exposed via program is exposed using the name of the service constructor
method converted into *PascalCase*. For example:

```rust
#[program]
impl MyProgram {
    // The `MyPing` service is exposed as `PingSvc`
    pub fn ping_svc(&self) -> MyPing {
        ...
    }
}
```

This behavior can be changed by applying the `#[route]` attribute:

```rust
#[program]
impl MyProgram {
    // The `MyPing` service is exposed as `Ping`
    #[route("ping")] // The specified name will be converted into PascalCase
    pub fn ping_svc(&self) -> MyPing {
        ...
    }
}
```

The same rules are applicable to service method names:

```rust
#[service]
impl MyPing {
    // The `do_ping` method is exposed as `Ping`
    #[route("ping")]
    pub fn do_ping(&mut self) {
        ...
    }

    // The `ping_count` method is exposed as `PingCount`
    pub fn ping_count(&self) -> u64 {
        ...
    }
}
```

### Events

Sails offers a mechanism to emit events from your service while processing commands.
These events serve as a means to notify off-chain subscribers about changes in
the application state. In Sails, events are configured and emitted on a per-service
basis through the `events` argument of the `#[service]` attribute. They are defined
by a Rust enum, with each variant representing a separate event and its optional data.
Once a service declares that it emits events, the `#[service]` attribute automatically
generates the `notify_on` service method. This method can be called by the service
to emit an event. For example:

```rust
fn counter_mut() -> &'static mut u32 {
    static mut COUNTER: u32 = 0;
    unsafe { &mut COUNTER }
}

struct MyCounter;

#[derive(Encode, TypeInfo)]
enum MyCounterEvent {
    Incremented(u32),
}

#[service(events = MyCounterEvent)]
impl MyCounter {
    pub fn new() -> Self {
        Self
    }

    pub fn increment(&mut self) {
        *counter_mut() += 1;
        self.notify_on(MyCounterEvent::Incremented(*counter_mut())).unwrap();
    }

    // This method is generated by the `#[service]` attribute
    fn notify_on(&mut self, event: MyCounterEvent) -> Result<()> {
        ...
    }
}
```

It's important to note that, internally, events use the same mechanism as any other
message transmission in the Gear Protocol. This means an event is only published
upon the successful completion of the command that emitted it.

### Service Extending (Mixins)

A standout feature of Sails is its capability to extend (or mix in) existing services.
This is facilitated through the use of the `extends` argument in the `#[service]`
attribute. Consider you have Service `A` and Service `B`, possibly sourced from
external crates, and you aim to integrate their functionalities into a new
Service `C`. This integration would result in methods and events from Services `A`
and `B` being seamlessly incorporated into Service `C`, as if they were originally
part of it. In such a case, the methods available in Service `C` represent a combination
of those from Services `A` and `B`. Should a method name conflict arise, where both
Services `A` and `B` contain a method with the same name, the method from the service
specified first in the `extends` argument takes precedence. This strategy not only
facilitates the blending of functionalities but also permits the overriding of specific
methods from the original services by defining a method with the same name in the
new service. With event names, conflicts are not allowed. Unfortunately, the IDL
generation process is the earliest when this can be reported as an error. For example:

```rust
struct MyServiceA;

#[service]
impl MyServiceA {
    pub fn do_a(&mut self) {
        ...
    }
}

struct MyServiceB;

#[service]
impl MyServiceB {
    pub fn do_b(&mut self) {
        ...
    }
}

struct MyServiceC;

#[service(extends = [MyServiceA, MyServiceB])]
impl MyServiceC {
    // New method
    pub fn do_c(&mut self) {
        ...
    }

    // Overridden method from MyServiceA
    pub fn do_a(&mut self) {
        ...
    }

    // do_b from MyServiceB will exposed due to the extends argument
}
```

### Payload Encoding

An application written with Sails uses [SCALE Codec](https://github.com/paritytech/parity-scale-codec) to encode/decode data
at its base.

Every incoming request message is expected to have the following format:

__|__ *SCALE encoded service name* __|__ *SCALE encoded method name* __|__ *SCALE encoded parameters* __|__

Every outgoing response message has the following format:

__|__ *SCALE encoded service name* __|__ *SCALE encoded method name* __|__ *SCALE encoded result* __|__

Every outgoing event message has the following format:

__|__ *SCALE encoded service name* __|__ *SCALE encoded event name* __|__ *SCALE encoded event data* __|__

### Client

Having robust interaction capabilities with applications is crucial. Sails offers
several options for interaction.

Firstly, it supports manual interaction using the Gear Protocol. You can use:
- The `msg::send` functions from the `gstd` crate to interact between applications.
- The `gclient` crate to interact from off-chain code with an on-chain application.
- The `@gear-js/api` library to interact with your program from JavaScript.

All you need to do is compose a byte payload according to the layout outlined in the
[Payload Encoding](#payload-encoding) section and send it to the application.

Thanks to the generated IDL, Sails provides a way to interact with your application
using generated clients with an interface similar to the one exposed by latter in
a clearer way. Currently, Sails can generate client code for Rust and TypeScript.

When it comes to Rust, there are two options:
- Use generated code that can encode and decode byte payloads for you, allowing you
to continue using functions that send raw bytes.
- Use fully generated code that can interact with your application in an RPC style.

For TypeScript see [generated clients](js/README.md#generate-library-from-idl)
documentation.

Say you have an application that exposes a service `MyService` with a command `do_something`:

```rust
struct Output {
    m1: u32,
    m2: String,
}

#[service]
impl MyService {
    pub fn do_something(&mut self, p1: u32, p2: String) -> Output {
        ...
    }
}

#[program]
impl MyProgram {
    pub fn my_service(&self) -> MyService {
        MyService::new()
    }
}
```

Then in a client application provided the code generation happens in Rust build script,
you can use the generated code like this (option 1):

```rust
include!(concat!(env!("OUT_DIR"), "/my_service.rs"));

fn some_client_code() {
    let call_payload = my_service::io::DoSomething::encode_call(42, "Hello".to_string());
    let reply_bytes = gstd::msg::send_bytes_for_reply(target_app_id, call_payload, 0, 0).await.unwrap();
    let reply = my_service::io::DoSomething::decode_reply(&reply_bytes).unwrap();
    let m1 = reply.m1;
    let m2 = reply.m2;
}
```

Or like this (option 2):

```rust
include!(concat!(env!("OUT_DIR"), "/my_service.rs"));

fn some_client_code() {
    let mut my_service = MyService::new(remoting); // remoting is an abstraction provided by Sails
    let reply_ticket = client.do_something(42, "Hello".to_string())
        .with_reply_deposit(42)
        .publish(target_app_id)
        .await.unwrap();
    let reply = reply_ticket.reply().await.unwrap();
    let m1 = reply.m1;
    let m2 = reply.m2;
}
```

The second option provides you with an option to have your code testable as the generated
code depends on the trait which can be easily mocked.

When it comes to TypeScript, `sails-js` library can be used to interact with the program. Check out [`sails-js` documentation](js/README.md) for more details.

## Examples

You can find all examples <a href="examples/">here</a> along with some descriptions
provided at the folder level. You can also find some explanatory comments in the code.
Here is a brief overview of features mentioned above and showcased by the examples:

### Exposing Services via Program

The examples are composed on a principle of a few programs exposing several services.
See [DemoProgram](/examples/demo/app/src/lib.rs) which demonstrates this, including
the use of program's multiple constructors and the `#[route]` attribute for one of
the exposed services. The example also includes Rust [build script](/examples/demo/app/build.rs)
building the program as a WASM app ready for loading onto Gear network.

### Basic Services

There are a couple of services which demonstrate basic service structure exposing
some primitive methods operating based on input parameters and returning some
results. They serve as an excellent starting point for developing your services. See
[Ping](examples/demo/app/src/ping) and [ThisThat](examples/demo/app/src/this_that/)
services. The latter, in addition to the basics, showcases the variety of types
which can be used as parameters and return values in service methods.

### Working with Data

In the real world, almost all apps work with some form of data, and apps developed
using Sails are no exception. As discussed in the [Application](#application) section,
services are instantiated for every incoming request message, indicating that these
services are stateless. However, there are a few ways to enable your services to
maintain some state. In this case, the state will be treated as external to the service.

The most recommended way is demonstrated in the [Counter](examples/demo/app/src/counter/)
service, where the data is stored as part of the program and passed to the service
via `RefCell`. The service module merely defines the shape of the data but requires
the data itself to be passed from the outside. This option provides you with full
flexibility and allows you to unit test your services in a multi-threaded environment,
ensuring the tests do not affect each other.

Another method is illustrated in the [RmrkCatalog](examples/rmrk/catalog/app/src/services/)
and [RmrkResource](examples/rmrk/resource/app/src/services/) services, where the data
is stored in static variables within the service module. This strategy ensures that the
state remains completely hidden from the outside, making the service entirely self-contained.
However, this approach is not ideal for unit testing in a multi-threaded environment
because each test can potentially influence others. Additionally, it's important
not to overlook calling the service's `seed` method before its first use.

You can also explore other approaches, such as making a service require `&'a mut` for its
data (which makes the service non-clonable), or using `Cell` (which requires data copying,
incurring additional costs).

In all scenarios, except when using `Cell`, it's crucial to consider the static nature
of data, especially during asynchronous calls within service methods. This implies that
data accessed before initiating an asynchronous call might change by the time the call
completes. See the [RmrkResource](examples/rmrk/resource/app/src/services/) service's
`add_part_to_resource` method for more details.

### Events

You can find an example of how to emit events from your service in the [Counter](examples/demo/app/src/counter/)
and [RmrkResource](examples/rmrk/resource/app/src/services/) services.

### Service Extending (Mixins)

An example of service extension is demonstrated with the [Dog](examples/demo/app/src/dog/)
service, which extends the [Mammal](examples/demo/app/src/mammal/) service from
the same crate and the [Walker](examples/demo/walker/src/) service from a different crate.
The service being extended must implement the `Clone` trait, while the extending
service must implement the `AsRef` trait for the service being extended.

### Using Generated Clients from Rust

The [Demo Client](/examples/demo/client/src/) crate showcases how to generate client
code from an IDL file as a separate Rust crate. Alternatively, you can use the same
approach directly in your application crate. See [Rmrk Resource](/examples/rmrk/resource/app/build.rs).

You can find various examples of how to interact with the application using the
generated client code in [Demo Tests](/examples/demo/app/tests/gtest.rs). Check
the comments in the code for more details.

Since the generated code is the same for all environments, whether it is an interaction
from tests or from another application, the techniques for these interactions are the same.
You can find an example of the interaction from an application in the
[Rmrk Resource](/examples/rmrk/resource/app/src/services/mod.rs) service's `add_part_to_resource`
method.

Bear in mind that working with the generated client requires the `sails_rs` crate to
be in dependencies.

##

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Sails by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
