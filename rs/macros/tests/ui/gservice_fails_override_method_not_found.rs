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
    // Error: 'bar' does not exist in BaseService
    #[export(overrides = BaseService)]
    pub fn bar(&self) -> u32 { 300 }
}

fn main() {}
