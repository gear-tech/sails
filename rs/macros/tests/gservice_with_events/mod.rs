use sails::prelude::*;

#[allow(dead_code)]
pub struct MyServiceWithEvents(pub u8);

#[event]
#[derive(TypeInfo, Encode, Clone, Debug, PartialEq, ReflectHash)]
#[reflect_hash(crate = sails)]
pub enum MyEvents {
    Event1,
}

#[service(events = MyEvents)]
impl MyServiceWithEvents {
    #[export]
    pub fn my_method(&mut self) {
        self.emit_event(MyEvents::Event1).unwrap();
    }
}
