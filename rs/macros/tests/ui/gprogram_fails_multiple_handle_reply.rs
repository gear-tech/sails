use sails_macros::program;

struct MyProgram;

#[program]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }

    #[handle_reply]
    fn handle_reply(&self) {}

    #[handle_reply]
    fn handle_reply_2(&self) {}
}

#[tokio::main]
async fn main() {}
