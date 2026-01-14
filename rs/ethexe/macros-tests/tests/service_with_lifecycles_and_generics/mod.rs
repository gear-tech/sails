use core::marker::PhantomData;
use sails_rs::prelude::*;

#[derive(Default)]
pub(super) struct MyGenericService<'a, T = String> {
    _a: PhantomData<&'a T>,
}

#[service]
impl<T> MyGenericService<'_, T>
where
    T: Clone,
{
    #[export]
    pub fn do_this(&mut self) -> u32 {
        42
    }
}
