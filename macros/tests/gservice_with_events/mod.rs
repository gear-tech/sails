use sails_rtl::{gstd::gservice, Encode, TypeInfo};

#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
pub enum MyEvents {
    Event1,
}

#[gservice(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.notify_on(MyEvents::Event1).unwrap();
    }
}
