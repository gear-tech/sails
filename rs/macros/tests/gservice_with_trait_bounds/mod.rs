use core::marker::PhantomData;
use sails_rs::gstd::service;

#[derive(Default)]
pub(super) struct MyServiceWithTraitBounds<'a, T> {
    _a: PhantomData<&'a T>,
}

#[allow(clippy::needless_lifetimes)]
#[service]
impl<'a, T: Into<u32>> MyServiceWithTraitBounds<'a, T> {
    pub fn do_this(&mut self) -> u32 {
        42
    }
}
