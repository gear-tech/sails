use core::marker::PhantomData;
use sails_rtl::{gstd::gservice, Encode, TypeInfo};

#[derive(Default)]
pub(super) struct MyGenericEventsService<'l, T> {
    _t: Option<u64>,
    _a: PhantomData<&'l T>,
}

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
pub enum MyEvents {
    Event1,
}

#[gservice(events = MyEvents)]
impl<'l, T> MyGenericEventsService<'l, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        self.notify_on(MyEvents::Event1).unwrap();
        42
    }
}
