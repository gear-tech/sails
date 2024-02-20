#![no_std]

use rmrk_resource_app::{requests, CatalogClientImpl, ResourceStorage};
use sails_rtl_gstd::{gstd, gstd::msg, GStdExecContext};
use sails_sender::GStdSender;

#[no_mangle]
extern "C" fn init() {
    let exec_context = GStdExecContext::new();
    let sender = GStdSender::new();
    let catalog_client = CatalogClientImpl::new(sender);
    ResourceStorage::new(exec_context, catalog_client);
}

#[gstd::async_main]
async fn main() {
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let exec_context = GStdExecContext::new();
    let sender = GStdSender::new();
    let catalog_client = CatalogClientImpl::new(sender);
    let mut resource_storage = ResourceStorage::new(exec_context, catalog_client);
    let output_bytes = requests::process(&mut resource_storage, &input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
