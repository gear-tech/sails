use sails_rs::prelude::*;

#[derive(Default, Clone)]
pub struct BaseService;

#[service]
impl BaseService {
    #[export]
    pub async fn foo(&self) -> u32 { 100 }
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
    // Error: base 'foo' is async, but this override is sync
    #[export(overrides = BaseService)]
    pub fn foo(&self) -> u32 { 200 }
}

fn main() {}
