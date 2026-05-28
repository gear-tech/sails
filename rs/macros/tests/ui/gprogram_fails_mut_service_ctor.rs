use sails_macros::program;

struct MyService;

struct MyProgram;

#[program]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }

    pub fn service(&mut self) -> MyService {
        MyService
    }
}

#[tokio::main]
async fn main() {}
