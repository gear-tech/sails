#![no_std]

use sails_rs::prelude::*;

pub struct MyProgram;

#[program]
impl MyProgram {
    pub fn create_prg() -> Self {
        MyProgram
    }

    #[export(payable)]
    pub fn create_payable() -> Self {
        MyProgram
    }

    pub fn svc1(&self) -> SomeService {
        SomeService
    }
}

pub struct SomeService;

#[service]
impl SomeService {
    #[export]
    pub async fn do_this(&mut self, p1: u32, _p2: String) -> u32 {
        p1
    }

    #[export(payable)]
    pub fn do_this_payable(&mut self, p1: u32) -> u32 {
        p1
    }

    #[export]
    pub fn this(&self, p1: ActorId) -> ActorId {
        p1
    }
}
