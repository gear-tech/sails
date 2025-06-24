use core::marker::PhantomData;

#[derive(Default)]
pub(super) struct MyGenericEventsService<'l, T> {
    _t: Option<u64>,
    _a: PhantomData<&'l T>,
}

#[sails_rs::event]
#[derive(Clone, Debug, PartialEq, sails_rs::Encode, sails_rs::TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum MyEvents {
    Event1,
}

#[sails_rs::service(events = MyEvents)]
impl<T> MyGenericEventsService<'_, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        self.emit_eth_event(MyEvents::Event1).unwrap();
        42
    }
}
