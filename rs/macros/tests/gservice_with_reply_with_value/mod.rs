use sails_macros::service;
use sails_rs::{gstd::CommandReply, Encode};

pub(super) struct MyServiceWithReplyWithValue;

#[service]
impl MyServiceWithReplyWithValue {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> CommandReply<String> {
        CommandReply::new(format!("{p1}: {p2}")).with_value(100_000_000_000)
    }

    pub async fn do_that(&mut self, p1: u32, p2: String) -> impl Into<CommandReply<String>> {
        (format!("{p1}: {p2}"), 100_000_000_000)
    }

    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[derive(Encode)]
pub(super) struct MyDoThisParams {
    pub(super) p1: u32,
    pub(super) p2: String,
}
