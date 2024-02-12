#![no_std]

pub mod puppet;

use puppet::Service;

use sails_macros::gservice;

use gstd::{prelude::*, ActorId};

pub struct Puppeteer {
    puppet: Box<dyn Service>,
}

const PUPPET_ADDRESS: ActorId = ActorId::new([1; 32]);

#[gservice]
impl Puppeteer {
    pub const fn new(puppet: Box<dyn Service>) -> Self {
        Self { puppet }
    }

    pub async fn call_this(&mut self) -> Result<u32, String> {
        let result = self
            .puppet
            .this()
            .send(PUPPET_ADDRESS)
            .await
            .expect("send msg")
            .response()
            .await
            .expect("parse msg");

        Ok(result)
    }
}
