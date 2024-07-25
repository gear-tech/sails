use core::ptr;
use sails_rs::prelude::*;

// This example makes use of fully incapsulated static state.
// It is safe to use this approach in WASM envrionment due to its single-threaded nature.
// But there might be issues in multi-threaded environment like testing.
// Tags: #state
static mut COUNTER: ReferenceCount = ReferenceCount(0);
static mut BYTES: Vec<u8> = Vec::new();

#[derive(Default)]
pub struct ReferenceService(());

#[service]
impl ReferenceService {
    pub fn baked(&self) -> &'static str {
        "Static str!"
    }

    pub fn incr(&mut self) -> &'static ReferenceCount {
        unsafe {
            COUNTER.0 += 1;
            &*ptr::addr_of!(COUNTER)
        }
    }

    pub fn add<'a>(&mut self, v: u32) -> &'a u32 {
        unsafe {
            COUNTER.0 += v;
            &COUNTER.0
        }
    }

    pub fn add_byte(&mut self, byte: u8) -> &'static [u8] {
        unsafe {
            BYTES.push(byte);
            &*ptr::addr_of!(BYTES)
        }
    }

    pub async fn first_byte<'a>(&self) -> Option<&'a u8> {
        unsafe { BYTES.first() }
    }

    pub async fn last_byte<'a>(&self) -> Option<&'a u8> {
        unsafe { BYTES.last() }
    }
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct ReferenceCount(u32);
