use sails_macros::gprogram;

struct MyProgram;

#[gprogram]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }
}

#[gprogram]
impl MyProgram {
    pub fn default() -> Self {
        Self
    }
}

#[tokio::main]
async fn main() {}
