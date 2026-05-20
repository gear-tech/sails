#![no_std]

pub const PROGRAM_NAME: &str = "noop-gstd";

#[unsafe(no_mangle)]
extern "C" fn init() {}

#[unsafe(no_mangle)]
extern "C" fn handle() {
    let _ = gstd::msg::reply_bytes([1u8], 0);
}
