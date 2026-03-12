use sails_rs::prelude::*;

#[derive(Default, Clone)]
pub struct BaseService;

#[service]
impl BaseService {
    #[export]
    pub fn foo(&self) -> u32 { 100 }
}

#[derive(Default, Clone)]
pub struct InheritedService {
    base: BaseService,
}

impl From<InheritedService> for BaseService {
    fn from(value: InheritedService) -> Self { value.base }
}

#[service(extends = BaseService)]
impl InheritedService {
    // Error: returns u64 instead of u32 (signature mismatch)
    #[export(overrides = BaseService)]
    pub fn foo(&self) -> u64 { 200 }
}

fn main() {}
