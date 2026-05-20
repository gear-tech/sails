#![no_std]

extern crate gstd;

pub const PROGRAM_NAME: &str = "noop_sails_hot";

const MAGIC_0: u8 = b'G';
const MAGIC_1: u8 = b'M';
const HEADER_LEN: usize = 16;
const ROUTE_NOOP: u8 = 1;
const ENTRY_NOOP: u16 = 0;
const INTERFACE_ID: [u8; 8] = [84, 87, 204, 48, 221, 148, 206, 41];

#[unsafe(no_mangle)]
extern "C" fn init() {}

#[unsafe(no_mangle)]
extern "C" fn handle() {
    gcore::msg::with_read_on_stack_or_heap(|payload| {
        let Ok(payload) = payload else {
            return;
        };
        let Some(entry_id) = read_header(payload) else {
            return;
        };
        if entry_id == ENTRY_NOOP {
            reply_bool(true);
        }
    });
}

fn read_header(payload: &[u8]) -> Option<u16> {
    if payload.len() < HEADER_LEN
        || payload[0] != MAGIC_0
        || payload[1] != MAGIC_1
        || payload[2] != 1
        || payload[3] != HEADER_LEN as u8
        || payload[4..12] != INTERFACE_ID
        || payload[14] != ROUTE_NOOP
    {
        return None;
    }

    Some(u16::from_le_bytes([payload[12], payload[13]]))
}

fn reply_bool(value: bool) {
    let mut payload = [0u8; 17];
    payload[0] = MAGIC_0;
    payload[1] = MAGIC_1;
    payload[2] = 1;
    payload[3] = HEADER_LEN as u8;
    payload[4..12].copy_from_slice(&INTERFACE_ID);
    payload[12..14].copy_from_slice(&ENTRY_NOOP.to_le_bytes());
    payload[14] = ROUTE_NOOP;
    payload[16] = u8::from(value);
    let _ = gcore::msg::reply(&payload, 0);
}
