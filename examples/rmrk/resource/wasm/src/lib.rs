#![no_std]

use gstd::msg;
use rmrk_resource_app::{requests, ResourceStorage};
use sails_exec_context_gstd::GStdExecContext;

#[no_mangle]
extern "C" fn init() {
    let exec_context = GStdExecContext::new();
    ResourceStorage::new(exec_context);
}

#[gstd::async_main]
async fn main() {
    let exec_context = GStdExecContext::new();
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes =
        requests::process(&mut ResourceStorage::new(exec_context), &input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
