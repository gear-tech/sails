#![no_std]

use sails_rs::prelude::*;

struct ComputeStressService;

#[sails_rs::service]
impl ComputeStressService {
    #[export]
    pub fn compute_stress(&mut self, n: u32) -> ComputeStressResult {
        let res = sum_of_fib(n);

        ComputeStressResult { res }
    }
}

pub fn sum_of_fib(n: u32) -> u32 {
    if n == 0 {
        0
    } else {
        fibonacci(n) + sum_of_fib(n - 1)
    }
}

fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[derive(TypeInfo, Encode)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct ComputeStressResult {
    pub res: u32,
}

pub struct ComputeStressProgram;

#[sails_rs::program]
impl ComputeStressProgram {
    pub fn new_for_bench() -> Self {
        Self
    }

    pub fn compute_stress(&self) -> ComputeStressService {
        ComputeStressService
    }
}
