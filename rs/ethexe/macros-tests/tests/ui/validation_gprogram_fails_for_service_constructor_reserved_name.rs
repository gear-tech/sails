use sails_rs::prelude::*;

#[derive(Default)]
struct MyProgram;

#[derive(Default)]
struct MyService;

#[program]
impl MyProgram {
    pub fn function(&self) -> MyService {
        MyService::default()
    }
}

#[service]
impl MyService {
    #[export]
    pub fn some_method(&self) -> bool {
        true
    }
}

fn main() {}
