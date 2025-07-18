use sails_rs::prelude::*;

pub(super) struct MyService;

// Service
#[service]
impl MyService {
    #[export]
    pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    #[export]
    #[allow(unused)]
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[derive(Encode)]
pub(super) struct MyDoThisParams {
    pub(super) p1: u32,
    pub(super) p2: String,
}

pub(super) struct MyOtherService;

// Service with different name
#[service]
impl MyOtherService {
    #[export]
    pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    #[export]
    #[allow(unused)]
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

pub mod yet_another_service {
    use super::*;
    pub struct MyService;
    // Service with duplicate name in another module
    #[service]
    impl MyService {
        #[export]
        pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
            format!("{p1}: ") + &p2
        }

        #[export]
        #[allow(unused)]
        pub fn this(&self, p1: bool) -> bool {
            !p1
        }
    }
}
