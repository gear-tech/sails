#![no_std]

use gstd::msg;
use puppeteer_app::{requests as service_requests, Puppeteer};

static mut SERVICE: Puppeteer = Puppeteer::new();

fn service() -> &'static mut Puppeteer {
    unsafe { &mut SERVICE }
}

#[gstd::async_main]
async fn main() {
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes = service_requests::process(service(), &input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
