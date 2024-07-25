use sails_rs::{gstd::service, Encode, TypeInfo};

#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.notify_on(MyEvents::Event1).unwrap();
    }
}
