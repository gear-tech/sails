#![no_std]

use core::ptr::addr_of_mut;
use puppeteer_app::{puppet::ThisThatSvc, Puppeteer};
use sails_rtl::gstd::{
    calls::{GStdArgs, GStdRemoting},
    msg,
    services::Service as GStdService,
};

type Service = Puppeteer<GStdArgs, GStdRemoting, ThisThatSvc<GStdRemoting, GStdArgs>>;

static mut SERVICE: Option<Service> = None;

fn service() -> &'static mut Service {
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
    let output_bytes = service()
        .clone()
        .expose(msg::id().into(), &[1, 2, 3])
        .handle(&input_bytes)
        .await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
