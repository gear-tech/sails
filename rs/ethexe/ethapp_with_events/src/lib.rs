#![no_std]

use sails_rs::prelude::*;

#[derive(Encode, TypeInfo, EvmEvent)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Events {
    DoThisEvent(#[indexed] u32, String),
}

pub struct MyProgram;

#[program]
impl MyProgram {
    pub fn svc1(&self) -> SomeService {
        SomeService
    }
}

pub struct SomeService;

#[service(events = Events)]
impl SomeService {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
        self.notify_on(Events::DoThisEvent(p1, p2)).unwrap();
        p1
    }
    pub fn this(&self, p1: bool) -> bool {
        p1
    }
}
