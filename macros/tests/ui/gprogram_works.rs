use sails_macros::gprogram;

struct MyProgram;

#[gprogram]
impl MyProgram {
    pub fn new(config: u32) -> Self {
        let _config = config;
        Self
    }

    pub fn service1(&self) -> svc::MyService {
        svc::MyService
    }
}

pub mod svc {
    #![allow(dead_code)]

    use sails_macros::gservice;

    pub struct MyService;

    #[gservice]
    impl MyService {
        pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
            format!("{p1}: ") + &p2
        }

        pub fn this(&self, p1: bool) -> bool {
            !p1
        }
    }
}

#[tokio::main]
async fn main() {
    let _prg = MyProgram::new(42);
    // let _s  = &prg.service1(); // panic here (!)
}
