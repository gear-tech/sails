#![no_std]

use sails_rs::prelude::*;

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

#[derive(Default)]
pub struct MyProgram;

#[sails_rs::program]
impl MyProgram {
    pub fn svc1(&self) -> SomeService {
        SomeService
    }

    pub fn svc2(&self) -> SomeService2 {
        let svc1 = self.svc1();
        SomeService2 { svc1 }
    }
}

pub struct SomeService;

#[sails_rs::service(events = Events)]
impl SomeService {
    #[sails_rs::export]
    pub async fn do_this(&mut self, p1: u32, p2: sails_rs::String) -> u32 {
        let r1 = p1 * 2;
        self.emit_eth_event(Events::DoThisEvent {
            p1: r1,
            p2: format!("{p2}: greetings from sails #1"),
        })
        .unwrap();
        r1
    }

    #[sails_rs::export]
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

pub struct SomeService2 {
    svc1: SomeServiceExposure<SomeService>,
}

#[sails_rs::service]
impl SomeService2 {
    #[sails_rs::export]
    pub async fn do_this(&mut self, p1: u32, p2: sails_rs::String) -> u32 {
        let r1 = p1 * 2;
        let r2 = format!("{p2}: greetings from sails #2");
        // Emit EthEvent via Svc1 Exposure
        self.svc1
            .emit_eth_event(Events::DoThisEvent {
                p1: r1,
                p2: r2.clone(),
            })
            .unwrap();
        // Emit gear event via Svc1 Exposure
        self.svc1
            .emit_event(Events::DoThisEvent { p1: r1, p2: r2 })
            .unwrap();
        r1
    }
}
