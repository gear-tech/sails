#![no_std]

use references_app::ReferenceService;
use sails_rtl::gstd::{msg, services::Service};

#[gstd::async_main]
async fn main() {
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes = ReferenceService::new("MyService")
        .expose(msg::id().into(), &[1, 2, 3])
        .handle(&input_bytes)
        .await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
