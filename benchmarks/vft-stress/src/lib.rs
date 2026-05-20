#![no_std]

use sails_rs::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    prelude::*,
};
use sails_storage::__experimental::{
    FixedAllowanceMap as SailsFixedAllowanceMap, FixedBalanceMap as SailsFixedBalanceMap,
    StaticAllowanceTable, StaticBalanceTable,
};

mod static_storage {
    include!(concat!(env!("OUT_DIR"), "/sails_static_storage.rs"));
}

const FIXED_CAPACITY: usize = 2048;

pub const STATIC_MEMORY_END_PAGE: u32 = static_storage::STATIC_MEMORY_END_PAGE;

type SailsStaticBalanceTable = StaticBalanceTable;
type SailsStaticAllowanceTable = StaticAllowanceTable;

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VftStorageBackend {
    BTree,
    HashMap,
    SailsFixed,
    SailsStatic,
    SailsStaticFast,
}

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VftTransferOp {
    Transfer,
    TransferFrom,
}

#[sails_type]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VftTransferResult {
    pub from_balance: U256,
    pub to_balance: U256,
    pub allowance: U256,
    pub balance_len: u32,
    pub allowance_len: u32,
    pub transferred: bool,
}

#[sails_type]
#[cfg(feature = "wasm-profile")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VftPhase {
    ProbeOverhead,
    NoopBody,
    EchoBody,
    KeyDerive,
    AllowanceGet,
    AllowancePut,
    BalanceGetFrom,
    BalanceGetTo,
    BalancePutFrom,
    BalancePutTo,
    BalanceTransfer,
    BalanceTransferFrom,
    ResultBuild,
}

#[sails_type]
#[cfg(feature = "wasm-profile")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VftPhaseGas {
    pub phase: VftPhase,
    pub gas: u64,
}

#[sails_type]
#[cfg(feature = "wasm-profile")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VftProfileResult {
    pub result: VftTransferResult,
    pub phases: Vec<VftPhaseGas>,
}

struct VftStressService<'a> {
    state: &'a RefCell<VftStressState>,
}

impl<'a> VftStressService<'a> {
    fn new(state: &'a RefCell<VftStressState>) -> Self {
        Self { state }
    }
}

#[cfg(not(feature = "wasm-profile"))]
#[sails_rs::service]
impl VftStressService<'_> {
    #[export]
    pub fn prepare_vft(&mut self, backend: VftStorageBackend, len: u32) -> VftTransferResult {
        self.state.borrow_mut().prepare_vft(backend, len)
    }

    #[export]
    pub fn bench_vft_transfer(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
    ) -> VftTransferResult {
        self.state
            .borrow_mut()
            .bench_vft_transfer(backend, op, seed)
    }

    #[export]
    pub fn bench_vft_transfer_fresh_bool(&mut self, backend: VftStorageBackend, seed: u32) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_transfer_fresh_bool(backend, seed)
    }

    #[export]
    pub fn bench_vft_approve_bool(
        &mut self,
        backend: VftStorageBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_approve_bool(backend, owner_seed, spender_seed)
    }
}

#[cfg(feature = "wasm-profile")]
#[sails_rs::service]
impl VftStressService<'_> {
    #[cfg(feature = "wasm-profile")]
    #[export]
    pub fn bench_noop(&mut self) -> bool {
        self.state.borrow_mut().bench_noop()
    }

    #[cfg(feature = "wasm-profile")]
    #[export]
    pub fn bench_echo_vft_args(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
    ) -> bool {
        self.state
            .borrow_mut()
            .bench_echo_vft_args(backend, op, seed)
    }

    #[export]
    pub fn prepare_vft(&mut self, backend: VftStorageBackend, len: u32) -> VftTransferResult {
        self.state.borrow_mut().prepare_vft(backend, len)
    }

    #[export]
    pub fn bench_vft_transfer(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
    ) -> VftTransferResult {
        self.state
            .borrow_mut()
            .bench_vft_transfer(backend, op, seed)
    }

    #[export]
    pub fn bench_vft_transfer_fresh_bool(&mut self, backend: VftStorageBackend, seed: u32) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_transfer_fresh_bool(backend, seed)
    }

    #[export]
    pub fn bench_vft_approve_bool(
        &mut self,
        backend: VftStorageBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_approve_bool(backend, owner_seed, spender_seed)
    }

    #[cfg(feature = "wasm-profile")]
    #[export]
    pub fn bench_vft_transfer_profile(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
    ) -> VftProfileResult {
        self.state
            .borrow_mut()
            .bench_vft_transfer_profile(backend, op, seed)
    }

    #[cfg(feature = "wasm-profile")]
    #[export]
    pub fn bench_vft_transfer_fresh_profile(
        &mut self,
        backend: VftStorageBackend,
        seed: u32,
    ) -> VftProfileResult {
        self.state
            .borrow_mut()
            .bench_vft_transfer_fresh_profile(backend, seed)
    }

    #[cfg(feature = "wasm-profile")]
    #[export]
    pub fn bench_vft_approve_profile(
        &mut self,
        backend: VftStorageBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> VftProfileResult {
        self.state
            .borrow_mut()
            .bench_vft_approve_profile(backend, owner_seed, spender_seed)
    }
}

