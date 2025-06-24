#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[sails_rs::event]
#[derive(Clone, Debug, PartialEq, sails_rs::Encode, sails_rs::TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum MyEvents {
    Event1,
}

#[sails_rs::service(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.emit_eth_event(MyEvents::Event1).unwrap();
    }
}
