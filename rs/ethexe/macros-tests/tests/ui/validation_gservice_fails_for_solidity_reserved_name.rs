use sails_rs::prelude::*;

#[derive(Default)]
struct MyProgram;

struct MyService;

#[program]
impl MyProgram {}

#[service]
impl MyService {
    #[export]
    pub fn new(&mut self) -> u32 {
        42
    }
}

fn main() {}
