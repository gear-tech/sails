use sails_macros::program;

struct MyProgram;

#[program]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }

    #[handle_reply]
    pub fn handle_reply(&self) {}
}

#[tokio::main]
async fn main() {}
