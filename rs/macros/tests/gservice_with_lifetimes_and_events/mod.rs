use core::marker::PhantomData;
use sails_rs::prelude::*;

#[derive(Default)]
pub(super) struct Service<'l, T = String> {
    _t: Option<u64>,
    _a: PhantomData<&'l T>,
}

#[event]
#[derive(TypeInfo, Encode, Clone, Debug, PartialEq, ReflectHash)]
#[reflect_hash(crate = sails_rs)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl<T> Service<'_, T>
where
    T: Clone,
{
    #[export]
    pub fn do_this(&mut self) -> u32 {
        self.emit_event(MyEvents::Event1).unwrap();
        42
    }
}
