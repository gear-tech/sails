#![no_std]

use sails_rs::prelude::*;

fn sum_fibonacci(n: u32) -> u32 {
    fn fibonacci(n: u32) -> u32 {
        if n <= 1 {
            return n;
        }
        fibonacci(n - 1) + fibonacci(n - 2)
    }

    (0..n).fold(0, |sum, i| sum + fibonacci(i))
}

#[derive(Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct AsyncService;

#[sails_rs::service]
impl AsyncService {
    pub async fn some_async_method(&self) -> &'static str {
        sum_fibonacci(10);
        "Async some method"
    }
}

#[derive(Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct NoAsyncService;

#[sails_rs::service]
impl NoAsyncService {
    pub fn some_method(&self) -> &'static str {
        sum_fibonacci(10);
        "Non async some method"
    }
}

pub struct MyProgram;

#[sails_rs::program]
impl MyProgram {
    #[allow(clippy::new_without_default)]
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
