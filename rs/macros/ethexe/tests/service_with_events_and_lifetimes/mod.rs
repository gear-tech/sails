use core::marker::PhantomData;
use sails_rs::{gstd::service, Encode, TypeInfo};

#[derive(Default)]
pub(super) struct MyGenericEventsService<'l, T> {
    _t: Option<u64>,
    _a: PhantomData<&'l T>,
}

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl<T> MyGenericEventsService<'_, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        self.notify_on(MyEvents::Event1).unwrap();
        42
    }
}
