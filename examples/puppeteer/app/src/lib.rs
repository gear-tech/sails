#![no_std]

pub mod puppet;

use core::marker::PhantomData;
use gstd::prelude::*;
use puppet::traits::ThisThatSvc;
use sails_rtl::{
    calls::{Call, Remoting},
    gstd::gservice,
    ActorId,
};

#[derive(Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    use puppet::{this_that_svc_calls, DoThatParam, ManyVariants};

    #[test]
    fn test_call_module() {
        let bytes = this_that_svc_calls::DoThatCall(DoThatParam {
            p1: u32::MAX,
            p2: "hello".to_string(),
            p3: ManyVariants::One,
        })
        .encode();

        assert_eq!(
            bytes,
            vec![
                24, 68, 111, 84, 104, 97, 116, // DoThat
                255, 255, 255, 255, // p1
                20, 104, 101, 108, 108, 111, // p2
                0    // p3
            ]
        );

        let call = this_that_svc_calls::DoThatCall::from_bytes(&bytes).expect("decode call");
        assert_eq!(call.0.p1, u32::MAX);
        assert_eq!(call.0.p2, "hello");
        assert_eq!(call.0.p3, ManyVariants::One);
    }
}
