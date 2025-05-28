#![no_std]

use sails_rs::prelude::*;

#[derive(Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct AsyncService;

#[sails_rs::service]
impl AsyncService {
    pub async fn async_method(&self) -> &'static str {
        "This is an asynchronous method"
    }
}

#[derive(Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct NoAsyncService;

#[sails_rs::service]
impl NoAsyncService {
    pub fn sync_method(&self) -> &'static str {
        "This is a synchronous method"
    }
}

pub struct MyProgram;

#[sails_rs::program]
impl MyProgram {
    pub fn new() -> Self {
        MyProgram
    }

    pub fn async_service(&self) -> AsyncService {
        AsyncService
    }

    pub fn no_async_service(&self) -> NoAsyncService {
        NoAsyncService
    }
}
