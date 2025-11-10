use sails_macros::{export, service};

mod svc1 {
    use super::*;
    pub struct MyService1;

    #[service]
    impl MyService1 {
        #[export]
        pub fn svc1(&self) -> bool {
            true
        }
    }
}

mod svc2 {
    use super::*;

    struct MyService2;

    #[service(extends = [svc1::MyService1, svc1::MyService1])]
    impl MyService2 {
        #[export]
        pub fn svc2(&self) -> bool {
            true
        }
    }
}

#[tokio::main]
async fn main() {}