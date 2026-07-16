#![no_std]
#![allow(unused_assignments)]

use sails::prelude::*;

/// Service Events
#[sails::event]
#[sails_type]
#[derive(Clone, Debug, PartialEq)]
pub enum Events {
    DoThisEvent {
        /// Some u32 value
        #[indexed]
        p1: u32,
        p2: sails::String,
    },
}

#[derive(Default)]
pub struct MyProgram;

#[sails::program]
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

#[sails::service(events = Events)]
impl SomeService {
    #[sails::export]
    pub async fn do_this(&mut self, p1: u32, p2: sails::String) -> u32 {
        let r1 = p1.checked_mul(2).expect("failed to multiply");
        self.emit_eth_event(Events::DoThisEvent {
            p1: r1,
            p2: format!("{p2}: greetings from sails #1"),
        })
        .unwrap();
        r1
    }

    #[sails::export]
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

pub struct SomeService2 {
    svc1: SomeServiceExposure<SomeService>,
}

#[sails::service]
impl SomeService2 {
    #[sails::export]
    pub async fn do_this(&mut self, p1: u32, p2: sails::String) -> u32 {
        let r1 = p1.checked_mul(2).expect("failed to multiply");
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
