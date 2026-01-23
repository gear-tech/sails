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
    // Error: entry_id 0 is 'correct_name', but here we use 'wrong_name'.
    // Even if types match, hashes won't match because names are different.
    #[export(overrides = BaseService, entry_id = 0)]
    pub fn wrong_name(&self) -> u32 { 200 }
}

fn main() {}
