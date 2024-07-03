#![no_std]

pub mod puppet;

use core::marker::PhantomData;
use gstd::prelude::*;
use puppet::traits::ThisThatSvc;
use sails_rtl::{calls::Call, gstd::gservice, ActorId};

#[derive(Clone)]
pub struct Puppeteer<A: Default, Client: ThisThatSvc<A>> {
    _args: PhantomData<A>,
    puppet: Client,
}

#[gservice]
impl<A, Client> Puppeteer<A, Client>
where
    A: Default,
    Client: ThisThatSvc<A>,
{
    pub const fn new(puppet: Client) -> Self {
        Self {
            _args: PhantomData,
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

    use puppet::{this_that_svc_io, DoThatParam, ManyVariants};

    #[test]
    fn test_io_module_encode() {
        let bytes = this_that_svc_io::DoThat::encode_call(DoThatParam {
            p1: u32::MAX,
            p2: "hello".to_string(),
            p3: ManyVariants::One,
        });

        assert_eq!(
            bytes,
            vec![
                24, 68, 111, 84, 104, 97, 116, // DoThat
                255, 255, 255, 255, // p1
                20, 104, 101, 108, 108, 111, // p2
                0    // p3
            ]
        );
    }

    #[test]
    fn test_io_module_decode_reply() {
        let bytes = vec![
            24, 68, 111, 84, 104, 97, 116, // DoThat
            0,   // Ok
            16, 65, 65, 65, 65, // len + "AAAA"
            255, 255, 255, 255, // u32::MAX
        ];

        let reply: Result<(String, u32), (String,)> =
            this_that_svc_io::DoThat::decode_reply(&bytes).unwrap();

        assert_eq!(reply, Ok(("AAAA".to_string(), u32::MAX)));
    }
}
