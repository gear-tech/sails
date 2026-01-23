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
    // Error: base 'foo' is sync, but this override is async
    #[export(overrides = BaseService)]
    pub async fn foo(&self) -> u32 { 200 }
}

fn main() {}
