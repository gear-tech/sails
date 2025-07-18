use sails_macros::program;

struct MyService;

struct MyProgram;

#[program]
impl MyProgram {
    #[export(route = "svc")]
    pub fn service1(&self) -> MyService {
        MyService
    }

    #[export(route = "svc")]
    pub fn service2(&self) -> MyService {
        MyService
    }
}

#[tokio::main]
async fn main() {}
