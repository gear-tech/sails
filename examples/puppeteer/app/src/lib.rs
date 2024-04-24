#![no_std]

pub mod puppet;

use core::marker::PhantomData;

use puppet::traits::ThisThatSvc;

use sails_macros::gservice;

use gstd::prelude::*;
use sails_rtl::calls::{Call, Remoting};
use sails_rtl::ActorId;

pub struct Puppeteer<A: Default, R: Remoting<A>, Client: ThisThatSvc<R, A>> {
    _args: PhantomData<A>,
    _remote: PhantomData<R>,
    puppet: Client,
}

#[gservice]
impl<A, R, Client> Puppeteer<A, R, Client>
where
    A: Default,
    R: Remoting<A>,
    Client: ThisThatSvc<R, A>,
{
    pub const fn new(puppet: Client) -> Self {
        Self {
            _args: PhantomData,
            _remote: PhantomData,
            puppet,
        }
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
