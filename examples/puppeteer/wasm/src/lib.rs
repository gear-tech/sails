#![no_std]

use core::ptr::addr_of_mut;
use gstd::msg;
use puppeteer_app::puppet::ThisThatSvc;
use puppeteer_app::Puppeteer;
use sails_rtl::gstd::calls::{GStdArgs, GStdRemoting};

static mut SERVICE: Option<Puppeteer<ThisThatSvc<GStdRemoting, GStdArgs>>> = None;

fn service() -> &'static mut Puppeteer<ThisThatSvc<GStdRemoting, GStdArgs>> {
    let s = unsafe { &mut *addr_of_mut!(SERVICE) };
    if s.is_none() {
        let remoting = GStdRemoting;
        *s = Some(Puppeteer::new(ThisThatSvc::new(remoting)));
    }

    s.as_mut().unwrap()
}

#[gstd::async_main]
async fn main() {
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes = service().handle(&input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
