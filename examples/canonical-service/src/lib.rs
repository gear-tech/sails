#![no_std]

extern crate alloc;

use sails_rs::prelude::*;

pub struct DemoService {
    counter: u32,
}

impl Default for DemoService {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

#[service]
impl DemoService {
    #[export]
    pub fn increment(&mut self) -> u32 {
        self.counter = self.counter.saturating_add(1);
        self.counter
    }

    #[export]
    pub fn value(&self) -> u32 {
        self.counter
    }

    #[export]
    pub async fn reset(&mut self, value: u32) {
        self.counter = value;
    }
}
