#![no_std]
#![allow(unexpected_cfgs)]

extern crate alloc;

use sails_rs::{gstd, prelude::*};

#[event]
#[derive(Clone, Debug, PartialEq, Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Events {
    Start,
    End,
}

#[derive(Default)]
pub struct Service(u8);

impl Service {
    pub fn new(byte: u8) -> Self {
        Self(byte)
    }
}

#[service(events = Events)]
impl Service {
    /// Send `Start` event
    /// Then await for reply from source
    /// Send `End` event
    #[export]
    pub async fn foo(&mut self) {
        let source = Syscall::message_source();
        self.emit_event(Events::Start).unwrap();
        let _res = gstd::msg::send_for_reply(source, self.0, 0, 0)
            .unwrap()
            .await;
        self.emit_event(Events::End).unwrap();
    }
}

#[derive(Default)]
pub struct Program;

#[program]
impl Program {
    pub fn new() -> Self {
        Self
    }

    pub fn foo(&self) -> Service {
        Service::new(1)
    }

    pub fn bar(&self) -> Service {
        Service::new(2)
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "sails-meta-dump")))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "sails-meta-dump")))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
