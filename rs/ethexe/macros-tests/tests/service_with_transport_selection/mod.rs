use sails_rs::prelude::*;

pub struct MyService;

#[sails_rs::service]
impl MyService {
    #[export(scale)]
    pub fn scale_only(&self, p1: u32) -> u32 {
        p1 + 1
    }

    #[export(ethabi)]
    pub fn ethabi_only(&self, p1: u32) -> u32 {
        p1 + 2
    }

    #[export(scale, ethabi)]
    pub fn dual(&self, p1: u32) -> u32 {
        p1 + 3
    }
}
