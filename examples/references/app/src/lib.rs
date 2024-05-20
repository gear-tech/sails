#![no_std]

use core::ptr::addr_of;

use gstd::prelude::*;
use sails_rtl::gstd::gservice;

static mut COUNTER: Counter = Counter { count: 0 };
static mut BYTES: Vec<u8> = Vec::new();

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct Counter {
    count: u32,
}

#[derive(Default)]
pub struct ReferenceService;

#[gservice]
impl ReferenceService {
    pub const fn new() -> Self {
        Self
    }

    pub fn baked(&self) -> &'static str {
        "Static str!"
    }

    pub fn incr(&mut self) -> &'static Counter {
        unsafe {
            COUNTER.count += 1;
            &*addr_of!(COUNTER)
        }
    }

    pub fn add_byte(&mut self, byte: u8) -> &'static [u8] {
        unsafe {
            BYTES.push(byte);
            &*addr_of!(BYTES)
        }
    }
}
