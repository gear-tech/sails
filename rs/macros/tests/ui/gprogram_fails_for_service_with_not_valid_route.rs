use sails_macros::program;

struct MyService;

struct MyProgram;

#[program]
impl MyProgram {
    #[export(route = "svc1/")]
    pub fn service1(&self) -> MyService {
        MyService
    }
}

#[tokio::main]
async fn main() {}
