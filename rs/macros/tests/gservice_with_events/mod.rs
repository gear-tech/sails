use sails_rs::{Encode, TypeInfo, gstd::service};

#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.emit_event(MyEvents::Event1).unwrap();
    }
}
