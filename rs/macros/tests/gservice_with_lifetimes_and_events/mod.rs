use core::marker::PhantomData;

#[derive(Default)]
pub(super) struct MyGenericEventsService<'l, T> {
    _t: Option<u64>,
    _a: PhantomData<&'l T>,
}

#[sails_rs::event]
#[derive(Clone, Debug, PartialEq, sails_rs::Encode, sails_rs::TypeInfo)]
pub enum MyEvents {
    Event1,
}

#[sails_rs::service(events = MyEvents)]
impl<T> MyGenericEventsService<'_, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        self.emit_event(MyEvents::Event1).unwrap();
        42
    }
}
