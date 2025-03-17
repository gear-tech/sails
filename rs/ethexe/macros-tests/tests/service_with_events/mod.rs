use sails_rs::{Encode, TypeInfo, service};

#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.notify_on(MyEvents::Event1).unwrap();
    }
}
