use sails_rtl::{gstd::gservice, prelude::*};

pub struct PingService {}

#[gservice]
impl PingService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ping(&mut self, input: String) -> Result<String, String> {
        if input != "ping" {
            Err("Invalid input".into())
        } else {
            Ok("pong".into())
        }
    }
}

impl Default for PingService {
    fn default() -> Self {
        Self::new()
    }
}
