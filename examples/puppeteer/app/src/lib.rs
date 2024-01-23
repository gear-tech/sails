#![no_std]

mod client;

use client::{Client, Service};

use sails_client::NativeSender;
use sails_macros::gservice;

use gstd::prelude::*;

pub struct Puppeteer;

#[gservice]
impl Puppeteer {
    pub const fn new() -> Self {
        Self
    }

    pub async fn call_this(&mut self) -> Result<u32, String> {
        let mut sender = NativeSender::new();

        let client = Client::new().with_program_id([1; 32]);

        let result = client
            .this()
            .send(&mut sender)
            .await
            .expect("send msg")
            .result()
            .await
            .expect("parse msg");

        Ok(result)
    }
}
