use core::marker::PhantomData;
use sails_rs::service;

#[derive(Default)]
pub(super) struct MyGenericService<'a, T> {
    _a: PhantomData<&'a T>,
}

#[service]
impl<T> MyGenericService<'_, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        42
    }
}
