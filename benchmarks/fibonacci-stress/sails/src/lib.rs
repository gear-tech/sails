#![no_std]

use fibonacci_stress_core::{self as fibo_stress, FiboStressResult};

struct FibonacciStressService;

#[sails_rs::service]
impl FibonacciStressService {
    pub fn stress_fibo(&mut self, n: u32) -> FiboStressResult {
        fibo_stress::stress_fibo(n)
    }

    pub fn stress_bytes(&mut self, n: u32) -> &'static [u8] {
        fibo_stress::stress_bytes(n)
    }
}

pub struct FibonacciStressProgram;

#[sails_rs::program]
impl FibonacciStressProgram {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn fibo_stress(&self) -> FibonacciStressService {
        FibonacciStressService
    }
}
