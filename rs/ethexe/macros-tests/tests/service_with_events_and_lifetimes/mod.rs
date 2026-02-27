use core::marker::PhantomData;
use sails_rs::prelude::*;

#[derive(Default)]
pub(super) struct MyGenericEventsService<'l, T = String> {
    _t: Option<u64>,
    _a: PhantomData<&'l T>,
}

#[event]
#[derive(TypeInfo, Encode, Clone, Debug, PartialEq, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
#[reflect_hash(crate = sails_rs)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl<T> MyGenericEventsService<'_, T>
where
    T: Clone,
{
    #[export]
    pub fn do_this(&mut self) -> u32 {
        self.emit_eth_event(MyEvents::Event1).unwrap();
        42
    }
}
