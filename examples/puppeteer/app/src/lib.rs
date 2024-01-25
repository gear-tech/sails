#![no_std]

pub mod puppet;

use puppet::Service;

use sails_macros::gservice;

use gstd::prelude::*;

pub struct Puppeteer {
    puppet: Box<dyn Service>,
}

const PROGRAM_ID: [u8; 32] = [1; 32];

#[gservice]
impl Puppeteer {
    pub const fn new(puppet: Box<dyn Service>) -> Self {
        Self { puppet }
    }

    pub async fn call_this(&mut self) -> Result<u32, String> {
        let result = self
            .puppet
            .this()
            .send(PROGRAM_ID)
            .await
            .expect("send msg")
            .result()
            .await
            .expect("parse msg");

        Ok(result)
    }
}
