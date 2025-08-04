use core::marker::PhantomData;
use sails_rs::prelude::*;

#[derive(Default)]
pub(super) struct MyServiceWithTraitBounds<'a, T> {
    _a: PhantomData<&'a T>,
}

#[service]
impl<T: Into<u32>> MyServiceWithTraitBounds<'_, T> {
    #[export]
    pub fn do_this(&mut self) -> u32 {
        42
    }
}
