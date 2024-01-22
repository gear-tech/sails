#![no_std]

mod client;

use gstd::{debug, prelude::*};
use sails_macros::gservice;

pub struct MyService;

#[gservice]
impl MyService {
    pub const fn new() -> Self {
        Self
    }

    pub async fn do_this(
        &mut self,
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: TupleStruct,
    ) -> (String, u32) {
        debug!("Handling 'do_this': {}, {}, {:?}, {:?}", p1, p2, p3, p4);
        (p2, p1)
    }

    pub fn do_that(&mut self, param: DoThatParam) -> Result<(String, u32), (String,)> {
        debug!("Handling 'do_that': {:?}", param);
        Ok((param.p2, param.p1))
    }
    pub fn this(&self) -> u32 {
        debug!("Handling 'this'");
        42
    }

    // That
    pub fn that(&self) -> Result<String, String> {
        debug!("Handling 'that'");
        Ok("Forty two".into())
    }
}

#[derive(Debug, Decode, TypeInfo)]
pub struct TupleStruct(bool);

#[derive(Debug, Decode, TypeInfo)]
pub struct DoThatParam {
    pub p1: u32,
    pub p2: String,
    pub p3: ManyVariants,
}

#[derive(Debug, Decode, TypeInfo)]
pub enum ManyVariants {
    One,
    Two(u32),
    Three(Option<u32>),
    Four { a: u32, b: Option<u16> },
    Five(String, u32),
    Six((u32,)),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::*;

    #[test]
    fn test_client() {
        let payload = Client::new()
            .do_this(
                42,
                "AAAA".to_owned(),
                (None, 0),
                ThisThatSvcAppTupleStruct(true),
            )
            .into_bytes();

        assert_eq!(
            payload,
            vec![68, 111, 84, 104, 105, 115, 47, 42, 0, 0, 0, 16, 65, 65, 65, 65, 0, 0, 1]
        );
    }
}
