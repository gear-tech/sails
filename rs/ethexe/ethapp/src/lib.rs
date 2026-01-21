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

    pub fn inherited(&self) -> InheritedService {
        InheritedService::new()
    }
}

pub struct BaseService;

#[service]
impl BaseService {
    #[export]
    pub fn foo(&self) -> u32 {
        100
    }
}

pub struct InheritedService {
    base: BaseService,
}

impl InheritedService {
    pub fn new() -> Self {
        Self { base: BaseService }
    }
}

impl From<InheritedService> for BaseService {
    fn from(value: InheritedService) -> Self {
        value.base
    }
}

#[service(extends = BaseService)]
impl InheritedService {
    // We override 'foo' from BaseService using a method named 'bar'
    #[override_entry(base_service_methods::Foo)]
    #[export]
    pub fn bar(&self) -> u32 {
        200
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
