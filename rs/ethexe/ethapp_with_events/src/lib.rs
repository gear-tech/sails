#![no_std]

#[derive(sails_rs::Encode, sails_rs::TypeInfo, sails_rs::EthEvent)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Events {
    DoThisEvent(#[indexed] u32, sails_rs::String),
}

pub struct MyProgram;

#[sails_rs::program]
impl MyProgram {
    pub fn svc1(&self) -> SomeService {
        SomeService
    }
}

pub struct SomeService;

#[sails_rs::service(events = Events)]
impl SomeService {
    pub async fn do_this(&mut self, p1: u32, p2: sails_rs::String) -> u32 {
        self.emit_eth_event(Events::DoThisEvent(p1, p2)).unwrap();
        p1
    }
    pub fn this(&self, p1: bool) -> bool {
        p1
    }
}
