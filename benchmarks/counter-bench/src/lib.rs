#![no_std]

use sails_rs::prelude::*;

static mut COUNTER: u64 = 0;

pub struct CounterBenchService;

#[sails_rs::service]
impl CounterBenchService {
    #[export]
    pub fn inc(&mut self) -> u64 {
        // todo [sab]
        // let mut data: Vec<u8> = vec![];
        // for _ in 0..317810 {
        //     data.push(0);
        // }
        unsafe {
            let prev = COUNTER;
            COUNTER += 1;

            prev
        }
        // data.len() as u64
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

#[sails_rs::program]
impl CounterBenchProgram {
    pub fn new_for_bench() -> Self {
        Self
    }

    pub fn counter_bench(&self) -> CounterBenchService {
        CounterBenchService
    }
}
