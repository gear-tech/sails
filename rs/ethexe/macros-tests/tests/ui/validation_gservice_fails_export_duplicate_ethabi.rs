use sails::prelude::*;

struct SomeService;

#[sails::service]
impl SomeService {
    #[export(ethabi, ethabi)]
    pub fn do_this(&self) -> u32 {
        42
    }
}

fn main() {}
