#![no_std]

use gprimitives::{ActorId, U256};
use sails_storage::__experimental::StaticVftStorage;

mod static_storage {
    include!(concat!(env!("OUT_DIR"), "/sails_static_storage.rs"));
}

pub const STATIC_MEMORY_END_PAGE: u32 = static_storage::STATIC_MEMORY_END_PAGE;
const VFT_LOG2_SLOTS: u8 = static_storage::BALANCES_LOG2_SLOTS;
const INITIAL_SUPPLY: u64 = 1_000_000_000_000;

const MAGIC_0: u8 = b'G';
const MAGIC_1: u8 = b'M';
const HEADER_LEN: usize = 16;
const ROUTE_VFT: u8 = 1;
const ENTRY_APPROVE: u16 = 0;
const ENTRY_TRANSFER: u16 = 1;
const ENTRY_TRANSFER_FROM: u16 = 2;
const ENTRY_BALANCE_OF: u16 = 3;
const ENTRY_ALLOWANCE: u16 = 4;
const INTERFACE_ID: [u8; 8] = [0x6d, 0x69, 0x6e, 0x76, 0x66, 0x74, 0x00, 0x01];

type Storage = StaticVftStorage<VFT_LOG2_SLOTS, { static_storage::ALLOWANCES_LOG2_SLOTS }>;

#[unsafe(no_mangle)]
extern "C" fn init() {
    let owner = gstd::msg::source();
    let _ = storage().mint(owner, U256::from(INITIAL_SUPPLY));
}

#[unsafe(no_mangle)]
extern "C" fn handle() {
    gcore::msg::with_read_on_stack_or_heap(|payload| {
        let Ok(payload) = payload else {
            reply_bool(ENTRY_TRANSFER, false);
            return;
        };

        let Some((entry_id, params)) = read_header(payload) else {
            reply_bool(ENTRY_TRANSFER, false);
            return;
        };

        match entry_id {
            ENTRY_APPROVE => {
                let Some((spender, amount)) = read_actor_u256(params) else {
                    reply_bool(entry_id, false);
                    return;
                };
                let changed = storage()
                    .approve(gstd::msg::source(), spender, amount)
                    .unwrap_or(false);
                reply_bool(entry_id, changed);
            }
            ENTRY_TRANSFER => {
                let Some((to, amount)) = read_actor_u256(params) else {
                    reply_bool(entry_id, false);
                    return;
                };
                let changed = storage()
                    .transfer(gstd::msg::source(), to, amount)
                    .unwrap_or(false);
                reply_bool(entry_id, changed);
            }
            ENTRY_TRANSFER_FROM => {
                let Some((owner, to, amount)) = read_actor_actor_u256(params) else {
                    reply_bool(entry_id, false);
                    return;
                };
                let changed = storage()
                    .transfer_from(gstd::msg::source(), owner, to, amount)
                    .unwrap_or(false);
                reply_bool(entry_id, changed);
            }
            ENTRY_BALANCE_OF => {
                let Some(owner) = read_actor(params) else {
                    reply_u256(entry_id, U256::zero());
                    return;
                };
                reply_u256(
                    entry_id,
                    storage().balance_of(owner).unwrap_or_else(|_| U256::zero()),
                );
            }
            ENTRY_ALLOWANCE => {
                let Some((owner, spender)) = read_actor_actor(params) else {
                    reply_u256(entry_id, U256::zero());
                    return;
                };
                reply_u256(
                    entry_id,
                    storage()
                        .allowance(owner, spender)
                        .unwrap_or_else(|_| U256::zero()),
                );
            }
            _ => reply_bool(entry_id, false),
        }
    });
}

fn storage() -> Storage {
    unsafe {
        Storage::new(
            static_storage::BALANCES_BASE,
            static_storage::ALLOWANCES_BASE,
        )
    }
    .expect("static VFT layout is valid")
}

fn read_header(payload: &[u8]) -> Option<(u16, &[u8])> {
    if payload.len() < HEADER_LEN
        || payload[0] != MAGIC_0
        || payload[1] != MAGIC_1
        || payload[2] != 1
        || payload[3] != HEADER_LEN as u8
        || payload[4..12] != INTERFACE_ID
        || payload[14] != ROUTE_VFT
    {
        return None;
    }

    Some((
        u16::from_le_bytes([payload[12], payload[13]]),
        &payload[16..],
    ))
}

fn read_actor(payload: &[u8]) -> Option<ActorId> {
    let bytes: [u8; 32] = payload.get(..32)?.try_into().ok()?;
    Some(ActorId::new(bytes))
}

fn read_actor_actor(payload: &[u8]) -> Option<(ActorId, ActorId)> {
    let owner = read_actor(payload)?;
    let spender = read_actor(payload.get(32..)?)?;
    Some((owner, spender))
}

fn read_u256(payload: &[u8]) -> Option<U256> {
    Some(U256::from_little_endian(payload.get(..32)?))
}

fn read_actor_u256(payload: &[u8]) -> Option<(ActorId, U256)> {
    let actor = read_actor(payload)?;
    let value = read_u256(payload.get(32..)?)?;
    Some((actor, value))
}

fn read_actor_actor_u256(payload: &[u8]) -> Option<(ActorId, ActorId, U256)> {
    let first = read_actor(payload)?;
    let second = read_actor(payload.get(32..)?)?;
    let value = read_u256(payload.get(64..)?)?;
    Some((first, second, value))
}

fn reply_bool(entry_id: u16, value: bool) {
    let mut payload = [0u8; 17];
    write_header(&mut payload, entry_id);
    payload[16] = u8::from(value);
    let _ = gcore::msg::reply(&payload, 0);
}

fn reply_u256(entry_id: u16, value: U256) {
    let mut payload = [0u8; 48];
    write_header(&mut payload, entry_id);
    value.to_little_endian(&mut payload[16..48]);
    let _ = gcore::msg::reply(&payload, 0);
}

fn write_header(payload: &mut [u8], entry_id: u16) {
    payload[0] = MAGIC_0;
    payload[1] = MAGIC_1;
    payload[2] = 1;
    payload[3] = HEADER_LEN as u8;
    payload[4..12].copy_from_slice(&INTERFACE_ID);
    payload[12..14].copy_from_slice(&entry_id.to_le_bytes());
    payload[14] = ROUTE_VFT;
}
