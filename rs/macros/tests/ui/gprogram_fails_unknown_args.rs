use sails_macros::program;

struct MyProgram;

#[program(_handle_reply = my_handle_reply, handle_signal = my_handle_signal)]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }
}

fn my_handle_reply() {}

fn my_handle_signal() {}

#[tokio::main]
async fn main() {}
