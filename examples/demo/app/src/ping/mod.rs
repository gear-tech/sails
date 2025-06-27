use sails_rs::prelude::*;

#[derive(Default)]
pub struct PingService(());

#[service]
impl PingService {
    // This is a service command as it works over `&mut self`
    #[export]
    pub fn ping(&mut self, input: String) -> Result<String, String> {
        if input != "ping" {
            Err("Invalid input".into())
        } else {
            Ok("pong".into())
        }
    }
}