pub struct VftStressProgram {
    state: RefCell<VftStressState>,
}

#[sails_rs::program]
impl VftStressProgram {
    pub fn new_for_bench() -> Self {
        Self {
            state: RefCell::new(VftStressState::new()),
        }
    }

    pub fn vft_stress(&self) -> VftStressService<'_> {
        VftStressService::new(&self.state)
    }
}

struct VftStressState {
    storage: ActiveVftStorage,
}

enum ActiveVftStorage {
    Empty,
    BTree {
        balances: BTreeMap<ActorId, U256>,
        allowances: BTreeMap<(ActorId, ActorId), U256>,
    },
    HashMap {
        balances: HashMap<ActorId, U256>,
        allowances: HashMap<(ActorId, ActorId), U256>,
    },
    SailsFixed {
        balances: SailsFixedBalanceMap<FIXED_CAPACITY>,
        allowances: SailsFixedAllowanceMap<FIXED_CAPACITY>,
    },
    SailsStatic {
        balances: SailsStaticBalanceMap,
        allowances: SailsStaticAllowanceMap,
    },
}

impl VftStressState {
    fn new() -> Self {
        Self {
            storage: ActiveVftStorage::Empty,
        }
    }

    #[cfg(feature = "wasm-profile")]
    fn bench_noop(&mut self) -> bool {
        true
    }

    #[cfg(feature = "wasm-profile")]
    fn bench_echo_vft_args(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
    ) -> bool {
        seed > 0
            && matches!(
                backend,
                VftStorageBackend::BTree
                    | VftStorageBackend::HashMap
                    | VftStorageBackend::SailsFixed
                    | VftStorageBackend::SailsStatic
                    | VftStorageBackend::SailsStaticFast
            )
            && matches!(op, VftTransferOp::Transfer | VftTransferOp::TransferFrom)
    }

    fn prepare_vft(&mut self, backend: VftStorageBackend, len: u32) -> VftTransferResult {
        assert!((len as usize) < FIXED_CAPACITY);
        self.clear_vft(backend);

        for seed in 1..=len {
            self.vft_insert_balance(backend, actor_for_seed(seed), vft_balance_for_seed(seed));
            let owner = actor_for_seed(seed);
            let spender = vft_spender_for_seed(seed);
            self.vft_insert_allowance(backend, owner, spender, vft_allowance_for_seed(seed));
        }

        VftTransferResult {
            from_balance: vft_balance_for_seed(len),
            to_balance: U256::zero(),
            allowance: U256::zero(),
            balance_len: self.vft_balance_len(backend),
            allowance_len: self.vft_allowance_len(backend),
            transferred: false,
        }
    }

    fn bench_vft_transfer(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
    ) -> VftTransferResult {
        self.bench_vft_transfer_with_to_seed(backend, op, seed, vft_recipient_seed(seed))
    }

    fn bench_vft_transfer_fresh_bool(&mut self, backend: VftStorageBackend, seed: u32) -> bool {
        let len = self.vft_balance_len(backend);
        assert!(len > 1);
        let to_seed = len
            .checked_add(seed)
            .and_then(|value| value.checked_add(1))
            .expect("fresh VFT recipient seed overflow");
        self.apply_vft_transfer_bool(backend, VftTransferOp::Transfer, seed, to_seed)
    }

