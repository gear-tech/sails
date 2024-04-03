Sails is a library for organizing inter-program transport in Gear protocol.

## Getting started

Add the following to your `Cargo.toml`
```
[dependencies]
sails-macros = { git = "https://github.com/gear-tech/sails" }
gstd = { git = "https://github.com/gear-tech/gear"}
parity-scale-codec = { version = "3.6", default-features = false }
scale-info = { version = "2.10", default-features = false }
sails-idl-meta = { git = "https://github.com/gear-tech/sails" }
```

And then:

```rust
#![no_std]

use gstd::{debug, prelude::*};
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
