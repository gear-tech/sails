#![no_std]

use sails_rs::prelude::*;

struct AllocStressService;

#[sails_rs::service]
impl AllocStressService {
    pub fn alloc_stress(&mut self, n: u32) -> AllocStressResult {
        alloc_stress(n)
    }
}

#[derive(TypeInfo, Encode)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct AllocStressResult {
    pub inner: Vec<u8>,
}

/// Allocates a buffer of size equal to the sum of the first `n` Fibonacci numbers,
/// filling it with the byte `42`.
pub fn alloc_stress(n: u32) -> AllocStressResult {
    let sum = fibonacci_sum(n);
    let mut inner = Vec::with_capacity(sum as usize);
    (0..sum).for_each(|_| inner.push(42));

    AllocStressResult { inner }
}

/// Counts the sum of the first `n` Fibonacci numbers.
pub fn fibonacci_sum(n: u32) -> u32 {
    let (mut sum, mut prev, mut curr) = (0u32, 0u32, 1u32);

    match n {
        0 => 0,
        1 => sum,
        _ => {
            for _ in 2..=n {
                sum += curr;
                (prev, curr) = (curr, prev + curr);
            }
            sum
        }
    }
}

pub struct AllocStressProgram;

#[sails_rs::program]
impl AllocStressProgram {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn alloc_stress(&self) -> AllocStressService {
        AllocStressService
    }
}
