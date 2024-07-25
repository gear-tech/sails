use sails_macros::gprogram;

struct MyService;

struct MyProgram;

#[gprogram]
impl MyProgram {
    #[route("")]
    pub fn service1(&self) -> MyService {
        MyService
    }
}

#[tokio::main]
async fn main() {}
