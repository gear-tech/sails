use core::marker::PhantomData;
use sails_rs::{Encode, TypeInfo, gstd::service};

#[derive(Default)]
pub(super) struct MyGenericEventsService<'l, T> {
    _t: Option<u64>,
    _a: PhantomData<&'l T>,
}

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl<T> MyGenericEventsService<'_, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        self.emit_event(MyEvents::Event1).unwrap();
        42
    }
}
