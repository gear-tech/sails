use sails_macros::gprogram;

struct MyService;

struct MyProgram;

#[gprogram]
impl MyProgram {
    #[groute("svc1/")]
    pub fn service1(&self) -> MyService {
        MyService
    }
}

#[tokio::main]
async fn main() {}
