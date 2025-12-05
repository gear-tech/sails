use sails_rs::prelude::*;

struct MyProgram;

struct MyService;

#[program]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }

    pub fn my_service(&self) -> MyService {
        MyService
    }
}

#[service]
impl MyService {
    #[export]
    pub fn do_something(&mut self) -> u32 {
        42
    }
}

fn main() {}
