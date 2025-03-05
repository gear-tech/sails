use sails_rs::{service, Encode, TypeInfo};

#[allow(dead_code)]
pub struct SomeService(pub u8);

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum SomeEvents {
    Event1,
}

#[service(events = SomeEvents)]
#[allow(dead_code)]
impl SomeService {
    pub fn do_this(&mut self) -> u32 {
        self.notify_on(SomeEvents::Event1).unwrap();
        42
    }

    pub fn this(&self) -> bool {
        true
    }
}
