#![no_std]

use sails_rs::prelude::*;
// use sails_rs::gstd::SyncCell;

// todo [sab] just use sync_cell
// static COUNTER: StaticCell<u64> = StaticCell::new(0);

static mut COUNTER: u64 = 0;

pub struct CounterBenchService;

#[sails_rs::service]
impl CounterBenchService {
    pub fn inc(&mut self) -> u64 {
        unsafe {
            let prev = COUNTER;
            COUNTER += 1;

            prev
        }
    }

    pub fn inc_async(&mut self) -> u64 {
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn counter_bench(&self) -> CounterBenchService {
        CounterBenchService
    }
}
