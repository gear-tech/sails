use sails_rtl::{gstd::gservice, Encode, TypeInfo};

struct MyService;

#[derive(TypeInfo, Encode)]
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
    let mut service = MyService;
    service.my_method();
}
