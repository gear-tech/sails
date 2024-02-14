Sails is a library for organizing inter-program transport in Gear protocol.

## Getting started

Add the following to your `Cargo.toml`
```
[dependencies]
sails = { git = "https://github.com/gear-tech/sails" }
sails_macros = { git = "https://github.com/gear-tech/sails" }
```

And then:

```rust
use sails_macros::gservice;

struct MyService;

#[gservice]
impl MyService {
    pub const fn new() -> Self {
        Self
    }

    pub async fn ping(&mut self) -> bool {
        debug!("Ping called");
        true
    }
}
```

## License

`sails` is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0), at your choice.

See LICENSE-APACHE, and LICENSE-MIT for details.
