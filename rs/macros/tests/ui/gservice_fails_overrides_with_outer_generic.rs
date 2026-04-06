use sails_macros::service;

pub struct BaseService<T>(T);

pub struct ChildService<T>(T);

#[service]
impl<T: Clone> ChildService<T> {
    #[export(overrides = BaseService<T>)]
    pub fn foo(&self) -> u32 {
        0
    }
}

fn main() {}
