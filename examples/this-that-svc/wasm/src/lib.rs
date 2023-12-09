#![no_std]

use gstd::msg;
use this_that_svc_app::{handlers as service_handlers, MyService};

static mut MY_SERVICE: MyService = MyService::new();

fn my_service() -> &'static mut MyService {
    unsafe { &mut MY_SERVICE }
}

#[gstd::async_main]
async fn main() {
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes = service_handlers::process_request(my_service(), &input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
