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
    use svc1::MyService1 as RenamedMyService1;

    struct MyService2;

    #[service(extends = [svc1::MyService1, RenamedMyService1])]
    impl MyService2 {
        #[export]
        pub fn svc2(&self) -> bool {
            true
        }
    }

    impl From<MyService2> for (svc1::MyService1, RenamedMyService1) {
        fn from(_: MyService2) -> Self {
            (svc1::MyService1, RenamedMyService1)
        }
    }
}

#[tokio::main]
async fn main() {}