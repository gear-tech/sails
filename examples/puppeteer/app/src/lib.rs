#![no_std]

pub mod puppet;

use puppet::traits::ThisThatSvc;

use sails_macros::gservice;

use gstd::prelude::*;
use sails_rtl::calls::Call;
use sails_rtl::gstd::calls::{GStdArgs, GStdRemoting};
use sails_rtl::ActorId;

pub struct Puppeteer<ThisThatClient> {
    puppet: ThisThatClient,
}

#[gservice]
impl<ThisThatClient> Puppeteer<ThisThatClient>
where
    ThisThatClient: ThisThatSvc<GStdRemoting, GStdArgs>,
{
    pub const fn new(puppet: ThisThatClient) -> Self {
        Self { puppet }
    }

    pub async fn call_this(&mut self) -> Result<u32, String> {
        let puppet_address = ActorId::from([1; 32]);

        let result = self
            .puppet
            .this()
            .publish(puppet_address)
            .await
            .expect("send msg")
            .reply()
            .await
            .expect("parse msg");

        Ok(result)
    }
}
