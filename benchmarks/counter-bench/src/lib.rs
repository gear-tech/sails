#![no_std]

use sails::prelude::*;

static mut COUNTER: u64 = 0;

pub struct CounterBenchService;

#[sails::service]
impl CounterBenchService {
    #[export]
    pub fn inc(&mut self) -> u64 {
        unsafe {
            let prev = COUNTER;
            COUNTER += 1;

            prev
        }
    }

    #[export]
    pub async fn inc_async(&mut self) -> u64 {
        unsafe {
            let prev = COUNTER;
            COUNTER += 1;

            prev
        }
    }
}

pub struct CounterBenchProgram;

#[sails::program]
impl CounterBenchProgram {
    pub fn new_for_bench() -> Self {
        Self
    }

    pub fn counter_bench(&self) -> CounterBenchService {
        CounterBenchService
    }
}
