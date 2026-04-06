use sails_macros::service;

pub struct BaseService<T>(T);

pub struct ChildService<T>(T);

#[service(extends = BaseService<T>)]
impl<T: Clone> ChildService<T> {
    #[export]
    pub fn foo(&self) -> u32 {
        0
    }
}

fn main() {}
