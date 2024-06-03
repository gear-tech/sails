# Sails &emsp; [![Build Status]][actions]

Sails is a library for bringing your experience of writing programs utilizing
Gear protocol to the next level. It deals with things like:
- eliminating
the necessity of writing some low-level boilerplate code and letting you to stay
focused on your bussiness problem
- generated [IDL](https://en.wikipedia.org/wiki/Interface_description_language) file for
your program
- generated client allowing to interact with your program from code written in
different languages and executed in different runtimes

---

## Getting started

Add the following to your `Cargo.toml`
```toml
[dependencies]
sails-rtl = { git = "https://github.com/gear-tech/sails" }
gstd = "{ git = "https://github.com/gear-tech/gear", features = ["debug"] }"
```

And then:

```rust
#![no_std]

use gstd::debug;
use sails_rtl::{gstd::gservice, prelude::*};

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

#[derive(Default)]
struct MyProgram;

#[gprogram]
impl MyProgram {
    #[groute("ping")]
    pub fn ping_svc(&self) -> MyService {
        MyService::new()
    }
}
```

## Details

Bla-bla

### Concepts

Bla-bla

### Examples

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
