#![no_std]

use core::ptr::addr_of_mut;
use gstd::{boxed::Box, msg};
use sails_rtl::gstd::calls::Sender;
use puppeteer_app::puppet::Client;
use puppeteer_app::Puppeteer;

static mut SERVICE: Option<Puppeteer> = None;

fn service() -> &'static mut Puppeteer {
    let s = unsafe { &mut *addr_of_mut!(SERVICE) };
    if s.is_none() {
        let sender = Sender;
        *s = Some(Puppeteer::new(Box::new(Client::new(sender))));
    }

    s.as_mut().unwrap()
}

#[gstd::async_main]
async fn main() {
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes = service().handle(&input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
