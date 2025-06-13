#![no_std]
use sails_rs::gstd::{EventEmitter, ExposureWithEvents};

/// Service Events
#[sails_rs::event]
#[derive(Clone, Debug, PartialEq, sails_rs::Encode, sails_rs::TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Events {
    DoThisEvent {
        /// Some u32 value
        #[indexed]
        p1: u32,
        p2: sails_rs::String,
    },
}

pub struct MyProgram;

#[sails_rs::program]
impl MyProgram {
    pub fn svc1(&self) -> SomeService {
        SomeService
    }

    pub fn svc2(&self) -> SomeService2 {
        let svc1_emitter = self.svc1().emitter();
        SomeService2 { svc1_emitter }
    }
}

pub struct SomeService;

#[sails_rs::service(events = Events)]
impl SomeService {
    pub async fn do_this(&mut self, p1: u32, p2: sails_rs::String) -> u32 {
        self.emit_eth_event(Events::DoThisEvent { p1, p2 }).unwrap();
        p1
    }

    pub fn this(&self, p1: bool) -> bool {
        p1
    }
}

pub struct SomeService2 {
    svc1_emitter: EventEmitter<Events>,
}

#[sails_rs::service]
impl SomeService2 {
    pub async fn do_this(&mut self, p1: u32, p2: sails_rs::String) -> u32 {
        // Emit EthEvent via Svc1 Exposure
        self.svc1_emitter
            .emit_eth_event(Events::DoThisEvent { p1, p2: p2.clone() })
            .unwrap();
        // Emit gear event via Svc1 Exposure
        self.svc1_emitter
            .emit_event(Events::DoThisEvent { p1, p2 })
            .unwrap();
        p1
    }
}
