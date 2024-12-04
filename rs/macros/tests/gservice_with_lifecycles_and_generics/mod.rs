use core::marker::PhantomData;
use sails_rs::gstd::service;

#[derive(Default)]
pub(super) struct MyGenericService<'a, T> {
    _a: PhantomData<&'a T>,
}

#[allow(clippy::needless_lifetimes)]
#[service]
impl<'a, T> MyGenericService<'a, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        42
    }
}
