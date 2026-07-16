use sails::prelude::*;

struct SomeService;

#[sails::service]
impl SomeService {
    #[export(scale, payable)]
    pub fn do_this(&self) -> u32 {
        42
    }
}

fn main() {}
