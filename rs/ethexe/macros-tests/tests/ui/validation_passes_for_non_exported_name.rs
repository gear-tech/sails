use sails_rs::prelude::*;

#[derive(Default)]
struct MyProgram;

struct MyService;

#[program]
impl MyProgram {
    pub fn my_service(&self) -> MyService {
        MyService
    }
}

#[service]
impl MyService {
    pub fn address(&mut self) -> u32 {
        42
    }

    #[export]
    pub fn some_value(&mut self) -> u32 {
        42
    }
}

fn main() {}