    fn bench_vft_approve_bool(
        &mut self,
        backend: VftStorageBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> bool {
        let owner = actor_for_seed(owner_seed);
        let spender = vft_spender_for_seed(spender_seed);
        if owner == spender {
            return false;
        }

        self.vft_insert_allowance(
            backend,
            owner,
            spender,
            vft_allowance_for_seed(spender_seed),
        );
        true
    }

    #[cfg(feature = "wasm-profile")]
    fn bench_vft_transfer_profile(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
    ) -> VftProfileResult {
        self.bench_vft_transfer_profile_with_to_seed(backend, op, seed, vft_recipient_seed(seed))
    }

    #[cfg(feature = "wasm-profile")]
    fn bench_vft_transfer_fresh_profile(
        &mut self,
        backend: VftStorageBackend,
        seed: u32,
    ) -> VftProfileResult {
        let len = self.vft_balance_len(backend);
        assert!(len > 1);
        let to_seed = len
            .checked_add(seed)
            .and_then(|value| value.checked_add(1))
            .expect("fresh VFT recipient seed overflow");
        self.bench_vft_transfer_profile_with_to_seed(
            backend,
            VftTransferOp::Transfer,
            seed,
            to_seed,
        )
    }

    #[cfg(feature = "wasm-profile")]
    fn bench_vft_approve_profile(
        &mut self,
        backend: VftStorageBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> VftProfileResult {
        let mut phases = VftPhaseRecorder::new();
        let owner = actor_for_seed(owner_seed);
        let spender = vft_spender_for_seed(spender_seed);
        let value = vft_allowance_for_seed(spender_seed);
        phases.checkpoint(VftPhase::KeyDerive);

        let approved = if owner == spender {
            false
        } else {
            self.vft_insert_allowance(backend, owner, spender, value);
            true
        };
        phases.checkpoint(VftPhase::AllowancePut);

        let result = VftTransferResult {
            from_balance: U256::zero(),
            to_balance: U256::zero(),
            allowance: self
                .vft_get_allowance(backend, &owner, &spender)
                .unwrap_or_else(U256::zero),
            balance_len: self.vft_balance_len(backend),
            allowance_len: self.vft_allowance_len(backend),
            transferred: approved,
        };
        phases.checkpoint(VftPhase::ResultBuild);

        VftProfileResult {
            result,
            phases: phases.finish(),
        }
    }

    fn clear_vft(&mut self, backend: VftStorageBackend) {
        self.storage = match backend {
            VftStorageBackend::BTree => ActiveVftStorage::BTree {
                balances: BTreeMap::new(),
                allowances: BTreeMap::new(),
            },
            VftStorageBackend::HashMap => ActiveVftStorage::HashMap {
                balances: HashMap::new(),
                allowances: HashMap::new(),
            },
            VftStorageBackend::SailsFixed => ActiveVftStorage::SailsFixed {
                balances: SailsFixedBalanceMap::new(),
                allowances: SailsFixedAllowanceMap::new(),
            },
            VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast => {
                ActiveVftStorage::SailsStatic {
                    balances: SailsStaticBalanceMap::new(),
                    allowances: SailsStaticAllowanceMap::new(),
                }
            }
        };
    }

    fn vft_balance_len(&self, backend: VftStorageBackend) -> u32 {
        match (backend, &self.storage) {
            (VftStorageBackend::BTree, ActiveVftStorage::BTree { balances, .. }) => {
                balances.len() as u32
            }
            (VftStorageBackend::HashMap, ActiveVftStorage::HashMap { balances, .. }) => {
                balances.len() as u32
            }
            (VftStorageBackend::SailsFixed, ActiveVftStorage::SailsFixed { balances, .. }) => {
                balances.len() as u32
            }
            (
                VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast,
                ActiveVftStorage::SailsStatic { balances, .. },
            ) => balances.len(),
            _ => 0,
        }
    }

    fn vft_allowance_len(&self, backend: VftStorageBackend) -> u32 {
        match (backend, &self.storage) {
            (VftStorageBackend::BTree, ActiveVftStorage::BTree { allowances, .. }) => {
                allowances.len() as u32
            }
            (VftStorageBackend::HashMap, ActiveVftStorage::HashMap { allowances, .. }) => {
                allowances.len() as u32
            }
            (VftStorageBackend::SailsFixed, ActiveVftStorage::SailsFixed { allowances, .. }) => {
                allowances.len() as u32
            }
            (
                VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast,
                ActiveVftStorage::SailsStatic { allowances, .. },
            ) => allowances.len(),
            _ => 0,
        }
    }

    fn vft_get_balance(&self, backend: VftStorageBackend, key: &ActorId) -> Option<U256> {
        match (backend, &self.storage) {
            (VftStorageBackend::BTree, ActiveVftStorage::BTree { balances, .. }) => {
                balances.get(key).copied()
            }
            (VftStorageBackend::HashMap, ActiveVftStorage::HashMap { balances, .. }) => {
                balances.get(key).copied()
            }
            (VftStorageBackend::SailsFixed, ActiveVftStorage::SailsFixed { balances, .. }) => {
                balances
                    .get_actor_u256(key)
                    .expect("sails-storage fixed VFT balance map failed")
            }
            (
                VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast,
                ActiveVftStorage::SailsStatic { balances, .. },
            ) => balances
                .get(key)
                .expect("sails-storage static VFT balance map failed"),
            _ => None,
        }
    }

    fn vft_get_allowance(
        &self,
        backend: VftStorageBackend,
        owner: &ActorId,
        spender: &ActorId,
    ) -> Option<U256> {
        match (backend, &self.storage) {
            (VftStorageBackend::BTree, ActiveVftStorage::BTree { allowances, .. }) => {
                allowances.get(&(*owner, *spender)).copied()
            }
            (VftStorageBackend::HashMap, ActiveVftStorage::HashMap { allowances, .. }) => {
                allowances.get(&(*owner, *spender)).copied()
            }
            (VftStorageBackend::SailsFixed, ActiveVftStorage::SailsFixed { allowances, .. }) => {
                allowances
                    .get_allowance_u256(owner, spender)
                    .expect("sails-storage fixed VFT allowance map failed")
            }
            (
                VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast,
                ActiveVftStorage::SailsStatic { allowances, .. },
            ) => allowances
                .get(owner, spender)
                .expect("sails-storage static VFT allowance map failed"),
            _ => None,
        }
    }

    fn vft_insert_balance(
        &mut self,
        backend: VftStorageBackend,
        key: ActorId,
        value: U256,
    ) -> Option<U256> {
        match (backend, &mut self.storage) {
            (VftStorageBackend::BTree, ActiveVftStorage::BTree { balances, .. }) => {
                balances.insert(key, value)
            }
            (VftStorageBackend::HashMap, ActiveVftStorage::HashMap { balances, .. }) => {
                balances.insert(key, value)
            }
            (VftStorageBackend::SailsFixed, ActiveVftStorage::SailsFixed { balances, .. }) => {
                balances
                    .insert_actor_u256(key, value)
                    .expect("sails-storage fixed VFT balance map failed")
            }
            (
                VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast,
                ActiveVftStorage::SailsStatic { balances, .. },
            ) => balances
                .try_insert(key, value)
                .expect("sails-storage static VFT balance map failed"),
            _ => panic!("VFT storage backend is not prepared"),
        }
    }

    fn vft_insert_allowance(
        &mut self,
        backend: VftStorageBackend,
        owner: ActorId,
        spender: ActorId,
        value: U256,
    ) -> Option<U256> {
        match (backend, &mut self.storage) {
            (VftStorageBackend::BTree, ActiveVftStorage::BTree { allowances, .. }) => {
                allowances.insert((owner, spender), value)
            }
            (VftStorageBackend::HashMap, ActiveVftStorage::HashMap { allowances, .. }) => {
                allowances.insert((owner, spender), value)
            }
            (VftStorageBackend::SailsFixed, ActiveVftStorage::SailsFixed { allowances, .. }) => {
                allowances
                    .insert_allowance_u256(owner, spender, value)
                    .expect("sails-storage fixed VFT allowance map failed")
            }
            (
                VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast,
                ActiveVftStorage::SailsStatic { allowances, .. },
            ) => allowances
                .try_insert(owner, spender, value)
                .expect("sails-storage static VFT allowance map failed"),
            _ => panic!("VFT storage backend is not prepared"),
        }
    }

    fn apply_vft_transfer_bool(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
        to_seed: u32,
    ) -> bool {
        if matches!(
            backend,
            VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast
        ) {
            return self.bench_vft_static_transfer_bool_with_to_seed(
                op,
                seed,
                to_seed,
                matches!(backend, VftStorageBackend::SailsStaticFast),
            );
        }

        self.bench_vft_transfer_with_to_seed(backend, op, seed, to_seed)
            .transferred
    }

    #[cfg(feature = "wasm-profile")]
    fn bench_vft_transfer_profile_with_to_seed(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
        to_seed: u32,
    ) -> VftProfileResult {
        if matches!(
            backend,
            VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast
        ) {
            return self.bench_vft_static_transfer_profile_with_to_seed(
                op,
                seed,
                to_seed,
                matches!(backend, VftStorageBackend::SailsStaticFast),
            );
        }

        let mut phases = VftPhaseRecorder::new();
        let amount = vft_transfer_amount(seed);
        let from = actor_for_seed(seed);
        let to = actor_for_seed(to_seed);
        let spender = vft_spender_for_seed(seed);
        phases.checkpoint(VftPhase::KeyDerive);

        let mut allowance_key = None;
        let mut transferred = false;

        if matches!(op, VftTransferOp::TransferFrom) {
            allowance_key = Some(spender);
            let allowance = self
                .vft_get_allowance(backend, &from, &spender)
                .unwrap_or_else(U256::zero);
            phases.checkpoint(VftPhase::AllowanceGet);
            if allowance < amount {
                let result = self.vft_result(backend, &from, &to, allowance_key.as_ref(), false);
                phases.checkpoint(VftPhase::ResultBuild);
                return VftProfileResult {
                    result,
                    phases: phases.finish(),
                };
            }
            self.vft_insert_allowance(backend, from, spender, allowance - amount);
            phases.checkpoint(VftPhase::AllowancePut);
        }

        let from_balance = self
            .vft_get_balance(backend, &from)
            .unwrap_or_else(U256::zero);
        phases.checkpoint(VftPhase::BalanceGetFrom);
        if from_balance >= amount {
            let to_balance = self
                .vft_get_balance(backend, &to)
                .unwrap_or_else(U256::zero);
            phases.checkpoint(VftPhase::BalanceGetTo);
            self.vft_insert_balance(backend, from, from_balance - amount);
            phases.checkpoint(VftPhase::BalancePutFrom);
            self.vft_insert_balance(backend, to, to_balance + amount);
            phases.checkpoint(VftPhase::BalancePutTo);
            transferred = true;
        }

        let result = self.vft_result(backend, &from, &to, allowance_key.as_ref(), transferred);
        phases.checkpoint(VftPhase::ResultBuild);
        VftProfileResult {
            result,
            phases: phases.finish(),
        }
    }

    fn bench_vft_transfer_with_to_seed(
        &mut self,
        backend: VftStorageBackend,
        op: VftTransferOp,
        seed: u32,
        to_seed: u32,
    ) -> VftTransferResult {
        if matches!(
            backend,
            VftStorageBackend::SailsStatic | VftStorageBackend::SailsStaticFast
        ) {
            return self.bench_vft_static_transfer_with_to_seed(
                op,
                seed,
                to_seed,
                matches!(backend, VftStorageBackend::SailsStaticFast),
            );
        }

        let amount = vft_transfer_amount(seed);
        let from = actor_for_seed(seed);
        let to = actor_for_seed(to_seed);
        let spender = vft_spender_for_seed(seed);

        if matches!(op, VftTransferOp::TransferFrom) {
            let allowance = self
                .vft_get_allowance(backend, &from, &spender)
                .unwrap_or_else(U256::zero);
            if allowance < amount {
                return self.vft_result(backend, &from, &to, Some(&spender), false);
            }
            self.vft_insert_allowance(backend, from, spender, allowance - amount);
        }

        let from_balance = self
            .vft_get_balance(backend, &from)
            .unwrap_or_else(U256::zero);
        if from_balance < amount {
            let allowance_key = matches!(op, VftTransferOp::TransferFrom).then_some(&spender);
            return self.vft_result(backend, &from, &to, allowance_key, false);
        }

        let to_balance = self
            .vft_get_balance(backend, &to)
            .unwrap_or_else(U256::zero);
        self.vft_insert_balance(backend, from, from_balance - amount);
        self.vft_insert_balance(backend, to, to_balance + amount);
        let allowance_key = matches!(op, VftTransferOp::TransferFrom).then_some(&spender);
        self.vft_result(backend, &from, &to, allowance_key, true)
    }

    fn bench_vft_static_transfer_with_to_seed(
        &mut self,
        op: VftTransferOp,
        seed: u32,
        to_seed: u32,
        unchecked: bool,
    ) -> VftTransferResult {
        let amount = vft_transfer_amount(seed);
        let from = actor_for_seed(seed);
        let to = actor_for_seed(to_seed);
        let spender = vft_spender_for_seed(seed);
        let ActiveVftStorage::SailsStatic {
            balances,
            allowances,
        } = &mut self.storage
        else {
            panic!("VFT static storage backend is not prepared");
        };

        match op {
            VftTransferOp::Transfer => {
                let outcome = balances
                    .transfer(from, to, amount, unchecked)
                    .expect("sails-storage static VFT balance transfer failed");
                match outcome {
                    Some(outcome) => VftTransferResult {
                        from_balance: outcome.from_balance,
                        to_balance: outcome.to_balance,
                        allowance: U256::zero(),
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: true,
                    },
                    None => VftTransferResult {
                        from_balance: balances
                            .get(&from)
                            .unwrap_or(None)
                            .unwrap_or_else(U256::zero),
                        to_balance: balances.get(&to).unwrap_or(None).unwrap_or_else(U256::zero),
                        allowance: U256::zero(),
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: false,
                    },
                }
            }
            VftTransferOp::TransferFrom => {
                let outcome = balances
                    .transfer_from(allowances, from, spender, to, amount, unchecked)
                    .expect("sails-storage static VFT transfer-from failed");
                match outcome {
                    Some(outcome) => VftTransferResult {
                        from_balance: outcome.from_balance,
                        to_balance: outcome.to_balance,
                        allowance: outcome.allowance,
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: true,
                    },
                    None => VftTransferResult {
                        from_balance: balances
                            .get(&from)
                            .unwrap_or(None)
                            .unwrap_or_else(U256::zero),
                        to_balance: balances.get(&to).unwrap_or(None).unwrap_or_else(U256::zero),
                        allowance: allowances
                            .get(&from, &spender)
                            .unwrap_or(None)
                            .unwrap_or_else(U256::zero),
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: false,
                    },
                }
            }
        }
    }

    fn bench_vft_static_transfer_bool_with_to_seed(
        &mut self,
        op: VftTransferOp,
        seed: u32,
        to_seed: u32,
        unchecked: bool,
    ) -> bool {
        let amount = vft_transfer_amount(seed);
        let from = actor_for_seed(seed);
        let to = actor_for_seed(to_seed);
        let spender = vft_spender_for_seed(seed);
        let ActiveVftStorage::SailsStatic {
            balances,
            allowances,
        } = &mut self.storage
        else {
            panic!("VFT static storage backend is not prepared");
        };

        match op {
            VftTransferOp::Transfer => balances
                .transfer(from, to, amount, unchecked)
                .expect("sails-storage static VFT balance transfer failed")
                .is_some(),
            VftTransferOp::TransferFrom => balances
                .transfer_from(allowances, from, spender, to, amount, unchecked)
                .expect("sails-storage static VFT transfer-from failed")
                .is_some(),
        }
    }

    #[cfg(feature = "wasm-profile")]
    fn bench_vft_static_transfer_profile_with_to_seed(
        &mut self,
        op: VftTransferOp,
        seed: u32,
        to_seed: u32,
        unchecked: bool,
    ) -> VftProfileResult {
        let mut phases = VftPhaseRecorder::new();
        let amount = vft_transfer_amount(seed);
        let from = actor_for_seed(seed);
        let to = actor_for_seed(to_seed);
        let spender = vft_spender_for_seed(seed);
        phases.checkpoint(VftPhase::KeyDerive);

        let ActiveVftStorage::SailsStatic {
            balances,
            allowances,
        } = &mut self.storage
        else {
            panic!("VFT static storage backend is not prepared");
        };

        let result = match op {
            VftTransferOp::Transfer => {
                let outcome = balances
                    .transfer(from, to, amount, unchecked)
                    .expect("sails-storage static VFT balance transfer failed");
                phases.checkpoint(VftPhase::BalanceTransfer);
                match outcome {
                    Some(outcome) => VftTransferResult {
                        from_balance: outcome.from_balance,
                        to_balance: outcome.to_balance,
                        allowance: U256::zero(),
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: true,
                    },
                    None => VftTransferResult {
                        from_balance: balances
                            .get(&from)
                            .unwrap_or(None)
                            .unwrap_or_else(U256::zero),
                        to_balance: balances.get(&to).unwrap_or(None).unwrap_or_else(U256::zero),
                        allowance: U256::zero(),
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: false,
                    },
                }
            }
            VftTransferOp::TransferFrom => {
                let outcome = balances
                    .transfer_from(allowances, from, spender, to, amount, unchecked)
                    .expect("sails-storage static VFT transfer-from failed");
                phases.checkpoint(VftPhase::BalanceTransferFrom);
                match outcome {
                    Some(outcome) => VftTransferResult {
                        from_balance: outcome.from_balance,
                        to_balance: outcome.to_balance,
                        allowance: outcome.allowance,
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: true,
                    },
                    None => VftTransferResult {
                        from_balance: balances
                            .get(&from)
                            .unwrap_or(None)
                            .unwrap_or_else(U256::zero),
                        to_balance: balances.get(&to).unwrap_or(None).unwrap_or_else(U256::zero),
                        allowance: allowances
                            .get(&from, &spender)
                            .unwrap_or(None)
                            .unwrap_or_else(U256::zero),
                        balance_len: balances.len(),
                        allowance_len: allowances.len(),
                        transferred: false,
                    },
                }
            }
        };
        phases.checkpoint(VftPhase::ResultBuild);

        VftProfileResult {
            result,
            phases: phases.finish(),
        }
    }

    fn vft_result(
        &self,
        backend: VftStorageBackend,
        from: &ActorId,
        to: &ActorId,
        spender: Option<&ActorId>,
        transferred: bool,
    ) -> VftTransferResult {
        VftTransferResult {
            from_balance: self
                .vft_get_balance(backend, from)
                .unwrap_or_else(U256::zero),
            to_balance: self.vft_get_balance(backend, to).unwrap_or_else(U256::zero),
            allowance: spender
                .and_then(|spender| self.vft_get_allowance(backend, from, spender))
                .unwrap_or_else(U256::zero),
            balance_len: self.vft_balance_len(backend),
            allowance_len: self.vft_allowance_len(backend),
            transferred,
        }
    }
}

#[cfg(feature = "wasm-profile")]
struct VftPhaseRecorder {
    previous_gas: u64,
    phases: Vec<VftPhaseGas>,
}

#[cfg(feature = "wasm-profile")]
impl VftPhaseRecorder {
    fn new() -> Self {
        let overhead = gas_probe_overhead();
        let previous_gas = gas_available();
        let mut phases = Vec::with_capacity(8);
        if overhead > 0 {
            phases.push(VftPhaseGas {
                phase: VftPhase::ProbeOverhead,
                gas: overhead,
            });
        }
        Self {
            previous_gas,
            phases,
        }
    }

    fn checkpoint(&mut self, phase: VftPhase) {
        let current = gas_available();
        self.phases.push(VftPhaseGas {
            phase,
            gas: self.previous_gas.saturating_sub(current),
        });
        self.previous_gas = current;
    }

    fn finish(self) -> Vec<VftPhaseGas> {
        self.phases
    }
}

#[cfg(feature = "wasm-profile")]
fn gas_probe_overhead() -> u64 {
    let before = gas_available();
    let after = gas_available();
    before.saturating_sub(after)
}

#[cfg(target_arch = "wasm32")]
#[cfg(feature = "wasm-profile")]
fn gas_available() -> u64 {
    Syscall::gas_available()
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "wasm-profile")]
fn gas_available() -> u64 {
    0
}

struct SailsStaticBalanceMap {
    #[cfg(not(target_arch = "wasm32"))]
    bytes: [u8; static_storage::SAILS_STATIC_BALANCES_BYTES],
    len: u32,
}

impl SailsStaticBalanceMap {
    fn new() -> Self {
        let map = Self {
            #[cfg(not(target_arch = "wasm32"))]
            bytes: [0; static_storage::SAILS_STATIC_BALANCES_BYTES],
            len: 0,
        };
        #[cfg(target_arch = "wasm32")]
        map.table()
            .clear()
            .expect("sails static balance table can be cleared");
        map
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn get(&self, key: &ActorId) -> Result<Option<U256>, sails_storage::TableError> {
        self.table().get_actor_u256(key)
    }

    fn try_insert(
        &mut self,
        key: ActorId,
        value: U256,
    ) -> Result<Option<U256>, sails_storage::TableError> {
        let previous = self.table_mut().insert_actor_u256(key, value)?;
        if previous.is_none() {
            self.len += 1;
        }

        Ok(previous)
    }

    fn transfer(
        &mut self,
        from: ActorId,
        to: ActorId,
        amount: U256,
        unchecked: bool,
    ) -> Result<Option<sails_storage::__experimental::StaticActorU256Transfer>, sails_storage::TableError>
    {
        let outcome = if unchecked {
            unsafe {
                self.table_mut()
                    .transfer_actor_u256_nonzero_distinct_unchecked(from, to, amount)?
            }
        } else {
            self.table_mut().transfer_actor_u256(from, to, amount)?
        };
        if outcome.map_or(false, |outcome| outcome.inserted_to) {
            self.len += 1;
        }
        Ok(outcome)
    }

    fn transfer_from(
        &mut self,
        allowances: &SailsStaticAllowanceMap,
        owner: ActorId,
        spender: ActorId,
        to: ActorId,
        amount: U256,
        unchecked: bool,
    ) -> Result<Option<sails_storage::__experimental::StaticActorU256TransferFrom>, sails_storage::TableError>
    {
        let outcome = if unchecked {
            unsafe {
                self.table_mut()
                    .transfer_actor_u256_from_nonzero_distinct_unchecked(
                        &allowances.table(),
                        owner,
                        spender,
                        to,
                        amount,
                    )?
            }
        } else {
            self.table_mut().transfer_actor_u256_from(
                &allowances.table(),
                owner,
                spender,
                to,
                amount,
            )?
        };
        if outcome.map_or(false, |outcome| outcome.inserted_to) {
            self.len += 1;
        }
        Ok(outcome)
    }

    fn table(&self) -> SailsStaticBalanceTable {
        #[cfg(target_arch = "wasm32")]
        let base = static_storage::SAILS_STATIC_BALANCES_BASE;
        #[cfg(not(target_arch = "wasm32"))]
        let base = self.bytes.as_ptr() as usize;

        unsafe {
            SailsStaticBalanceTable::new(base, static_storage::SAILS_STATIC_BALANCES_SLOTS)
                .expect("sails static balance layout is valid")
        }
    }

    fn table_mut(&mut self) -> SailsStaticBalanceTable {
        #[cfg(target_arch = "wasm32")]
        let base = static_storage::SAILS_STATIC_BALANCES_BASE;
        #[cfg(not(target_arch = "wasm32"))]
        let base = self.bytes.as_mut_ptr() as usize;

        unsafe {
            SailsStaticBalanceTable::new(base, static_storage::SAILS_STATIC_BALANCES_SLOTS)
                .expect("sails static balance layout is valid")
        }
    }
}

struct SailsStaticAllowanceMap {
    #[cfg(not(target_arch = "wasm32"))]
    bytes: [u8; static_storage::SAILS_STATIC_ALLOWANCES_BYTES],
    len: u32,
}

impl SailsStaticAllowanceMap {
    fn new() -> Self {
        let map = Self {
            #[cfg(not(target_arch = "wasm32"))]
            bytes: [0; static_storage::SAILS_STATIC_ALLOWANCES_BYTES],
            len: 0,
        };
        #[cfg(target_arch = "wasm32")]
        map.table()
            .clear()
            .expect("sails static allowance table can be cleared");
        map
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn get(
        &self,
        owner: &ActorId,
        spender: &ActorId,
    ) -> Result<Option<U256>, sails_storage::TableError> {
        self.table().get_allowance_u256(owner, spender)
    }

    fn try_insert(
        &mut self,
        owner: ActorId,
        spender: ActorId,
        value: U256,
    ) -> Result<Option<U256>, sails_storage::TableError> {
        let previous = self
            .table_mut()
            .insert_allowance_u256(owner, spender, value)?;
        if previous.is_none() {
            self.len += 1;
        }

        Ok(previous)
    }

    fn table(&self) -> SailsStaticAllowanceTable {
        #[cfg(target_arch = "wasm32")]
        let base = static_storage::SAILS_STATIC_ALLOWANCES_BASE;
        #[cfg(not(target_arch = "wasm32"))]
        let base = self.bytes.as_ptr() as usize;

        unsafe {
            SailsStaticAllowanceTable::new(base, static_storage::SAILS_STATIC_ALLOWANCES_SLOTS)
                .expect("sails static allowance layout is valid")
        }
    }

    fn table_mut(&mut self) -> SailsStaticAllowanceTable {
        #[cfg(target_arch = "wasm32")]
        let base = static_storage::SAILS_STATIC_ALLOWANCES_BASE;
        #[cfg(not(target_arch = "wasm32"))]
        let base = self.bytes.as_mut_ptr() as usize;

        unsafe {
            SailsStaticAllowanceTable::new(base, static_storage::SAILS_STATIC_ALLOWANCES_SLOTS)
                .expect("sails static allowance layout is valid")
        }
    }
}

fn actor_for_seed(seed: u32) -> ActorId {
    ActorId::from(seed as u64 + 1)
}

fn vft_balance_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1_000_000)
}

fn vft_allowance_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 10_000)
}

fn vft_transfer_amount(seed: u32) -> U256 {
    U256::from((seed % 7) as u64 + 1)
}

fn vft_recipient_seed(seed: u32) -> u32 {
    seed.saturating_add(10_000_000)
}

fn vft_spender_for_seed(seed: u32) -> ActorId {
    ActorId::from(seed as u64 + 20_000_000)
}
