#![no_std]

use sails_rs::prelude::*;
use sails_storage::gear::StaticVftStorage;

mod static_storage {
    include!(concat!(env!("OUT_DIR"), "/sails_static_storage.rs"));
}

pub const STATIC_MEMORY_END_PAGE: u32 = static_storage::STATIC_MEMORY_END_PAGE;
const VFT_LOG2_SLOTS: u8 = static_storage::BALANCES_LOG2_SLOTS;
const INITIAL_SUPPLY: u64 = 1_000_000_000_000;

type Storage = StaticVftStorage<VFT_LOG2_SLOTS, { static_storage::ALLOWANCES_LOG2_SLOTS }>;

pub struct MinimalVftService;

#[sails_rs::service]
impl MinimalVftService {
    #[export]
    pub fn approve(&mut self, spender: ActorId, amount: U256) -> bool {
        storage()
            .approve(sails_rs::gstd::msg::source(), spender, amount)
            .unwrap_or(false)
    }

    #[export]
    pub fn transfer(&mut self, to: ActorId, amount: U256) -> bool {
        storage()
            .transfer(sails_rs::gstd::msg::source(), to, amount)
            .unwrap_or(false)
    }

    #[export]
    pub fn transfer_from(&mut self, owner: ActorId, to: ActorId, amount: U256) -> bool {
        storage()
            .transfer_from(sails_rs::gstd::msg::source(), owner, to, amount)
            .unwrap_or(false)
    }

    #[export]
    pub fn balance_of(&mut self, owner: ActorId) -> U256 {
        storage().balance_of(owner).unwrap_or_else(|_| U256::zero())
    }

    #[export]
    pub fn allowance(&mut self, owner: ActorId, spender: ActorId) -> U256 {
        storage()
            .allowance(owner, spender)
            .unwrap_or_else(|_| U256::zero())
    }
}

pub struct MinimalVftSailsProgram;

#[sails_rs::program]
impl MinimalVftSailsProgram {
    pub fn new_for_bench() -> Self {
        let owner = sails_rs::gstd::msg::source();
        let _ = storage().mint(owner, U256::from(INITIAL_SUPPLY));
        Self
    }

    pub fn vft(&self) -> MinimalVftService {
        MinimalVftService
    }
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
