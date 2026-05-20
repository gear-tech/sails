#![no_std]

pub const RAW_REPLY_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/raw_reply.wasm"));
pub const SAILS_WIRE_REPLY_WASM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/sails_wire_reply.wasm"));
