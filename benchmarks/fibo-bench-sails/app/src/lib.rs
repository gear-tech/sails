#![no_std]

use sails_rs::{
    Encode,
    prelude::{TypeInfo, Vec},
};

#[derive(TypeInfo, Encode)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
struct FiboStressResult {
    inner: Vec<u8>,
}

struct FiboStressService;

#[sails_rs::service]
impl FiboStressService {
    pub fn stress(&mut self, n: u32) -> FiboStressResult {
        let sum = fibonacci_sum(n);
        let mut buf = Vec::with_capacity(sum as usize);
        (0..sum).for_each(|_| buf.push(42));

        FiboStressResult { inner: buf }
    }
}

pub struct FiboStressProgram;

#[sails_rs::program]
impl FiboStressProgram {
    pub fn new() -> Self {
        Self
    }

    pub fn fibo_stress(&self) -> FiboStressService {
        FiboStressService
    }
}

pub fn fibonacci_sum(n: u32) -> u32 {
    let (mut sum, mut prev, mut curr) = (0u32, 0u32, 1u32);

    match n {
        0 => 0,
        1 => sum,
        _ => {
            for _ in 2..=n {
                sum = sum + curr;
                (prev, curr) = (curr, prev + curr);
            }
            sum
        }
    }
}
