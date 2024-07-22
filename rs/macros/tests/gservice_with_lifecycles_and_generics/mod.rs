use core::marker::PhantomData;
use sails_rs::gstd::gservice;

#[derive(Default)]
pub(super) struct MyGenericService<'a, T> {
    _a: PhantomData<&'a T>,
}

#[gservice]
impl<'a, T> MyGenericService<'a, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        42
    }
}
