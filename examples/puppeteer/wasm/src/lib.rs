#![no_std]

use gstd::{boxed::Box, msg};
use puppeteer_app::puppet::Client;
use puppeteer_app::{requests as service_requests, Puppeteer};
use sails_sender::GStdSender;

static mut SERVICE: Option<Puppeteer> = None;
static mut SENDER: Option<GStdSender> = None;

fn service() -> &'static mut Puppeteer {
    let s = unsafe { &mut SERVICE };
    if s.is_none() {
        unsafe { SENDER = Some(GStdSender::new()) };
        let sender = unsafe { &SENDER }.as_ref().unwrap();
        *s = Some(Puppeteer::new(Box::new(Client::new(&sender))));
    }

    s.as_mut().unwrap()
}

#[gstd::async_main]
async fn main() {
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes = service_requests::process(service(), &input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
