#[allow(dead_code)]
pub struct SomeService;

#[sails_rs::event]
#[derive(Clone, Debug, PartialEq, sails_rs::Encode, sails_rs::TypeInfo)]
pub enum SomeEvents {
    Event1,
}

#[sails_rs::service(events = SomeEvents)]
impl SomeService {
    pub fn do_this(&mut self) -> u32 {
        self.emit_event(SomeEvents::Event1).unwrap();
        42
    }
    pub fn this(&self) -> bool {
        true
    }
}
