use sails::{gstd::gservice, prelude::*};

#[derive(Default)]
pub struct PingService(());

#[gservice]
impl PingService {
    // This is a service command as it works over `&mut self`
    pub fn ping(&mut self, input: String) -> Result<String, String> {
        if input != "ping" {
            Err("Invalid input".into())
        } else {
            Ok("pong".into())
        }
    }
}
