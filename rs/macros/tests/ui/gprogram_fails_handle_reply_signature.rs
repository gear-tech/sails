use sails_macros::program;

struct MyProgram;

#[program]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }

    #[handle_reply]
    fn handle_reply() {}
}

#[tokio::main]
async fn main() {}
