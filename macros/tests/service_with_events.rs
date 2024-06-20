use sails_rtl::{
    gstd::{gservice, services::Service},
    Encode, MessageId, TypeInfo,
};

#[allow(dead_code)]
struct MyService(u8);

#[derive(TypeInfo, Encode, Clone, Debug, PartialEq)]
enum MyEvents {
    Event1,
}

#[gservice(events = MyEvents)]
impl MyService {
    pub fn my_method(&mut self) {
        self.notify_on(MyEvents::Event1).unwrap();
    }
}

#[test]
fn service_with_events_works() {
    let mut exposure = MyService(0).expose(MessageId::from(142), &[1, 4, 2]);

    let mut events = Vec::new();
    {
        let _event_listener_guard = exposure.set_event_listener(|event| events.push(event.clone()));

        exposure.my_method();
    }

    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}
