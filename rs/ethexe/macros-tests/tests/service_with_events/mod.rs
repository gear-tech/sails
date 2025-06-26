use sails_rs::{event, service, Encode, TypeInfo};

#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[event]
#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.emit_eth_event(MyEvents::Event1).unwrap();
    }
}
