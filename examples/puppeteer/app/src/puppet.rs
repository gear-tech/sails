#![automatically_derived]
#![allow(unused)]
use gstd::prelude::*;
use parity_scale_codec::{Decode, Encode};
use sails_sender::Call;
pub trait Service {
    fn do_this(
        &mut self,
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: ThisThatSvcAppTupleStruct,
    ) -> Call<(String, u32)>;
    fn do_that(
        &mut self,
        param: ThisThatSvcAppDoThatParam,
    ) -> Call<Result<(String, u32), (String,)>>;
    fn this(&self) -> Call<u32>;
    fn that(&self) -> Call<Result<String, String>>;
}
#[derive(PartialEq, Debug, Encode, Decode)]
struct DoThisRequestArgs {
    pub p1: u32,
    pub p2: String,
    pub p3: (Option<String>, u8),
    pub p4: ThisThatSvcAppTupleStruct,
}
#[derive(PartialEq, Debug, Encode, Decode)]
struct DoThatRequestArgs {
    pub param: ThisThatSvcAppDoThatParam,
}
#[derive(PartialEq, Debug, Encode, Decode)]
struct ThisRequestArgs {}
#[derive(PartialEq, Debug, Encode, Decode)]
struct ThatRequestArgs {}
#[derive(Default, Clone)]
pub struct Client {
    program_id: [u8; 32],
}
impl Client {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Service for Client {
    fn do_this(
        &mut self,
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: ThisThatSvcAppTupleStruct,
    ) -> Call<(String, u32)> {
        let mut payload = Vec::from("DoThis/");
        DoThisRequestArgs { p1, p2, p3, p4 }.encode_to(&mut payload);
        Call::new(payload)
    }
    fn do_that(
        &mut self,
        param: ThisThatSvcAppDoThatParam,
    ) -> Call<Result<(String, u32), (String,)>> {
        let mut payload = Vec::from("DoThat/");
        DoThatRequestArgs { param }.encode_to(&mut payload);
        Call::new(payload)
    }
    fn this(&self) -> Call<u32> {
        let mut payload = Vec::from("This/");
        ThisRequestArgs {}.encode_to(&mut payload);
        Call::new(payload)
    }
    fn that(&self) -> Call<Result<String, String>> {
        let mut payload = Vec::from("That/");
        ThatRequestArgs {}.encode_to(&mut payload);
        Call::new(payload)
    }
}
#[derive(PartialEq, Debug, Encode, Decode)]
pub struct ThisThatSvcAppTupleStruct(bool);
#[derive(PartialEq, Debug, Encode, Decode)]
pub struct ThisThatSvcAppDoThatParam {
    p1: u32,
    p2: String,
    p3: ThisThatSvcAppManyVariants,
}
#[derive(PartialEq, Debug, Encode, Decode)]
pub enum ThisThatSvcAppManyVariants {
    One,
    Two(u32),
    Three(Option<u32>),
    Four { a: u32, b: Option<u16> },
    Five((String, u32)),
    Six((u32,)),
}
