#![no_std]

use core::ptr::addr_of;

use gstd::prelude::*;
use sails_rtl::gstd::gservice;

static mut COUNTER: Counter = Counter { count: 0 };

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct Counter {
    count: u32,
}

#[derive(Default)]
pub struct ReferenceService<'a> {
    name: &'a str,
}

#[gservice]
impl<'a> ReferenceService<'a> {
    pub const fn new(name: &'a str) -> Self {
        Self { name }
    }

    // Types returned by services don't have to be owned
    pub fn name(&self) -> &'a str {
        self.name
    }

    // They can also be static
    pub fn baked(&self) -> &'static str {
        "Static str!"
    }

    // Or references to structs and enums
    pub fn incr(&mut self) -> &'a Counter {
        unsafe {
            COUNTER.count += 1;
            &*addr_of!(COUNTER)
        }
    }

    // Something more complex
    pub fn add(&mut self, x: i32) -> Result<&Counter, &'static str> {
        if x < 0 {
            return Err("Can't add negative numbers");
        }

        unsafe {
            COUNTER.count += x as u32;
            Ok(&*addr_of!(COUNTER))
        }
    }

    // Note that returning types with lifetimes is not yet supported
}
