use sails_rtl::{gstd::gservice, prelude::*};

#[derive(Default)]
pub struct PingService(());

#[derive(sails_rtl::Encode, sails_rtl::Decode, sails_rtl::TypeInfo)]
pub enum PingEvents {
    Ping,
    Pong,
}

#[gservice(events = PingEvents)]
impl PingService {
    pub fn ping(&mut self, input: String) -> Result<String, String> {
        if input != "ping" {
            Err("Invalid input".into())
        } else {
            self.notify_on(PingEvents::Ping).unwrap();
            self.notify_on(PingEvents::Pong).unwrap();
            Ok("pong".into())
        }
    }
}
