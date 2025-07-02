# Sails &emsp;

`Sails` is a library for bringing your experience of writing applications utilizing
[Gear Protocol](https://gear-tech.io/) to the next level of simplicity and
clarity. It deals with things like:
- eliminating the necessity of writing some low-level boilerplate code and letting
  you to stay focused on your business problem
- generated [IDL](https://en.wikipedia.org/wiki/Interface_description_language) file
  for your application
- generated client allowing to interact with your application from code written in
  different languages and executed in different runtimes

> **NOTE**
>
> The `Sails` library is published under the name `sails-rs` on `crates-io`.
>

## Getting started

Either use `Sails` CLI:
```bash
cargo install sails-cli
cargo sails program my-ping
```

Or add the following to your `Cargo.toml`
```toml
[dependencies]
sails-rs = "*"

[build-dependencies]
sails-rs = { version = "*", features = ["wasm-builder"] }
```

And then in your `lib.rs`:

```rust
#![no_std]

use sails_rs::{gstd::debug, prelude::*};

struct MyPing;

impl MyPing {
    pub const fn new() -> Self {
        Self
    }
}

#[service]
impl MyPing {
    #[export]
    pub async fn ping(&mut self) -> bool {
        debug!("Ping called");
        true
    }
}

#[derive(Default)]
struct MyProgram;

#[program]
impl MyProgram {
    #[export(route = "ping")]
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

`Sails` architecture for applications is based on a few key concepts.

The first one is *__service__* which is represented by an impl of some Rust struct
marked with the `#[service]` attribute. The service main responsibility is
implementing some aspect of application business logic.

A set of service's __public__ methods with `#[export]` attribute defined by the impl
is essentially a set of remote calls the service exposes to external consumers.
Each such method working over a `&mut self` is treated as a command changing some state, whereas each method
working over a `&self` is treated as a query keeping everything unchanged and
returning some data. Both types of methods can accept some parameters passed by a
client and can be synchronous or asynchronous. All other methods and associated functions of the service
are considered implementation details and are not accessible through remote calls.

The code created behind the service using the `#[service]` attribute declares a new `Exposure` structure
and moves the implementation methods into it.
The `Exposure` struct implements the `Deref<Target = TService>` and `DerefMut<Target = TService>`
traits enabling transparent access to the underlying service instance.
Additionally, the generated code handles incoming request messages by decoding them and dispatching them
to the corresponding service method based on the method's name.
On the method's completion, its result is encoded and returned as a response to a caller.

> **NOTE**
>
> In some cases, a command might need to return a certain amount of tokens (value) from
> the application's balance to the caller's one. This can be done via using a dedicated
> type, `CommandReply<T>`.

Sometimes it is convenient to have a method that returns the `Result<T, E>` type,
but not expose it to clients. This allows using the `?` operator
in the method body. For this purpose, you can use the `#[export]` attribute macro with
the `unwrap_result` parameter.

```rust
#[service]
impl MyService {
    // This is a command
    #[export]
    pub fn do_something(&mut self, p1: u32, p2: String) -> &'static [u8] {
        ...
    }

    // This is a command returning value along with the result
    #[export]
    pub fn withdraw(&mut self, amount: u64) -> CommandReply<()> {
        CommandReply::new(()).with_value(amount)
    }

    // This is a command returning `()` or panicking
    #[export(unwrap_result)]
    pub fn do_somethig_with_unwrap_result(&mut self, amount: u64) -> Result<(), String> {
        do_somethig_returning_result()?;
        Ok(())
    }

    // This is a query
    #[export]
    pub fn something(&self, p1: Option<bool>) -> String {
        ...
    }

    // This is a inner method, not accesible via remote calls
    pub fn do_something_inner(&mut self, p1: u32, p2: String) -> &'static [u8] {
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
mandatory representation in code, but can be altered by using the `#[export]`
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

This behavior can be changed by applying the `#[export]` attribute with `route` parameter:

```rust
#[program]
impl MyProgram {
    // The `MyPing` service is exposed as `Ping`
    #[export(route = "ping")] // The specified name will be converted into PascalCase
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
    #[export(route = "ping")]
    pub fn do_ping(&mut self) {
        ...
    }

    // The `ping_count` method is exposed as `PingCount`
    #[export]
    pub fn ping_count(&self) -> u64 {
        ...
    }
}
```

### Events

`Sails` offers a mechanism to emit events from your service while processing commands.
These events serve as a means to notify off-chain subscribers about changes in
the application state. In `Sails`, events are configured and emitted on a per-service
basis through the `events` argument of the `#[service]` attribute. They are defined
by a Rust enum, with each variant representing a separate event and its optional data.
Once a service declares that it emits events, the `#[service]` attribute automatically
generates the `emit_event` service method. This method can be called by the service
to emit an event. For example:

```rust
fn counter_mut() -> &'static mut u32 {
    static mut COUNTER: u32 = 0;
    unsafe { &mut COUNTER }
}

struct MyCounter;

impl MyCounter {
    pub fn new() -> Self {
        Self
    }
}


#[derive(Encode, TypeInfo)]
enum MyCounterEvent {
    Incremented(u32),
}

#[service(events = MyCounterEvent)]
impl MyCounter {
    #[export]
    pub fn increment(&mut self) {
        *counter_mut() += 1;
        self.emit_event(MyCounterEvent::Incremented(*counter_mut())).unwrap();
    }

    // This method is generated by the `#[service]` attribute
    fn emit_event(&self, event: MyCounterEvent) -> Result<()> {
        ...
    }
}
```

It's important to note that, internally, events use the same mechanism as any other
message transmission in the [Gear Protocol](https://gear-tech.io/). This means an
event is only published upon the successful completion of the command that emitted it.

### Service Extending (Mixins)

A standout feature of `Sails` is its capability to extend (or mix in) existing services.
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
    #[export]
    pub fn do_a(&mut self) {
        ...
    }
}

struct MyServiceB;

#[service]
impl MyServiceB {
    #[export]
    pub fn do_b(&mut self) {
        ...
    }
}

struct MyServiceC;

#[service(extends = [MyServiceA, MyServiceB])]
impl MyServiceC {
    // New method
    #[export]
    pub fn do_c(&mut self) {
        ...
    }

    // Overridden method from MyServiceA
    #[export]
    pub fn do_a(&mut self) {
        ...
    }

    // do_b from MyServiceB will exposed due to the extends argument
}
```

### Payload Encoding

An application written with `Sails` uses [SCALE Codec](https://github.com/paritytech/parity-scale-codec) to encode/decode data
at its base.

Every incoming request message is expected to have the following format:

__|__ *SCALE encoded service name* __|__ *SCALE encoded method name* __|__ *SCALE encoded parameters* __|__

Every outgoing response message has the following format:

__|__ *SCALE encoded service name* __|__ *SCALE encoded method name* __|__ *SCALE encoded result* __|__

Every outgoing event message has the following format:

__|__ *SCALE encoded service name* __|__ *SCALE encoded event name* __|__ *SCALE encoded event data* __|__

### Syscalls

During message processing, `Sails` program can obtain details of incoming messages and current execution environment by using `Syscall` struct which provides a collection of methods that abstract lower-level operations ([`message_source`], [`message_size`], [`message_id`], [`message_value`], [`reply_to`], [`reply_code`], [`signal_from`], [`signal_code`], [`program_id`], etc.).

These methods are essential for enabling on-chain applications to interact with the Gear runtime in a consistent manner. Depending on the target environment, different implementations are provided:
- For the WASM target, direct calls are made to `gstd::msg` and `gstd::exec` to fetch runtime data.
- In standard (`std`) environments, a mock implementation uses thread-local state for testing purposes.
- In `no_std` configurations without the `std` feature and and not WASM target, the functions are marked as unimplemented.

### Client

Having robust interaction capabilities with applications is crucial. `Sails` offers
several options for interaction.

Firstly, it supports manual interaction using the [Gear Protocol](https://gear-tech.io/).
You can use:
- The `msg::send` functions from the `gstd` crate to interact between applications.
- The `gclient` crate to interact from off-chain code with an on-chain application.
- The `@gear-js/api` library to interact with your program from JavaScript.

All you need to do is compose a byte payload according to the layout outlined in the
[Payload Encoding](#payload-encoding) section and send it to the application.

Thanks to the generated IDL, `Sails` provides a way to interact with your application
using generated clients with an interface similar to the one exposed by latter in
a clearer way. Currently, `Sails` can generate client code for Rust and TypeScript.

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
    #[export]
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

Then, in a client application, provided the code generation happens in a Rust build script,
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
        .send(target_app_id)
        .await.unwrap();
    let reply = reply_ticket.recv().await.unwrap();
    let m1 = reply.m1;
    let m2 = reply.m2;
}
```

The second option provides you with an option to have your code testable, as the generated
code depends on the trait which can be easily mocked.

As you may have noticed, the option 2 uses the concept of a `remoting` object, which needs
to be passed to the client instantiation code. This object should implement the `Remoting`
trait from the `sails-rs` crate. It abstracts the low-level communication details
between client and the application. The `sails-rs` crate provides three implementations of this
trait:
- `sails_rs::gstd::calls::GStdRemoting` should be used when the client code is executed
  as a part of another on-chain application.
- `sails_rs::gclient::calls::GClientRemoting` should be used when the client code is executed
  as a part of an off-chain application.
- `sails_rs::gstd::calls::GTestRemoting` should be used when the client code is executed
  as a part of a tests utilizing the `gtest` crate.

When it comes to TypeScript, `sails-js` library can be used to interact with the program. Check out [`sails-js` documentation](js/README.md) for more details.

### Writing Sagas (Advanced)

Occasionally, you may need to design a system where a business transaction spans multiple
applications. In other words, a single business transaction may involve several local
transactions within different applications. This challenge is typically addressed using
a pattern called `Saga`. You can find detailed documentation about this pattern [here](https://microservices.io/patterns/data/saga.html)
and implementation guidelines [here](https://livebook.manning.com/book/microservices-patterns/chapter-4/142).

The process of addressing this issue in applications built with `Sails` is similar, but only
the orchestration approach can be used, as `Sails` applications cannot catch events from each
other. Additionally, it is important to handle infrastructure errors that can arise,
particularly those related to the [Gear Protocol](https://gear-tech.io/) and its concept of gas.

Let's refer to the guidelines and explore the nuances involved.

First, it is important to note that all the errors mentioned in the guidelines are business
errors — only business errors can trigger compensation actions. In contrast, infrastructure
errors are expected to be resolved through retries. This applies to both retriable and
compensatable transactions. The key difference is that the former should never return a
business error.

Normally, these retries are handled programmatically. However, when a `RunOutOfGas` error is
received from another application, retrying is not an option. The only solution in this case
is to propagate the error to the top-level caller, who will then need to attach more funds
and attempt the entire business transaction again.

For retries to function correctly, local transactions must be idempotent (i.e., safe to retry
without causing side effects, such as sending duplicate events). This can be ensured by
assigning a unique identifier to each business transaction, which is passed to all actions
within the `Saga`, whether it’s the initial attempt or a retry.

The Saga’s state must track this identifier to skip actions that were completed in previous
attempts. Similarly, the actions themselves should recognize this identifier to prevent
repeating changes that have already been made.

Another issue to consider is the possibility of receiving a `Timeout` error while waiting for
a response from another application. In some systems, timeouts can be treated as business errors,
triggering compensation. However, if the timeout is caused by infrastructure issues (e.g.,
network congestion), it can be handled similarly to the `RunOutOfGas` error.

There are optimization opportunities in such cases. Since the [Gear Protocol](https://gear-tech.io/)
guarantees that the caller will eventually receive a response (successful or not), one simple
optimization is to increase the number of blocks allowed for waiting on a response. On the
[Vara network](https://vara.network/), this value is set to 100 blocks by default. Increasing
it to 10,000 blocks would mitigate most network load issues without significantly raising
transaction costs. This adjustment can be made using the [with_wait_up_to](rs/src/gstd/calls.rs#L22) method.

Another option is to use the [with_reply_hook](rs/src/gstd/calls.rs#L51) method, which involves
additional logic to manage the `Saga`’s state. The reply hook can be triggered while the main
code handles the `Timeout` error, allowing the caller to initiate another attempt. During this
process, the timed-out action can be marked as completed in the `Saga`’s state, preventing it
from being re-executed. However, it's worth considering whether this added complexity is
necessary, as retrying an already completed idempotent action is harmless and only slightly
increases the overall cost.

To summarize:
- Implement an orchestrating `Saga` (orchestrator application) by maintaining its state.
- Design calls to other applications as either compensatable or retriable transactions.
- Record a list of actions needed to execute the transactions in the `Saga`’s state, along
  with the status of each action (e.g., not executed, succeeded, failed).
- Ensure that each transaction's actions are implemented in an idempotent manner.
- Prepare for 2 key infrastructure errors: `RunOutOfGas` and `Timeout`. The simplest approach
  is to propagate these errors to the top-level caller for retries.
  - For `Timeout` errors, optimize by increasing the number of blocks allowed for waiting on a response.
- Keep in mind that every call to an application will eventually yield a response.

## Examples

You can find all examples <a href="examples/">here</a> along with some descriptions
provided at the folder level. You can also find some explanatory comments in the code.
Here is a brief overview of features mentioned above and showcased by the examples:

### Exposing Services via Program

The examples are composed on a principle of a few programs exposing several services.
See [DemoProgram](/examples/demo/app/src/lib.rs) which demonstrates this, including
the use of program's multiple constructors and the `#[export]` attribute for one of
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
using `Sails` are no exception. As discussed in the [Application](#application) section,
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
The extending service must implement the `Into` trait for the service being extended.

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
for inclusion in `Sails` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
