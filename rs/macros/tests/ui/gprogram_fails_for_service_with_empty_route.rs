use sails_macros::program;

struct MyService;

struct MyProgram;

#[program]
impl MyProgram {
    #[route("")]
    pub fn service1(&self) -> MyService {
        MyService
    }
}

#[tokio::main]
async fn main() {}
