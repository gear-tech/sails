use sails_rtl::{gstd::gservice, Encode, TypeInfo};

pub(super) struct MyServiceWithEvents;

#[derive(TypeInfo, Encode)]
enum MyEvents {
    Event1,
}

#[gservice(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.notify_on(MyEvents::Event1).unwrap();
    }
}
