#![no_std]
use sails_rs::prelude::*;

#[sails_rs::event]
#[derive(Clone, Debug, PartialEq, Encode, TypeInfo, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[reflect_hash(crate = sails_rs)]
pub enum PartialIdlEvents {
    FirstDone,
    SecondDone(u32),
    ThirdDone(String),
}

#[derive(Default)]
pub struct PartialIdlService;

impl PartialIdlService {
    pub fn new() -> Self {
        Self
    }
}

#[sails_rs::service(events = PartialIdlEvents)]
impl PartialIdlService {
    #[export]
    pub fn first(&mut self) -> bool {
        self.emit_event(PartialIdlEvents::FirstDone).unwrap();
        true
    }

    #[export]
    pub fn second(&mut self, val: u32) -> u32 {
        self.emit_event(PartialIdlEvents::SecondDone(val)).unwrap();
        val * 2
    }

    #[export]
    pub fn third(&mut self) -> String {
        let res = "Third".to_string();
        self.emit_event(PartialIdlEvents::ThirdDone(res.clone()))
            .unwrap();
        res
    }
}

#[derive(Default)]
pub struct PartialIdlProgram;

#[sails_rs::program]
impl PartialIdlProgram {
    pub fn new() -> Self {
        Self
    }

    pub fn partial_idl_service(&self) -> PartialIdlService {
        PartialIdlService::new()
    }
}
