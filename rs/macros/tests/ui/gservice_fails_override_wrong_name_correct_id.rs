use sails_rs::prelude::*;

#[derive(Default, Clone)]
pub struct BaseService;

#[service]
impl BaseService {
    #[export]
    pub fn correct_name(&self) -> u32 { 100 } // entry_id = 0
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
    // Error: 'wrong_name' is not found in 'BaseService' by name.
    #[export(overrides = BaseService)]
    pub fn wrong_name(&self) -> u32 { 200 }
}

fn main() {}