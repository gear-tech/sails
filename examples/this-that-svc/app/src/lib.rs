#![no_std]

use gstd::{debug, prelude::*};
use sails_rtl::{gstd::gservice, H256, U256};

#[derive(Default)]
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

    pub fn noop(&mut self) {
        debug!("Handling 'noop'");
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

#[allow(dead_code)]
#[derive(Debug, Decode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct TupleStruct(bool);

#[derive(Debug, Decode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct DoThatParam {
    pub p1: u32,
    pub p2: String,
    pub p3: ManyVariants,
}

#[derive(Debug, Decode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub enum ManyVariants {
    One,
    Two(u32),
    Three(Option<U256>),
    Four { a: u32, b: Option<u16> },
    Five(String, H256),
    Six((u32,)),
}
