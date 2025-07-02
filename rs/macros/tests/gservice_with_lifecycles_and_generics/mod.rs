use core::marker::PhantomData;
use sails_rs::prelude::*;

pub(super) struct SomeService<'a, 'b, T, U> {
    _t: PhantomData<&'a T>,
    u: &'b mut U,
}

impl<'a, 'b, T, U> SomeService<'a, 'b, T, U>
where
    T: Clone,
    U: Iterator<Item = u32>,
{
    pub fn new(u: &'b mut U) -> Self {
        Self { _t: PhantomData, u }
    }
}

#[service]
impl<'a, 'b, T, U> SomeService<'a, 'b, T, U>
where
    T: Clone,
    U: Iterator<Item = u32>,
{
    #[export]
    pub fn do_this(&mut self) -> u32 {
        self.u.next().unwrap_or_default()
    }
}
