#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[sails_rs::event]
#[derive(Clone, Debug, PartialEq)]
pub enum MyEvents {
    Event1,
}

#[sails_rs::service(events = MyEvents)]
impl MyServiceWithEvents {
    pub fn my_method(&mut self) {
        self.emit_eth_event(MyEvents::Event1).unwrap();
    }
}
