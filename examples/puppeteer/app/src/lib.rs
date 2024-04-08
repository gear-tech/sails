#![no_std]

pub mod puppet;

use puppet::Service;

use sails_macros::gservice;

use gstd::{prelude::*};
use sails_rtl::ActorId;
use sails_rtl::calls::Call;

pub struct Puppeteer {
    puppet: Box<dyn Service>,
}


#[gservice]
impl Puppeteer {
    pub const fn new(puppet: Box<dyn Service>) -> Self {
        Self { puppet }
    }

    pub async fn call_this(&mut self) -> Result<u32, String> {
        let puppet_address = ActorId::from([1; 32]);

        let result = self
            .puppet
            .this()
            .send_to(puppet_address)
            .await
            .expect("send msg")
            .reply()
            .await
            .expect("parse msg");

        Ok(result)
    }
}
