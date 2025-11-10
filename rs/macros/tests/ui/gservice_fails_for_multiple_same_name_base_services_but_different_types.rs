// TODO [future]: This must work on Sails binary protocol with new headers model

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
    pub struct MyService1;

    #[service]
    impl MyService1 {
        #[export]
        pub fn svc2(&self) -> bool {
            true
        }
    }
}

mod svc3 {
    use super::*;

    struct MyService3;

    #[service(extends = [svc1::MyService1, svc2::MyService1])]
    impl MyService3 {
        #[export]
        pub fn svc3(&self) -> bool {
            true
        }
    }

    impl From<MyService3> for (svc1::MyService1, svc2::MyService1) {
        fn from(_: MyService3) -> Self {
            (svc1::MyService1, svc2::MyService1)
        }
    }
}

#[tokio::main]
async fn main() {}