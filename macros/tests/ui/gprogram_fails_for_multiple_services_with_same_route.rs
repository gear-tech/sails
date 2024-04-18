use sails_macros::gprogram;

struct MyService;

struct MyProgram;

#[gprogram]
impl MyProgram {
    #[groute("svc")]
    pub fn service1(&self) -> MyService {
        MyService
    }

    #[groute("svc")]
    pub fn service2(&self) -> MyService {
        MyService
    }
}

#[tokio::main]
async fn main() {}
