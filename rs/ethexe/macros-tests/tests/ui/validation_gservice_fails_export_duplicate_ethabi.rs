use sails_rs::prelude::*;

struct SomeService;

#[sails_rs::service]
impl SomeService {
    #[export(ethabi, ethabi)]
    pub fn do_this(&self) -> u32 {
        42
    }
}

fn main() {}
