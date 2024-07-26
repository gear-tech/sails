use sails_macros::program;

struct MyProgram;

#[program]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }
}

#[program]
impl MyProgram {
    pub fn default() -> Self {
        Self
    }
}

#[tokio::main]
async fn main() {}
