#![no_std]

use sails_rs::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    prelude::*,
};
use sails_storage::gear::{
    StaticActorIdU256Map as SailsStaticActorIdU256Map,
    StaticActorTagU64Map as SailsStaticActorTagU64Map,
    StaticActorTagU256Map as SailsStaticActorTagU256Map,
    StaticAllowanceTable as SailsStaticAllowanceTable,
    StaticAllowanceU256Map as SailsStaticAllowanceU256Map,
    StaticBalanceTable as SailsStaticBalanceTable,
    StaticControlActorIdU256Map as SailsStaticControlActorIdU256Map,
    StaticGroupedControlActorIdU256Map as SailsStaticGroupedControlActorIdU256Map,
    StaticMixedActorIdU256Map as SailsStaticMixedActorIdU256Map,
    StaticPageLocalActorIdU256Map as SailsStaticPageLocalActorIdU256Map,
    StaticVftAccountStorage as SailsStaticVftAccountStorage,
};

mod static_storage {
    include!(concat!(env!("OUT_DIR"), "/sails_static_storage.rs"));
}

const GENERIC_BALANCE_CAPACITY: u32 = static_storage::MILLION_BALANCES_SLOTS as u32;
const GENERIC_ALLOWANCE_CAPACITY: u32 = static_storage::MILLION_ALLOWANCES_SLOTS as u32;
const WAT_ACTOR_BALANCE_CAPACITY: u32 = static_storage::WAT_ACTOR_BALANCES_SLOTS as u32;
const MIXED_ACTOR_BALANCE_CAPACITY: u32 = static_storage::MIXED_ACTOR_BALANCES_SLOTS as u32;
const TAG_ACTOR_BALANCE_CAPACITY: u32 = static_storage::MILLION_BALANCES_SLOTS as u32;
const TAG_U64_ACTOR_BALANCE_CAPACITY: u32 = (static_storage::MILLION_BALANCES_SLOTS * 4) as u32;
const WAT_ALLOWANCE_CAPACITY: u32 = static_storage::WAT_ALLOWANCES_SLOTS as u32;
const CONTROL_ACTOR_BALANCE_CAPACITY: u32 = static_storage::CONTROL_ACTOR_BALANCES_SLOTS as u32;
const PAGE_LOCAL_ACTOR_BALANCE_CAPACITY: u32 =
    static_storage::PAGE_LOCAL_ACTOR_BALANCES_SLOTS as u32;

pub const STATIC_MEMORY_END_PAGE: u32 = static_storage::STATIC_MEMORY_END_PAGE;

type WatActorBalanceTable =
    SailsStaticActorIdU256Map<{ static_storage::WAT_ACTOR_BALANCES_LOG2_SLOTS }>;
type MixedActorBalanceTable =
    SailsStaticMixedActorIdU256Map<{ static_storage::MIXED_ACTOR_BALANCES_LOG2_SLOTS }>;
type TagActorBalanceTable = SailsStaticActorTagU256Map;
type TagU64ActorTable = SailsStaticActorTagU64Map;
type WatAllowanceTable = SailsStaticAllowanceU256Map<{ static_storage::WAT_ALLOWANCES_LOG2_SLOTS }>;
type ControlActorBalanceTable =
    SailsStaticControlActorIdU256Map<{ static_storage::CONTROL_ACTOR_BALANCES_LOG2_SLOTS }>;
type PageLocalActorBalanceTable =
    SailsStaticPageLocalActorIdU256Map<{ static_storage::PAGE_LOCAL_ACTOR_BALANCES_LOG2_TILES }>;
type GroupedActorPages2Table = SailsStaticGroupedControlActorIdU256Map<
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES2_LOG2_GROUPS },
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES2_LOG2_GROUP_PAGES },
>;
type GroupedActorPages4Table = SailsStaticGroupedControlActorIdU256Map<
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES4_LOG2_GROUPS },
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES4_LOG2_GROUP_PAGES },
>;
type GroupedActorPages8Table = SailsStaticGroupedControlActorIdU256Map<
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES8_LOG2_GROUPS },
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES8_LOG2_GROUP_PAGES },
>;
type GroupedActorPages16Table = SailsStaticGroupedControlActorIdU256Map<
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES16_LOG2_GROUPS },
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES16_LOG2_GROUP_PAGES },
>;
type GroupedActorPages32Table = SailsStaticGroupedControlActorIdU256Map<
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES32_LOG2_GROUPS },
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES32_LOG2_GROUP_PAGES },
>;
type GroupedActorPages64Table = SailsStaticGroupedControlActorIdU256Map<
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES64_LOG2_GROUPS },
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES64_LOG2_GROUP_PAGES },
>;
type GroupedActorPages128Table = SailsStaticGroupedControlActorIdU256Map<
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES128_LOG2_GROUPS },
    { static_storage::GROUPED_ACTOR_BALANCES_PAGES128_LOG2_GROUP_PAGES },
>;
type InlineOwnerAccountTable = SailsStaticVftAccountStorage<
    { static_storage::WAT_ACTOR_BALANCES_LOG2_SLOTS },
    { static_storage::WAT_ALLOWANCES_LOG2_SLOTS },
>;

const STORAGE_BACKEND_COUNT: usize = 14;

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MillionStorageBackend {
    GenericStatic,
    WatActorStatic,
    MixedActorStatic,
    TagActorStatic,
    TagU64ActorStatic,
    ControlActorStatic,
    PageLocalActorStatic,
    GroupedActorPages2,
    GroupedActorPages4,
    GroupedActorPages8,
    GroupedActorPages16,
    GroupedActorPages32,
    GroupedActorPages64,
    GroupedActorPages128,
}

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MillionVftBackend {
    BTree,
    HashMap,
    GenericStatic,
    GenericStaticFused,
    GenericStaticFast,
    WatActorStatic,
    MixedActorStatic,
    MixedActorFast,
    TagActorStatic,
    TagU64ActorStatic,
    ControlActorStatic,
    PageLocalActorStatic,
    GroupedActorPages64,
    GroupedActorPages128,
    InlineOwnerAccountU256,
}

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MillionStorageOp {
    InsertFresh,
    UpdateExisting,
    ReadExisting,
    ReadMissing,
    Remove,
}

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MillionVftTransferOp {
    Transfer,
    TransferFrom,
}

#[sails_type]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MillionStorageBenchResult {
    pub value: U256,
    pub len: u32,
    pub existed: bool,
}

#[sails_type]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MillionVftTransferResult {
    pub from_balance: U256,
    pub to_balance: U256,
    pub allowance: U256,
    pub balance_len: u32,
    pub allowance_len: u32,
    pub transferred: bool,
}

struct AppliedOp {
    value: U256,
    existed: bool,
}

struct StorageMillionService<'a> {
    state: &'a RefCell<StorageMillionState>,
}

impl<'a> StorageMillionService<'a> {
    fn new(state: &'a RefCell<StorageMillionState>) -> Self {
        Self { state }
    }
}

#[sails_rs::service]
impl StorageMillionService<'_> {
    #[export]
    pub fn prepare_chunk(
        &mut self,
        backend: MillionStorageBackend,
        start: u32,
        len: u32,
    ) -> MillionStorageBenchResult {
        self.state.borrow_mut().prepare_chunk(backend, start, len)
    }

    #[export]
    pub fn bench(
        &mut self,
        backend: MillionStorageBackend,
        op: MillionStorageOp,
        seed: u32,
    ) -> MillionStorageBenchResult {
        self.state.borrow_mut().bench(backend, op, seed)
    }

    #[export]
    pub fn bench_many(
        &mut self,
        backend: MillionStorageBackend,
        op: MillionStorageOp,
        start_seed: u32,
        count: u32,
    ) -> MillionStorageBenchResult {
        self.state
            .borrow_mut()
            .bench_many(backend, op, start_seed, count)
    }

    #[export]
    pub fn prepare_vft_chunk(
        &mut self,
        backend: MillionVftBackend,
        start: u32,
        len: u32,
    ) -> MillionVftTransferResult {
        self.state
            .borrow_mut()
            .prepare_vft_chunk(backend, start, len)
    }

    #[export]
    pub fn bench_vft_transfer(
        &mut self,
        backend: MillionVftBackend,
        op: MillionVftTransferOp,
        seed: u32,
    ) -> MillionVftTransferResult {
        self.state
            .borrow_mut()
            .bench_vft_transfer(backend, op, seed)
    }

    #[export]
    pub fn bench_vft_transfer_bool(
        &mut self,
        backend: MillionVftBackend,
        op: MillionVftTransferOp,
        seed: u32,
    ) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_transfer_bool(backend, op, seed)
    }

    #[export]
    pub fn bench_vft_transfer_fresh_bool(&mut self, backend: MillionVftBackend, seed: u32) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_transfer_fresh_bool(backend, seed)
    }

    #[export]
    pub fn bench_vft_transfer_from_spender_bool(
        &mut self,
        backend: MillionVftBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_transfer_from_spender_bool(backend, owner_seed, spender_seed)
    }

    #[export]
    pub fn bench_vft_approve_bool(
        &mut self,
        backend: MillionVftBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> bool {
        self.state
            .borrow_mut()
            .bench_vft_approve_bool(backend, owner_seed, spender_seed)
    }
}

pub struct StorageMillionProgram {
    state: RefCell<StorageMillionState>,
}

#[sails_rs::program]
impl StorageMillionProgram {
    pub fn new_for_bench() -> Self {
        Self {
            state: RefCell::new(StorageMillionState::new()),
        }
    }

    pub fn storage_million(&self) -> StorageMillionService<'_> {
        StorageMillionService::new(&self.state)
    }
}

struct StorageMillionState {
    lens: [u32; STORAGE_BACKEND_COUNT],
    allowance_lens: [u32; STORAGE_BACKEND_COUNT],
    dynamic_vft: DynamicMillionVftStorage,
}

enum DynamicMillionVftStorage {
    Empty,
    BTree {
        balances: BTreeMap<ActorId, U256>,
        allowances: BTreeMap<(ActorId, ActorId), U256>,
    },
    HashMap {
        balances: HashMap<ActorId, U256>,
        allowances: HashMap<(ActorId, ActorId), U256>,
    },
}

impl StorageMillionState {
    fn new() -> Self {
        Self {
            lens: [0; STORAGE_BACKEND_COUNT],
            allowance_lens: [0; STORAGE_BACKEND_COUNT],
            dynamic_vft: DynamicMillionVftStorage::Empty,
        }
    }

    fn prepare_chunk(
        &mut self,
        backend: MillionStorageBackend,
        start: u32,
        len: u32,
    ) -> MillionStorageBenchResult {
        let end = start.checked_add(len).expect("million prepare overflow");
        assert!(end <= backend_capacity(backend));

        for seed in start + 1..=end {
            self.insert(backend, seed, value_for_seed(seed));
        }

        MillionStorageBenchResult {
            value: value_for_seed(end),
            len: self.len(backend),
            existed: false,
        }
    }

    fn bench(
        &mut self,
        backend: MillionStorageBackend,
        op: MillionStorageOp,
        seed: u32,
    ) -> MillionStorageBenchResult {
        let applied = self.apply_op(backend, op, seed);
        MillionStorageBenchResult {
            value: applied.value,
            len: self.len(backend),
            existed: applied.existed,
        }
    }

    fn bench_many(
        &mut self,
        backend: MillionStorageBackend,
        op: MillionStorageOp,
        start_seed: u32,
        count: u32,
    ) -> MillionStorageBenchResult {
        assert!(count > 0);

        let mut applied = AppliedOp {
            value: U256::zero(),
            existed: false,
        };
        for offset in 0..count {
            let seed = start_seed
                .checked_add(offset)
                .expect("million batch seed overflow");
            applied = self.apply_op(backend, op, seed);
        }

        MillionStorageBenchResult {
            value: applied.value,
            len: self.len(backend),
            existed: applied.existed,
        }
    }

    fn prepare_vft_chunk(
        &mut self,
        backend: MillionVftBackend,
        start: u32,
        len: u32,
    ) -> MillionVftTransferResult {
        let end = start
            .checked_add(len)
            .expect("million VFT prepare overflow");
        if let Some(static_backend) = backend.static_backend() {
            assert!(end <= backend_capacity(static_backend));
            assert!(end <= allowance_capacity(static_backend));
        }
        if start == 0 {
            self.dynamic_vft = match backend {
                MillionVftBackend::BTree => DynamicMillionVftStorage::BTree {
                    balances: BTreeMap::new(),
                    allowances: BTreeMap::new(),
                },
                MillionVftBackend::HashMap => DynamicMillionVftStorage::HashMap {
                    balances: HashMap::new(),
                    allowances: HashMap::new(),
                },
                _ => DynamicMillionVftStorage::Empty,
            };
        }

        for seed in start + 1..=end {
            self.vft_insert_balance(backend, seed, vft_balance_for_seed(seed));
            self.insert_allowance(
                backend,
                vft_actor_for_seed(seed),
                vft_spender_for_seed(seed),
                vft_allowance_for_seed(seed),
            );
        }

        MillionVftTransferResult {
            from_balance: vft_balance_for_seed(end),
            to_balance: U256::zero(),
            allowance: U256::zero(),
            balance_len: self.vft_balance_len(backend),
            allowance_len: self.vft_allowance_len(backend),
            transferred: false,
        }
    }

    fn bench_vft_transfer(
        &mut self,
        backend: MillionVftBackend,
        op: MillionVftTransferOp,
        seed: u32,
    ) -> MillionVftTransferResult {
        let len = self.vft_balance_len(backend);
        assert!(len > 1);
        let to_seed = if seed == len { 1 } else { seed + 1 };
        let amount = vft_transfer_amount(seed);
        let spender = vft_spender_for_seed(seed);

        if backend == MillionVftBackend::WatActorStatic {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            let allowance_key = matches!(op, MillionVftTransferOp::TransferFrom).then_some(spender);

            if allowance_key.is_some() {
                let Some(transfer) = wat_actor_balance_table()
                    .transfer_actor_u256_from(
                        &wat_allowance_table(),
                        owner,
                        spender,
                        recipient,
                        amount,
                    )
                    .expect("wat actor million transfer-from failed")
                else {
                    return self.vft_result(backend, seed, to_seed, allowance_key, false);
                };

                return MillionVftTransferResult {
                    from_balance: transfer.from_balance,
                    to_balance: transfer.to_balance,
                    allowance: transfer.allowance,
                    balance_len: self.vft_balance_len(backend),
                    allowance_len: self.vft_allowance_len(backend),
                    transferred: true,
                };
            }

            let Some(transfer) = wat_actor_balance_table()
                .transfer_actor_u256(owner, recipient, amount)
                .expect("wat actor million balance transfer failed")
            else {
                return self.vft_result(backend, seed, to_seed, allowance_key, false);
            };

            return MillionVftTransferResult {
                from_balance: transfer.from_balance,
                to_balance: transfer.to_balance,
                allowance: U256::zero(),
                balance_len: self.vft_balance_len(backend),
                allowance_len: self.vft_allowance_len(backend),
                transferred: true,
            };
        }
        if backend == MillionVftBackend::MixedActorStatic {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            let allowance_key = matches!(op, MillionVftTransferOp::TransferFrom).then_some(spender);

            if allowance_key.is_some() {
                let Some(transfer) = mixed_actor_balance_table()
                    .transfer_actor_u256_from(
                        &wat_allowance_table(),
                        owner,
                        spender,
                        recipient,
                        amount,
                    )
                    .expect("mixed actor million transfer-from failed")
                else {
                    return self.vft_result(backend, seed, to_seed, allowance_key, false);
                };

                return MillionVftTransferResult {
                    from_balance: transfer.from_balance,
                    to_balance: transfer.to_balance,
                    allowance: transfer.allowance,
                    balance_len: self.vft_balance_len(backend),
                    allowance_len: self.vft_allowance_len(backend),
                    transferred: true,
                };
            }

            let Some(transfer) = mixed_actor_balance_table()
                .transfer_actor_u256(owner, recipient, amount)
                .expect("mixed actor million balance transfer failed")
            else {
                return self.vft_result(backend, seed, to_seed, allowance_key, false);
            };

            return MillionVftTransferResult {
                from_balance: transfer.from_balance,
                to_balance: transfer.to_balance,
                allowance: U256::zero(),
                balance_len: self.vft_balance_len(backend),
                allowance_len: self.vft_allowance_len(backend),
                transferred: true,
            };
        }
        if backend == MillionVftBackend::MixedActorFast {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            let allowance_key = matches!(op, MillionVftTransferOp::TransferFrom).then_some(spender);

            if allowance_key.is_some() {
                let Some(transfer) = (unsafe {
                    mixed_actor_balance_table()
                        .transfer_actor_u256_from_nonzero_distinct_unchecked(
                            &wat_allowance_table(),
                            owner,
                            spender,
                            recipient,
                            amount,
                        )
                        .expect("mixed actor fast million transfer-from failed")
                }) else {
                    return self.vft_result(backend, seed, to_seed, allowance_key, false);
                };

                return MillionVftTransferResult {
                    from_balance: transfer.from_balance,
                    to_balance: transfer.to_balance,
                    allowance: transfer.allowance,
                    balance_len: self.vft_balance_len(backend),
                    allowance_len: self.vft_allowance_len(backend),
                    transferred: true,
                };
            }

            let Some(transfer) = (unsafe {
                mixed_actor_balance_table()
                    .transfer_actor_u256_nonzero_distinct_unchecked(owner, recipient, amount)
                    .expect("mixed actor fast million balance transfer failed")
            }) else {
                return self.vft_result(backend, seed, to_seed, allowance_key, false);
            };

            return MillionVftTransferResult {
                from_balance: transfer.from_balance,
                to_balance: transfer.to_balance,
                allowance: U256::zero(),
                balance_len: self.vft_balance_len(backend),
                allowance_len: self.vft_allowance_len(backend),
                transferred: true,
            };
        }
        if matches!(op, MillionVftTransferOp::TransferFrom) {
            let allowance = self
                .get_allowance(backend, vft_actor_for_seed(seed), spender)
                .unwrap_or_else(U256::zero);
            if allowance < amount {
                return self.vft_result(backend, seed, to_seed, Some(spender), false);
            }
            self.insert_allowance(
                backend,
                vft_actor_for_seed(seed),
                spender,
                allowance - amount,
            );
        }

        let from_balance = self
            .vft_get_balance(backend, seed)
            .unwrap_or_else(U256::zero);
        if from_balance < amount {
            let allowance_key = matches!(op, MillionVftTransferOp::TransferFrom).then_some(spender);
            return self.vft_result(backend, seed, to_seed, allowance_key, false);
        }

        let to_balance = self
            .vft_get_balance(backend, to_seed)
            .unwrap_or_else(U256::zero);
        self.vft_insert_balance(backend, seed, from_balance - amount);
        self.vft_insert_balance(backend, to_seed, to_balance + amount);

        let allowance_key = matches!(op, MillionVftTransferOp::TransferFrom).then_some(spender);
        self.vft_result(backend, seed, to_seed, allowance_key, true)
    }

    fn bench_vft_transfer_bool(
        &mut self,
        backend: MillionVftBackend,
        op: MillionVftTransferOp,
        seed: u32,
    ) -> bool {
        let len = self.vft_balance_len(backend);
        assert!(len > 1);
        let to_seed = if seed == len { 1 } else { seed + 1 };
        self.apply_vft_transfer_bool(backend, op, seed, to_seed)
    }

    fn bench_vft_transfer_fresh_bool(&mut self, backend: MillionVftBackend, seed: u32) -> bool {
        let len = self.vft_balance_len(backend);
        assert!(len > 1);
        let to_seed = len
            .checked_add(seed)
            .and_then(|seed| seed.checked_add(1))
            .expect("fresh VFT recipient seed overflow");
        self.apply_vft_transfer_bool(backend, MillionVftTransferOp::Transfer, seed, to_seed)
    }

    fn bench_vft_transfer_from_spender_bool(
        &mut self,
        backend: MillionVftBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> bool {
        let len = self.vft_balance_len(backend);
        assert!(len > 1);
        let to_seed = if owner_seed == len {
            1
        } else {
            owner_seed + 1
        };
        self.apply_vft_transfer_from_spender_bool(backend, owner_seed, spender_seed, to_seed)
    }

    fn bench_vft_approve_bool(
        &mut self,
        backend: MillionVftBackend,
        owner_seed: u32,
        spender_seed: u32,
    ) -> bool {
        let owner = vft_actor_for_seed(owner_seed);
        let spender = vft_spender_for_seed(spender_seed);
        if owner == spender {
            return false;
        }

        if backend == MillionVftBackend::TagActorStatic {
            return unsafe {
                tag_allowance_table()
                    .insert_allowance_u256_low64_bool_unchecked(
                        owner,
                        spender,
                        vft_allowance_for_seed(spender_seed),
                    )
                    .expect("tag million allowance bool insert failed")
            };
        }
        if backend == MillionVftBackend::TagU64ActorStatic {
            return unsafe {
                tag_u64_allowance_table().insert_tag_value_bool_capacity_unchecked(
                    vft_allowance_tag_for_seeds(owner_seed, spender_seed),
                    vft_allowance_for_seed_u64(spender_seed),
                )
            };
        }

        self.insert_allowance(
            backend,
            owner,
            spender,
            vft_allowance_for_seed(spender_seed),
        );
        true
    }

    fn apply_vft_transfer_bool(
        &mut self,
        backend: MillionVftBackend,
        op: MillionVftTransferOp,
        seed: u32,
        to_seed: u32,
    ) -> bool {
        let amount = vft_transfer_amount(seed);
        let spender = vft_spender_for_seed(seed);

        if matches!(
            backend,
            MillionVftBackend::GenericStaticFused | MillionVftBackend::GenericStaticFast
        ) {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            let inserted_recipient = to_seed > self.vft_balance_len(backend);
            let transferred = match op {
                MillionVftTransferOp::Transfer
                    if backend == MillionVftBackend::GenericStaticFast =>
                {
                    unsafe {
                        generic_balance_table()
                            .transfer_actor_u256_nonzero_distinct_unchecked(
                                owner, recipient, amount,
                            )
                            .expect("generic million balance transfer failed")
                            .is_some()
                    }
                }
                MillionVftTransferOp::Transfer => generic_balance_table()
                    .transfer_actor_u256(owner, recipient, amount)
                    .expect("generic million balance transfer failed")
                    .is_some(),
                MillionVftTransferOp::TransferFrom
                    if backend == MillionVftBackend::GenericStaticFast =>
                {
                    unsafe {
                        generic_balance_table()
                            .transfer_actor_u256_from_nonzero_distinct_unchecked(
                                &generic_allowance_table(),
                                owner,
                                spender,
                                recipient,
                                amount,
                            )
                            .expect("generic million transfer-from failed")
                            .is_some()
                    }
                }
                MillionVftTransferOp::TransferFrom => generic_balance_table()
                    .transfer_actor_u256_from(
                        &generic_allowance_table(),
                        owner,
                        spender,
                        recipient,
                        amount,
                    )
                    .expect("generic million transfer-from failed")
                    .is_some()
            };
            if transferred && inserted_recipient {
                *self.len_mut(MillionStorageBackend::GenericStatic) += 1;
            }
            return transferred;
        }

        if backend == MillionVftBackend::WatActorStatic {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            return match op {
                MillionVftTransferOp::Transfer => wat_actor_balance_table()
                    .transfer_actor_u256(owner, recipient, amount)
                    .expect("wat actor million balance transfer failed")
                    .is_some(),
                MillionVftTransferOp::TransferFrom => wat_actor_balance_table()
                    .transfer_actor_u256_from(
                        &wat_allowance_table(),
                        owner,
                        spender,
                        recipient,
                        amount,
                    )
                    .expect("wat actor million transfer-from failed")
                    .is_some(),
            };
        }
        if backend == MillionVftBackend::MixedActorStatic {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            return match op {
                MillionVftTransferOp::Transfer => mixed_actor_balance_table()
                    .transfer_actor_u256(owner, recipient, amount)
                    .expect("mixed actor million balance transfer failed")
                    .is_some(),
                MillionVftTransferOp::TransferFrom => mixed_actor_balance_table()
                    .transfer_actor_u256_from(
                        &wat_allowance_table(),
                        owner,
                        spender,
                        recipient,
                        amount,
                    )
                    .expect("mixed actor million transfer-from failed")
                    .is_some(),
            };
        }

        if backend == MillionVftBackend::MixedActorFast {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            return match op {
                MillionVftTransferOp::Transfer => unsafe {
                    mixed_actor_balance_table()
                        .transfer_actor_u256_bool_nonzero_distinct_unchecked(
                            owner, recipient, amount,
                        )
                        .expect("mixed actor fast million balance transfer failed")
                },
                MillionVftTransferOp::TransferFrom => unsafe {
                    mixed_actor_balance_table()
                        .transfer_actor_u256_from_bool_nonzero_distinct_unchecked(
                            &wat_allowance_table(),
                            owner,
                            spender,
                            recipient,
                            amount,
                        )
                        .expect("mixed actor fast million transfer-from failed")
                },
            };
        }
        if backend == MillionVftBackend::TagActorStatic {
            let owner = vft_actor_for_seed(seed);
            let recipient = vft_actor_for_seed(to_seed);
            return match op {
                MillionVftTransferOp::Transfer => unsafe {
                    tag_actor_balance_table()
                        .transfer_actor_u256_low64_bool_unchecked(owner, recipient, amount)
                        .expect("tag actor million balance transfer failed")
                },
                MillionVftTransferOp::TransferFrom => unsafe {
                    tag_actor_balance_table()
                        .transfer_actor_u256_from_tag_low64_bool_unchecked(
                            &tag_allowance_table(),
                            owner,
                            spender,
                            recipient,
                            amount,
                        )
                        .expect("tag actor million transfer-from failed")
                },
            };
        }
        if backend == MillionVftBackend::TagU64ActorStatic {
            let owner = vft_actor_tag_for_seed(seed);
            let recipient = vft_actor_tag_for_seed(to_seed);
            let amount = vft_transfer_amount_u64(seed);
            return match op {
                MillionVftTransferOp::Transfer => unsafe {
                    tag_u64_balance_table()
                        .transfer_tags_bool_capacity_unchecked(owner, recipient, amount)
                },
                MillionVftTransferOp::TransferFrom => unsafe {
                    tag_u64_balance_table().transfer_from_tags_bool_capacity_unchecked(
                        &tag_u64_allowance_table(),
                        vft_allowance_tag_for_seeds(seed, seed),
                        owner,
                        recipient,
                        amount,
                    )
                },
            };
        }
        if backend == MillionVftBackend::InlineOwnerAccountU256 {
            return match op {
                MillionVftTransferOp::Transfer => {
                    inline_owner_account_table().transfer(
                        vft_actor_for_seed(seed),
                        vft_actor_for_seed(to_seed),
                        amount,
                    )
                    .expect("inline owner account transfer failed")
                },
                MillionVftTransferOp::TransferFrom => self
                    .apply_vft_transfer_from_spender_bool(backend, seed, seed, to_seed),
            };
        }

        if matches!(op, MillionVftTransferOp::TransferFrom) {
            let allowance = self
                .get_allowance(backend, vft_actor_for_seed(seed), spender)
                .unwrap_or_else(U256::zero);
            if allowance < amount {
                return false;
            }
            self.insert_allowance(
                backend,
                vft_actor_for_seed(seed),
                spender,
                allowance - amount,
            );
        }

        let from_balance = self
            .vft_get_balance(backend, seed)
            .unwrap_or_else(U256::zero);
        if from_balance < amount {
            return false;
        }

        let to_balance = self
            .vft_get_balance(backend, to_seed)
            .unwrap_or_else(U256::zero);
        let (to_balance, overflow) = to_balance.overflowing_add(amount);
        if overflow {
            return false;
        }

        self.vft_insert_balance(backend, seed, from_balance - amount);
        self.vft_insert_balance(backend, to_seed, to_balance);
        true
    }

    fn apply_vft_transfer_from_spender_bool(
        &mut self,
        backend: MillionVftBackend,
        owner_seed: u32,
        spender_seed: u32,
        to_seed: u32,
    ) -> bool {
        let amount = vft_transfer_amount(owner_seed);
        if backend == MillionVftBackend::TagActorStatic {
            return unsafe {
                tag_actor_balance_table()
                    .transfer_actor_u256_from_tag_low64_bool_unchecked(
                        &tag_allowance_table(),
                        vft_actor_for_seed(owner_seed),
                        vft_spender_for_seed(spender_seed),
                        vft_actor_for_seed(to_seed),
                        amount,
                    )
                    .expect("tag actor spender transfer-from failed")
            };
        }
        if backend == MillionVftBackend::TagU64ActorStatic {
            return unsafe {
                tag_u64_balance_table().transfer_from_tags_bool_capacity_unchecked(
                    &tag_u64_allowance_table(),
                    vft_allowance_tag_for_seeds(owner_seed, spender_seed),
                    vft_actor_tag_for_seed(owner_seed),
                    vft_actor_tag_for_seed(to_seed),
                    vft_transfer_amount_u64(owner_seed),
                )
            };
        }
        if backend == MillionVftBackend::InlineOwnerAccountU256 {
            return inline_owner_account_table()
                .transfer_from(
                    vft_spender_for_seed(spender_seed),
                    vft_actor_for_seed(owner_seed),
                    vft_actor_for_seed(to_seed),
                    amount,
                )
                .expect("inline owner account transfer-from failed");
        }

        let owner = vft_actor_for_seed(owner_seed);
        let spender = vft_spender_for_seed(spender_seed);
        let allowance = self
            .get_allowance(backend, owner, spender)
            .unwrap_or_else(U256::zero);
        if allowance < amount {
            return false;
        }

        let from_balance = self
            .vft_get_balance(backend, owner_seed)
            .unwrap_or_else(U256::zero);
        if from_balance < amount {
            return false;
        }

        let to_balance = self
            .vft_get_balance(backend, to_seed)
            .unwrap_or_else(U256::zero);
        let (to_balance, overflow) = to_balance.overflowing_add(amount);
        if overflow {
            return false;
        }

        self.insert_allowance(backend, owner, spender, allowance - amount);
        self.vft_insert_balance(backend, owner_seed, from_balance - amount);
        self.vft_insert_balance(backend, to_seed, to_balance);
        true
    }

    fn apply_op(
        &mut self,
        backend: MillionStorageBackend,
        op: MillionStorageOp,
        seed: u32,
    ) -> AppliedOp {
        match op {
            MillionStorageOp::InsertFresh => {
                let value = value_for_seed(seed);
                let existed = self.insert(backend, seed, value).is_some();
                AppliedOp { value, existed }
            }
            MillionStorageOp::UpdateExisting => {
                let value = updated_value_for_seed(seed);
                let existed = self.insert(backend, seed, value).is_some();
                AppliedOp { value, existed }
            }
            MillionStorageOp::ReadExisting | MillionStorageOp::ReadMissing => {
                let value = self.get(backend, seed);
                AppliedOp {
                    existed: value.is_some(),
                    value: value.unwrap_or_else(U256::zero),
                }
            }
            MillionStorageOp::Remove => {
                let value = self.remove(backend, seed);
                AppliedOp {
                    existed: value.is_some(),
                    value: value.unwrap_or_else(U256::zero),
                }
            }
        }
    }

    fn len(&self, backend: MillionStorageBackend) -> u32 {
        self.lens[backend_index(backend)]
    }

    fn len_mut(&mut self, backend: MillionStorageBackend) -> &mut u32 {
        &mut self.lens[backend_index(backend)]
    }

    fn allowance_len(&self, backend: MillionStorageBackend) -> u32 {
        self.allowance_lens[backend_index(backend)]
    }

    fn allowance_len_mut(&mut self, backend: MillionStorageBackend) -> &mut u32 {
        &mut self.allowance_lens[backend_index(backend)]
    }

    fn insert_actor_value(
        &mut self,
        backend: MillionStorageBackend,
        actor: ActorId,
        value: U256,
    ) -> Option<U256> {
        let previous = match backend {
            MillionStorageBackend::GenericStatic => generic_balance_table()
                .insert_actor_u256(actor, value)
                .expect("generic million balance table insert failed"),
            MillionStorageBackend::WatActorStatic => wat_actor_balance_table()
                .insert_actor_u256(actor, value)
                .expect("wat actor million balance table insert failed"),
            MillionStorageBackend::MixedActorStatic => mixed_actor_balance_table()
                .insert_actor_u256(actor, value)
                .expect("mixed actor million balance table insert failed"),
            MillionStorageBackend::TagActorStatic => tag_actor_balance_table()
                .insert_actor_u256(actor, value)
                .expect("tag actor million balance table insert failed"),
            MillionStorageBackend::TagU64ActorStatic => tag_u64_balance_table()
                .insert_actor_u256(actor, value)
                .expect("tag u64 actor million balance table insert failed"),
            MillionStorageBackend::ControlActorStatic => control_actor_balance_table()
                .insert_actor_u256(actor, value)
                .expect("control actor million balance table insert failed"),
            MillionStorageBackend::PageLocalActorStatic => page_local_actor_balance_table()
                .insert_actor_u256(actor, value)
                .expect("page-local actor million balance table insert failed"),
            MillionStorageBackend::GroupedActorPages2 => grouped_actor_pages2_table()
                .insert_actor_u256(actor, value)
                .expect("grouped actor pages2 million balance table insert failed"),
            MillionStorageBackend::GroupedActorPages4 => grouped_actor_pages4_table()
                .insert_actor_u256(actor, value)
                .expect("grouped actor pages4 million balance table insert failed"),
            MillionStorageBackend::GroupedActorPages8 => grouped_actor_pages8_table()
                .insert_actor_u256(actor, value)
                .expect("grouped actor pages8 million balance table insert failed"),
            MillionStorageBackend::GroupedActorPages16 => grouped_actor_pages16_table()
                .insert_actor_u256(actor, value)
                .expect("grouped actor pages16 million balance table insert failed"),
            MillionStorageBackend::GroupedActorPages32 => grouped_actor_pages32_table()
                .insert_actor_u256(actor, value)
                .expect("grouped actor pages32 million balance table insert failed"),
            MillionStorageBackend::GroupedActorPages64 => grouped_actor_pages64_table()
                .insert_actor_u256(actor, value)
                .expect("grouped actor pages64 million balance table insert failed"),
            MillionStorageBackend::GroupedActorPages128 => grouped_actor_pages128_table()
                .insert_actor_u256(actor, value)
                .expect("grouped actor pages128 million balance table insert failed"),
        };
        if previous.is_none() && !value.is_zero() {
            *self.len_mut(backend) += 1;
        }
        previous
    }

    fn get_actor_value(&self, backend: MillionStorageBackend, actor: ActorId) -> Option<U256> {
        match backend {
            MillionStorageBackend::GenericStatic => generic_balance_table()
                .get_actor_u256(&actor)
                .expect("generic million balance table get failed"),
            MillionStorageBackend::WatActorStatic => wat_actor_balance_table()
                .get_actor_u256(&actor)
                .expect("wat actor million balance table get failed"),
            MillionStorageBackend::MixedActorStatic => mixed_actor_balance_table()
                .get_actor_u256(&actor)
                .expect("mixed actor million balance table get failed"),
            MillionStorageBackend::TagActorStatic => tag_actor_balance_table()
                .get_actor_u256(&actor)
                .expect("tag actor million balance table get failed"),
            MillionStorageBackend::TagU64ActorStatic => tag_u64_balance_table()
                .get_actor_u256(&actor)
                .expect("tag u64 actor million balance table get failed"),
            MillionStorageBackend::ControlActorStatic => control_actor_balance_table()
                .get_actor_u256(&actor)
                .expect("control actor million balance table get failed"),
            MillionStorageBackend::PageLocalActorStatic => page_local_actor_balance_table()
                .get_actor_u256(&actor)
                .expect("page-local actor million balance table get failed"),
            MillionStorageBackend::GroupedActorPages2 => grouped_actor_pages2_table()
                .get_actor_u256(&actor)
                .expect("grouped actor pages2 million balance table get failed"),
            MillionStorageBackend::GroupedActorPages4 => grouped_actor_pages4_table()
                .get_actor_u256(&actor)
                .expect("grouped actor pages4 million balance table get failed"),
            MillionStorageBackend::GroupedActorPages8 => grouped_actor_pages8_table()
                .get_actor_u256(&actor)
                .expect("grouped actor pages8 million balance table get failed"),
            MillionStorageBackend::GroupedActorPages16 => grouped_actor_pages16_table()
                .get_actor_u256(&actor)
                .expect("grouped actor pages16 million balance table get failed"),
            MillionStorageBackend::GroupedActorPages32 => grouped_actor_pages32_table()
                .get_actor_u256(&actor)
                .expect("grouped actor pages32 million balance table get failed"),
            MillionStorageBackend::GroupedActorPages64 => grouped_actor_pages64_table()
                .get_actor_u256(&actor)
                .expect("grouped actor pages64 million balance table get failed"),
            MillionStorageBackend::GroupedActorPages128 => grouped_actor_pages128_table()
                .get_actor_u256(&actor)
                .expect("grouped actor pages128 million balance table get failed"),
        }
    }

    fn insert(&mut self, backend: MillionStorageBackend, seed: u32, value: U256) -> Option<U256> {
        let actor = actor_for_seed(seed);
        self.insert_actor_value(backend, actor, value)
    }

    fn get(&self, backend: MillionStorageBackend, seed: u32) -> Option<U256> {
        let actor = actor_for_seed(seed);
        self.get_actor_value(backend, actor)
    }

    fn remove(&mut self, backend: MillionStorageBackend, seed: u32) -> Option<U256> {
        let actor = actor_for_seed(seed);
        let previous = match backend {
            MillionStorageBackend::GenericStatic => generic_balance_table()
                .remove_actor_u256(&actor)
                .expect("generic million balance table remove failed"),
            MillionStorageBackend::WatActorStatic => wat_actor_balance_table()
                .remove_actor_u256(&actor)
                .expect("wat actor million balance table remove failed"),
            MillionStorageBackend::MixedActorStatic => mixed_actor_balance_table()
                .remove_actor_u256(&actor)
                .expect("mixed actor million balance table remove failed"),
            MillionStorageBackend::TagActorStatic => tag_actor_balance_table()
                .remove_actor_u256(&actor)
                .expect("tag actor million balance table remove failed"),
            MillionStorageBackend::TagU64ActorStatic => tag_u64_balance_table()
                .remove_actor_u256(&actor)
                .expect("tag u64 actor million balance table remove failed"),
            MillionStorageBackend::ControlActorStatic => control_actor_balance_table()
                .remove_actor_u256(&actor)
                .expect("control actor million balance table remove failed"),
            MillionStorageBackend::PageLocalActorStatic => page_local_actor_balance_table()
                .remove_actor_u256(&actor)
                .expect("page-local actor million balance table remove failed"),
            MillionStorageBackend::GroupedActorPages2 => grouped_actor_pages2_table()
                .remove_actor_u256(&actor)
                .expect("grouped actor pages2 million balance table remove failed"),
            MillionStorageBackend::GroupedActorPages4 => grouped_actor_pages4_table()
                .remove_actor_u256(&actor)
                .expect("grouped actor pages4 million balance table remove failed"),
            MillionStorageBackend::GroupedActorPages8 => grouped_actor_pages8_table()
                .remove_actor_u256(&actor)
                .expect("grouped actor pages8 million balance table remove failed"),
            MillionStorageBackend::GroupedActorPages16 => grouped_actor_pages16_table()
                .remove_actor_u256(&actor)
                .expect("grouped actor pages16 million balance table remove failed"),
            MillionStorageBackend::GroupedActorPages32 => grouped_actor_pages32_table()
                .remove_actor_u256(&actor)
                .expect("grouped actor pages32 million balance table remove failed"),
            MillionStorageBackend::GroupedActorPages64 => grouped_actor_pages64_table()
                .remove_actor_u256(&actor)
                .expect("grouped actor pages64 million balance table remove failed"),
            MillionStorageBackend::GroupedActorPages128 => grouped_actor_pages128_table()
                .remove_actor_u256(&actor)
                .expect("grouped actor pages128 million balance table remove failed"),
        };
        if previous.is_some() {
            *self.len_mut(backend) -= 1;
        }
        previous
    }

    fn vft_balance_len(&self, backend: MillionVftBackend) -> u32 {
        match (backend, &self.dynamic_vft) {
            (MillionVftBackend::BTree, DynamicMillionVftStorage::BTree { balances, .. }) => {
                balances.len() as u32
            }
            (MillionVftBackend::HashMap, DynamicMillionVftStorage::HashMap { balances, .. }) => {
                balances.len() as u32
            }
            _ => backend
                .static_backend()
                .map_or(0, |backend| self.len(backend)),
        }
    }

    fn vft_allowance_len(&self, backend: MillionVftBackend) -> u32 {
        match (backend, &self.dynamic_vft) {
            (MillionVftBackend::BTree, DynamicMillionVftStorage::BTree { allowances, .. }) => {
                allowances.len() as u32
            }
            (MillionVftBackend::HashMap, DynamicMillionVftStorage::HashMap { allowances, .. }) => {
                allowances.len() as u32
            }
            _ => backend
                .static_backend()
                .map_or(0, |backend| self.allowance_len(backend)),
        }
    }

    fn vft_get_balance(&self, backend: MillionVftBackend, seed: u32) -> Option<U256> {
        let actor = vft_actor_for_seed(seed);
        match (backend, &self.dynamic_vft) {
            (MillionVftBackend::BTree, DynamicMillionVftStorage::BTree { balances, .. }) => {
                balances.get(&actor).copied()
            }
            (MillionVftBackend::HashMap, DynamicMillionVftStorage::HashMap { balances, .. }) => {
                balances.get(&actor).copied()
            }
            (MillionVftBackend::InlineOwnerAccountU256, _) => {
                inline_owner_account_table()
                    .accounts()
                    .get_balance(vft_actor_for_seed(seed))
                    .expect("inline owner account balance get failed")
            }
            _ => backend
                .static_backend()
                .and_then(|backend| self.get_actor_value(backend, actor)),
        }
    }

    fn vft_insert_balance(
        &mut self,
        backend: MillionVftBackend,
        seed: u32,
        value: U256,
    ) -> Option<U256> {
        let actor = vft_actor_for_seed(seed);
        match (backend, &mut self.dynamic_vft) {
            (MillionVftBackend::BTree, DynamicMillionVftStorage::BTree { balances, .. }) => {
                balances.insert(actor, value)
            }
            (MillionVftBackend::HashMap, DynamicMillionVftStorage::HashMap { balances, .. }) => {
                balances.insert(actor, value)
            }
            (MillionVftBackend::InlineOwnerAccountU256, _) => {
                let previous = inline_owner_account_table()
                    .accounts()
                    .insert_balance(vft_actor_for_seed(seed), value)
                    .expect("inline owner account balance insert failed");
                if previous.is_none() && !value.is_zero() {
                    *self.len_mut(MillionStorageBackend::GroupedActorPages128) += 1;
                }
                previous
            }
            _ => {
                let static_backend = backend
                    .static_backend()
                    .expect("static VFT backend expected");
                self.insert_actor_value(static_backend, actor, value)
            }
        }
    }

    fn insert_allowance(
        &mut self,
        backend: MillionVftBackend,
        owner: ActorId,
        spender: ActorId,
        value: U256,
    ) -> Option<U256> {
        match (backend, &mut self.dynamic_vft) {
            (MillionVftBackend::BTree, DynamicMillionVftStorage::BTree { allowances, .. }) => {
                return allowances.insert((owner, spender), value);
            }
            (MillionVftBackend::HashMap, DynamicMillionVftStorage::HashMap { allowances, .. }) => {
                return allowances.insert((owner, spender), value);
            }
            _ => {}
        }

        if backend == MillionVftBackend::InlineOwnerAccountU256 {
            let storage = inline_owner_account_table();
            let previous = storage
                .allowance(owner, spender)
                .expect("inline owner account allowance get failed");
            storage
                .approve(owner, spender, value)
                .expect("inline owner account allowance insert failed");
            if previous.is_zero() && !value.is_zero() {
                *self.allowance_len_mut(MillionStorageBackend::GroupedActorPages128) += 1;
            }
            return (!previous.is_zero()).then_some(previous);
        }

        let static_backend = backend
            .static_backend()
            .expect("static VFT backend expected");
        let previous = match static_backend {
            MillionStorageBackend::GenericStatic => generic_allowance_table()
                .insert_allowance_u256(owner, spender, value)
                .expect("generic million allowance table insert failed"),
            MillionStorageBackend::TagActorStatic => tag_allowance_table()
                .insert_allowance_u256(owner, spender, value)
                .expect("tag million allowance table insert failed"),
            MillionStorageBackend::TagU64ActorStatic => {
                tag_u64_allowance_table()
                    .insert_allowance_u256_bool(owner, spender, value)
                    .expect("tag u64 million allowance table insert failed");
                None
            }
            _ => wat_allowance_table()
                .insert_allowance_u256(owner, spender, value)
                .expect("wat million allowance table insert failed"),
        };
        if previous.is_none() && !value.is_zero() {
            *self.allowance_len_mut(static_backend) += 1;
        }
        previous
    }

    fn get_allowance(
        &self,
        backend: MillionVftBackend,
        owner: ActorId,
        spender: ActorId,
    ) -> Option<U256> {
        match (backend, &self.dynamic_vft) {
            (MillionVftBackend::BTree, DynamicMillionVftStorage::BTree { allowances, .. }) => {
                return allowances.get(&(owner, spender)).copied();
            }
            (MillionVftBackend::HashMap, DynamicMillionVftStorage::HashMap { allowances, .. }) => {
                return allowances.get(&(owner, spender)).copied();
            }
            _ => {}
        }

        if backend == MillionVftBackend::InlineOwnerAccountU256 {
            let allowance = inline_owner_account_table()
                .allowance(owner, spender)
                .expect("inline owner account allowance get failed");
            return (!allowance.is_zero()).then_some(allowance);
        }

        match backend
            .static_backend()
            .expect("static VFT backend expected")
        {
            MillionStorageBackend::GenericStatic => generic_allowance_table()
                .get_allowance_u256(&owner, &spender)
                .expect("generic million allowance table get failed"),
            MillionStorageBackend::TagActorStatic => tag_allowance_table()
                .get_allowance_u256(&owner, &spender)
                .expect("tag million allowance table get failed"),
            MillionStorageBackend::TagU64ActorStatic => tag_u64_allowance_table()
                .get_allowance_u256(&owner, &spender)
                .expect("tag u64 million allowance table get failed"),
            _ => wat_allowance_table()
                .get_allowance_u256(&owner, &spender)
                .expect("wat million allowance table get failed"),
        }
    }

    fn vft_result(
        &self,
        backend: MillionVftBackend,
        from_seed: u32,
        to_seed: u32,
        spender: Option<ActorId>,
        transferred: bool,
    ) -> MillionVftTransferResult {
        let owner = vft_actor_for_seed(from_seed);
        MillionVftTransferResult {
            from_balance: self
                .vft_get_balance(backend, from_seed)
                .unwrap_or_else(U256::zero),
            to_balance: self
                .vft_get_balance(backend, to_seed)
                .unwrap_or_else(U256::zero),
            allowance: spender
                .and_then(|spender| self.get_allowance(backend, owner, spender))
                .unwrap_or_else(U256::zero),
            balance_len: self.vft_balance_len(backend),
            allowance_len: self.vft_allowance_len(backend),
            transferred,
        }
    }
}

impl MillionVftBackend {
    fn static_backend(self) -> Option<MillionStorageBackend> {
        match self {
            Self::BTree | Self::HashMap => None,
            Self::GenericStatic | Self::GenericStaticFused | Self::GenericStaticFast => {
                Some(MillionStorageBackend::GenericStatic)
            }
            Self::WatActorStatic => Some(MillionStorageBackend::WatActorStatic),
            Self::MixedActorStatic | Self::MixedActorFast => {
                Some(MillionStorageBackend::MixedActorStatic)
            }
            Self::TagActorStatic => Some(MillionStorageBackend::TagActorStatic),
            Self::TagU64ActorStatic => Some(MillionStorageBackend::TagU64ActorStatic),
            Self::ControlActorStatic => Some(MillionStorageBackend::ControlActorStatic),
            Self::PageLocalActorStatic => Some(MillionStorageBackend::PageLocalActorStatic),
            Self::GroupedActorPages64 => Some(MillionStorageBackend::GroupedActorPages64),
            Self::GroupedActorPages128 | Self::InlineOwnerAccountU256 => {
                Some(MillionStorageBackend::GroupedActorPages128)
            }
        }
    }
}

fn backend_index(backend: MillionStorageBackend) -> usize {
    match backend {
        MillionStorageBackend::GenericStatic => 0,
        MillionStorageBackend::WatActorStatic => 1,
        MillionStorageBackend::MixedActorStatic => 2,
        MillionStorageBackend::TagActorStatic => 3,
        MillionStorageBackend::TagU64ActorStatic => 4,
        MillionStorageBackend::ControlActorStatic => 5,
        MillionStorageBackend::PageLocalActorStatic => 6,
        MillionStorageBackend::GroupedActorPages2 => 7,
        MillionStorageBackend::GroupedActorPages4 => 8,
        MillionStorageBackend::GroupedActorPages8 => 9,
        MillionStorageBackend::GroupedActorPages16 => 10,
        MillionStorageBackend::GroupedActorPages32 => 11,
        MillionStorageBackend::GroupedActorPages64 => 12,
        MillionStorageBackend::GroupedActorPages128 => 13,
    }
}

fn backend_capacity(backend: MillionStorageBackend) -> u32 {
    match backend {
        MillionStorageBackend::GenericStatic => GENERIC_BALANCE_CAPACITY,
        MillionStorageBackend::WatActorStatic => WAT_ACTOR_BALANCE_CAPACITY,
        MillionStorageBackend::MixedActorStatic => MIXED_ACTOR_BALANCE_CAPACITY,
        MillionStorageBackend::TagActorStatic => TAG_ACTOR_BALANCE_CAPACITY,
        MillionStorageBackend::TagU64ActorStatic => TAG_U64_ACTOR_BALANCE_CAPACITY,
        MillionStorageBackend::ControlActorStatic => CONTROL_ACTOR_BALANCE_CAPACITY,
        MillionStorageBackend::PageLocalActorStatic => PAGE_LOCAL_ACTOR_BALANCE_CAPACITY,
        MillionStorageBackend::GroupedActorPages2 => {
            static_storage::GROUPED_ACTOR_BALANCES_PAGES2_SLOTS as u32
        }
        MillionStorageBackend::GroupedActorPages4 => {
            static_storage::GROUPED_ACTOR_BALANCES_PAGES4_SLOTS as u32
        }
        MillionStorageBackend::GroupedActorPages8 => {
            static_storage::GROUPED_ACTOR_BALANCES_PAGES8_SLOTS as u32
        }
        MillionStorageBackend::GroupedActorPages16 => {
            static_storage::GROUPED_ACTOR_BALANCES_PAGES16_SLOTS as u32
        }
        MillionStorageBackend::GroupedActorPages32 => {
            static_storage::GROUPED_ACTOR_BALANCES_PAGES32_SLOTS as u32
        }
        MillionStorageBackend::GroupedActorPages64 => {
            static_storage::GROUPED_ACTOR_BALANCES_PAGES64_SLOTS as u32
        }
        MillionStorageBackend::GroupedActorPages128 => {
            static_storage::GROUPED_ACTOR_BALANCES_PAGES128_SLOTS as u32
        }
    }
}

fn allowance_capacity(backend: MillionStorageBackend) -> u32 {
    match backend {
        MillionStorageBackend::GenericStatic => GENERIC_ALLOWANCE_CAPACITY,
        _ => WAT_ALLOWANCE_CAPACITY,
    }
}

#[cfg(any())]
struct InlineOwnerAccountU256Table {
    base: usize,
    slots: usize,
    mask: usize,
}

#[cfg(any())]
impl InlineOwnerAccountU256Table {
    const OCCUPIED: u8 = 1;
    const OWNER_STATE_OFFSET: usize = 0;
    const SPENDER0_STATE_OFFSET: usize = 1;
    const SPENDER1_STATE_OFFSET: usize = 2;
    const OWNER_OFFSET: usize = 8;
    const BALANCE_OFFSET: usize = 40;
    const SPENDER0_OFFSET: usize = 72;
    const ALLOWANCE0_OFFSET: usize = 104;
    const SPENDER1_OFFSET: usize = 136;
    const ALLOWANCE1_OFFSET: usize = 168;
    const SLOT_SIZE: usize = 200;

    #[allow(dead_code)]
    unsafe fn new(base: usize, slots: usize) -> Self {
        debug_assert!(slots.is_power_of_two());
        Self {
            base,
            slots,
            mask: slots - 1,
        }
    }

    fn get_balance(&self, owner: ActorId) -> Option<U256> {
        let slot = unsafe { self.lookup(owner)? };
        Some(unsafe { self.read_u256(slot, Self::BALANCE_OFFSET) })
    }

    unsafe fn insert_balance(&self, owner: ActorId, value: U256) -> Option<U256> {
        let slot = unsafe { self.lookup_or_insert(owner) };
        let previous = unsafe { self.read_u256(slot, Self::BALANCE_OFFSET) };
        unsafe {
            self.write_u256(slot, Self::BALANCE_OFFSET, value);
        }
        (!previous.is_zero()).then_some(previous)
    }

    fn get_inline_allowance(&self, owner: ActorId, spender: ActorId) -> Option<U256> {
        let slot = unsafe { self.lookup(owner)? };
        unsafe { self.inline_allowance_offset(slot, spender) }
            .map(|offset| unsafe { self.read_u256(slot, offset) })
    }

    unsafe fn insert_inline_allowance(
        &self,
        owner: ActorId,
        spender: ActorId,
        value: U256,
    ) -> Option<Option<U256>> {
        let slot = unsafe { self.lookup_or_insert(owner) };
        unsafe {
            if self.read_state(slot, Self::SPENDER0_STATE_OFFSET) == Self::OCCUPIED
                && self.actor_matches(slot, Self::SPENDER0_OFFSET, spender)
            {
                let previous = self.read_u256(slot, Self::ALLOWANCE0_OFFSET);
                self.write_u256(slot, Self::ALLOWANCE0_OFFSET, value);
                return Some(Some(previous));
            }
            if self.read_state(slot, Self::SPENDER1_STATE_OFFSET) == Self::OCCUPIED
                && self.actor_matches(slot, Self::SPENDER1_OFFSET, spender)
            {
                let previous = self.read_u256(slot, Self::ALLOWANCE1_OFFSET);
                self.write_u256(slot, Self::ALLOWANCE1_OFFSET, value);
                return Some(Some(previous));
            }
            if self.read_state(slot, Self::SPENDER0_STATE_OFFSET) != Self::OCCUPIED {
                self.write_state(slot, Self::SPENDER0_STATE_OFFSET, Self::OCCUPIED);
                self.write_actor(slot, Self::SPENDER0_OFFSET, spender);
                self.write_u256(slot, Self::ALLOWANCE0_OFFSET, value);
                return Some(None);
            }
            if self.read_state(slot, Self::SPENDER1_STATE_OFFSET) != Self::OCCUPIED {
                self.write_state(slot, Self::SPENDER1_STATE_OFFSET, Self::OCCUPIED);
                self.write_actor(slot, Self::SPENDER1_OFFSET, spender);
                self.write_u256(slot, Self::ALLOWANCE1_OFFSET, value);
                return Some(None);
            }
        }
        None
    }

    unsafe fn transfer(&self, from: ActorId, to: ActorId, amount: U256) -> bool {
        let Some(from_slot) = (unsafe { self.lookup(from) }) else {
            return false;
        };
        let from_balance = unsafe { self.read_u256(from_slot, Self::BALANCE_OFFSET) };
        if from_balance < amount {
            return false;
        }

        let to_slot = unsafe { self.lookup_or_insert(to) };
        let to_balance = unsafe { self.read_u256(to_slot, Self::BALANCE_OFFSET) };
        let (to_balance, overflow) = to_balance.overflowing_add(amount);
        if overflow {
            return false;
        }
        unsafe {
            self.write_u256(from_slot, Self::BALANCE_OFFSET, from_balance - amount);
            self.write_u256(to_slot, Self::BALANCE_OFFSET, to_balance);
        }
        true
    }

    unsafe fn transfer_from(
        &self,
        overflow_allowances: &WatAllowanceTable,
        owner: ActorId,
        spender: ActorId,
        to: ActorId,
        amount: U256,
    ) -> bool {
        let Some(owner_slot) = (unsafe { self.lookup(owner) }) else {
            return false;
        };

        let inline_allowance_offset = unsafe { self.inline_allowance_offset(owner_slot, spender) };
        let allowance = if let Some(offset) = inline_allowance_offset {
            unsafe { self.read_u256(owner_slot, offset) }
        } else {
            overflow_allowances
                .get_allowance_u256(&owner, &spender)
                .expect("inline owner overflow allowance get failed")
                .unwrap_or_else(U256::zero)
        };

        if allowance < amount {
            return false;
        }
        let owner_balance = unsafe { self.read_u256(owner_slot, Self::BALANCE_OFFSET) };
        if owner_balance < amount {
            return false;
        }

        let to_slot = unsafe { self.lookup_or_insert(to) };
        let to_balance = unsafe { self.read_u256(to_slot, Self::BALANCE_OFFSET) };
        let (to_balance, overflow) = to_balance.overflowing_add(amount);
        if overflow {
            return false;
        }
        unsafe {
            if let Some(offset) = inline_allowance_offset {
                self.write_u256(owner_slot, offset, allowance - amount);
            } else {
                overflow_allowances
                    .insert_allowance_u256(owner, spender, allowance - amount)
                    .expect("inline owner overflow allowance update failed");
            }
            self.write_u256(owner_slot, Self::BALANCE_OFFSET, owner_balance - amount);
            self.write_u256(to_slot, Self::BALANCE_OFFSET, to_balance);
        }
        true
    }

    unsafe fn inline_allowance_offset(&self, slot: usize, spender: ActorId) -> Option<usize> {
        if unsafe {
            self.read_state(slot, Self::SPENDER0_STATE_OFFSET) == Self::OCCUPIED
                && self.actor_matches(slot, Self::SPENDER0_OFFSET, spender)
        } {
            return Some(Self::ALLOWANCE0_OFFSET);
        }
        if unsafe {
            self.read_state(slot, Self::SPENDER1_STATE_OFFSET) == Self::OCCUPIED
                && self.actor_matches(slot, Self::SPENDER1_OFFSET, spender)
        } {
            return Some(Self::ALLOWANCE1_OFFSET);
        }
        None
    }

    unsafe fn lookup(&self, owner: ActorId) -> Option<usize> {
        let mut index = account_index(owner, self.mask);
        for _ in 0..self.slots {
            let occupied =
                unsafe { self.read_state(index, Self::OWNER_STATE_OFFSET) } == Self::OCCUPIED;
            if !occupied {
                return None;
            }
            if unsafe { self.actor_matches(index, Self::OWNER_OFFSET, owner) } {
                return Some(index);
            }
            index = (index + 1) & self.mask;
        }
        None
    }

    unsafe fn lookup_or_insert(&self, owner: ActorId) -> usize {
        let mut index = account_index(owner, self.mask);
        for _ in 0..self.slots {
            let occupied =
                unsafe { self.read_state(index, Self::OWNER_STATE_OFFSET) } == Self::OCCUPIED;
            if occupied && unsafe { self.actor_matches(index, Self::OWNER_OFFSET, owner) } {
                return index;
            }
            if !occupied {
                unsafe {
                    self.write_state(index, Self::OWNER_STATE_OFFSET, Self::OCCUPIED);
                    self.write_actor(index, Self::OWNER_OFFSET, owner);
                }
                return index;
            }
            index = (index + 1) & self.mask;
        }
        panic!("inline owner account table is full")
    }

    unsafe fn actor_matches(&self, slot: usize, offset: usize, actor: ActorId) -> bool {
        unsafe {
            core::slice::from_raw_parts(self.slot_ptr(slot).add(offset), 32) == actor.as_ref()
        }
    }

    unsafe fn read_state(&self, slot: usize, offset: usize) -> u8 {
        unsafe { ptr::read(self.slot_ptr(slot).add(offset)) }
    }

    unsafe fn read_u256(&self, slot: usize, offset: usize) -> U256 {
        let mut bytes = [0u8; 32];
        unsafe {
            ptr::copy_nonoverlapping(self.slot_ptr(slot).add(offset), bytes.as_mut_ptr(), 32);
        }
        U256::from_little_endian(&bytes)
    }

    unsafe fn write_state(&self, slot: usize, offset: usize, state: u8) {
        unsafe {
            ptr::write(self.slot_ptr(slot).add(offset), state);
        }
    }

    unsafe fn write_actor(&self, slot: usize, offset: usize, actor: ActorId) {
        unsafe {
            ptr::copy_nonoverlapping(actor.as_ref().as_ptr(), self.slot_ptr(slot).add(offset), 32);
        }
    }

    unsafe fn write_u256(&self, slot: usize, offset: usize, value: U256) {
        let mut bytes = [0u8; 32];
        value.to_little_endian(&mut bytes);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), self.slot_ptr(slot).add(offset), 32);
        }
    }

    fn slot_ptr(&self, slot: usize) -> *mut u8 {
        (self.base + slot * Self::SLOT_SIZE) as *mut u8
    }
}

#[cfg(target_arch = "wasm32")]
fn generic_balance_table() -> SailsStaticBalanceTable {
    unsafe {
        SailsStaticBalanceTable::new(
            static_storage::MILLION_BALANCES_BASE,
            static_storage::MILLION_BALANCES_SLOTS,
        )
        .expect("million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn generic_balance_table() -> SailsStaticBalanceTable {
    unimplemented!("million static balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn generic_allowance_table() -> SailsStaticAllowanceTable {
    unsafe {
        SailsStaticAllowanceTable::new(
            static_storage::MILLION_ALLOWANCES_BASE,
            static_storage::MILLION_ALLOWANCES_SLOTS,
        )
        .expect("million allowance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn generic_allowance_table() -> SailsStaticAllowanceTable {
    unimplemented!("million static allowance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn wat_actor_balance_table() -> WatActorBalanceTable {
    unsafe {
        WatActorBalanceTable::new(static_storage::WAT_ACTOR_BALANCES_BASE)
            .expect("wat actor million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn wat_actor_balance_table() -> WatActorBalanceTable {
    unimplemented!("wat actor million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn mixed_actor_balance_table() -> MixedActorBalanceTable {
    unsafe {
        MixedActorBalanceTable::new(static_storage::MIXED_ACTOR_BALANCES_BASE)
            .expect("mixed actor million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn mixed_actor_balance_table() -> MixedActorBalanceTable {
    unimplemented!("mixed actor million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn tag_actor_balance_table() -> TagActorBalanceTable {
    unsafe {
        TagActorBalanceTable::new(
            static_storage::GROUPED_ACTOR_BALANCES_PAGES128_BASE,
            static_storage::MILLION_BALANCES_SLOTS,
        )
        .expect("tag actor million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn tag_actor_balance_table() -> TagActorBalanceTable {
    unimplemented!("tag actor million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn tag_u64_balance_table() -> TagU64ActorTable {
    unsafe {
        TagU64ActorTable::new(
            static_storage::GROUPED_ACTOR_BALANCES_PAGES128_BASE,
            static_storage::MILLION_BALANCES_SLOTS * 4,
        )
        .expect("tag u64 actor million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn tag_u64_balance_table() -> TagU64ActorTable {
    unimplemented!("tag u64 actor million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn tag_allowance_table() -> TagActorBalanceTable {
    unsafe {
        TagActorBalanceTable::new(
            static_storage::CONTROL_ACTOR_BALANCES_SLOTS_BASE,
            static_storage::MILLION_ALLOWANCES_SLOTS,
        )
        .expect("tag allowance million table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn tag_allowance_table() -> TagActorBalanceTable {
    unimplemented!("tag allowance million table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn tag_u64_allowance_table() -> TagU64ActorTable {
    unsafe {
        TagU64ActorTable::new(
            static_storage::CONTROL_ACTOR_BALANCES_SLOTS_BASE,
            static_storage::MILLION_ALLOWANCES_SLOTS * 4,
        )
        .expect("tag u64 allowance million table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn tag_u64_allowance_table() -> TagU64ActorTable {
    unimplemented!("tag u64 allowance million table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn inline_owner_account_table() -> InlineOwnerAccountTable {
    unsafe {
        InlineOwnerAccountTable::new(
            static_storage::GROUPED_ACTOR_BALANCES_PAGES16_BASE,
            static_storage::WAT_ALLOWANCES_BASE,
        )
        .expect("inline owner account storage layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn inline_owner_account_table() -> InlineOwnerAccountTable {
    unimplemented!("inline owner account million table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn wat_allowance_table() -> WatAllowanceTable {
    unsafe {
        WatAllowanceTable::new(static_storage::WAT_ALLOWANCES_BASE)
            .expect("wat million allowance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn wat_allowance_table() -> WatAllowanceTable {
    unimplemented!("wat million allowance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn control_actor_balance_table() -> ControlActorBalanceTable {
    unsafe {
        ControlActorBalanceTable::new(
            static_storage::CONTROL_ACTOR_BALANCES_CONTROL_BASE,
            static_storage::CONTROL_ACTOR_BALANCES_SLOTS_BASE,
        )
        .expect("control actor million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn control_actor_balance_table() -> ControlActorBalanceTable {
    unimplemented!("control actor million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn page_local_actor_balance_table() -> PageLocalActorBalanceTable {
    unsafe {
        PageLocalActorBalanceTable::new(static_storage::PAGE_LOCAL_ACTOR_BALANCES_BASE)
            .expect("page-local actor million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn page_local_actor_balance_table() -> PageLocalActorBalanceTable {
    unimplemented!("page-local actor million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn grouped_actor_pages2_table() -> GroupedActorPages2Table {
    unsafe {
        GroupedActorPages2Table::new(static_storage::GROUPED_ACTOR_BALANCES_PAGES2_BASE)
            .expect("grouped actor pages2 million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn grouped_actor_pages2_table() -> GroupedActorPages2Table {
    unimplemented!("grouped actor pages2 million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn grouped_actor_pages4_table() -> GroupedActorPages4Table {
    unsafe {
        GroupedActorPages4Table::new(static_storage::GROUPED_ACTOR_BALANCES_PAGES4_BASE)
            .expect("grouped actor pages4 million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn grouped_actor_pages4_table() -> GroupedActorPages4Table {
    unimplemented!("grouped actor pages4 million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn grouped_actor_pages8_table() -> GroupedActorPages8Table {
    unsafe {
        GroupedActorPages8Table::new(static_storage::GROUPED_ACTOR_BALANCES_PAGES8_BASE)
            .expect("grouped actor pages8 million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn grouped_actor_pages8_table() -> GroupedActorPages8Table {
    unimplemented!("grouped actor pages8 million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn grouped_actor_pages16_table() -> GroupedActorPages16Table {
    unsafe {
        GroupedActorPages16Table::new(static_storage::GROUPED_ACTOR_BALANCES_PAGES16_BASE)
            .expect("grouped actor pages16 million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn grouped_actor_pages16_table() -> GroupedActorPages16Table {
    unimplemented!("grouped actor pages16 million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn grouped_actor_pages32_table() -> GroupedActorPages32Table {
    unsafe {
        GroupedActorPages32Table::new(static_storage::GROUPED_ACTOR_BALANCES_PAGES32_BASE)
            .expect("grouped actor pages32 million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn grouped_actor_pages32_table() -> GroupedActorPages32Table {
    unimplemented!("grouped actor pages32 million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn grouped_actor_pages64_table() -> GroupedActorPages64Table {
    unsafe {
        GroupedActorPages64Table::new(static_storage::GROUPED_ACTOR_BALANCES_PAGES64_BASE)
            .expect("grouped actor pages64 million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn grouped_actor_pages64_table() -> GroupedActorPages64Table {
    unimplemented!("grouped actor pages64 million balance table is available only on wasm32")
}

#[cfg(target_arch = "wasm32")]
fn grouped_actor_pages128_table() -> GroupedActorPages128Table {
    unsafe {
        GroupedActorPages128Table::new(static_storage::GROUPED_ACTOR_BALANCES_PAGES128_BASE)
            .expect("grouped actor pages128 million balance table layout is valid")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn grouped_actor_pages128_table() -> GroupedActorPages128Table {
    unimplemented!("grouped actor pages128 million balance table is available only on wasm32")
}

fn actor_for_seed(seed: u32) -> ActorId {
    ActorId::from(seed as u64 + 1)
}

fn vft_actor_for_seed(seed: u32) -> ActorId {
    actor_from_u64(vft_random_u64(seed, 0xA11C_E001_D15C_A11C))
}

fn vft_actor_tag_for_seed(seed: u32) -> u64 {
    vft_random_u64(seed, 0xA11C_E001_D15C_A11C).max(1)
}

fn vft_spender_for_seed(seed: u32) -> ActorId {
    actor_from_u64(vft_random_u64(seed, 0x5EED_5EED_A110_CAFE))
}

fn vft_spender_tag_for_seed(seed: u32) -> u64 {
    vft_random_u64(seed, 0x5EED_5EED_A110_CAFE).max(1)
}

#[cfg(any())]
fn account_index(actor: ActorId, mask: usize) -> usize {
    let key = actor.as_ref();
    let hot_lane = unsafe { ptr::read_unaligned(key.as_ptr().add(12).cast::<u64>()) };
    let hash = if hot_lane != 0 {
        hot_lane
    } else {
        unsafe {
            ptr::read_unaligned(key.as_ptr().cast::<u64>())
                ^ ptr::read_unaligned(key.as_ptr().add(8).cast::<u64>())
                ^ ptr::read_unaligned(key.as_ptr().add(16).cast::<u64>())
                ^ ptr::read_unaligned(key.as_ptr().add(24).cast::<u64>())
        }
    };
    hash as usize & mask
}

fn vft_allowance_tag_for_seeds(owner_seed: u32, spender_seed: u32) -> u64 {
    vft_actor_tag_for_seed(owner_seed)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(vft_spender_tag_for_seed(spender_seed).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        .max(1)
}

fn actor_from_u64(id: u64) -> ActorId {
    ActorId::from(id.max(1))
}

fn vft_random_u64(seed: u32, domain: u64) -> u64 {
    let state = u64::from(seed)
        .wrapping_add(domain)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    state ^ (state >> 32)
}

fn value_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1)
}

fn updated_value_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1_000_001)
}

fn vft_balance_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1_000_000)
}

fn vft_allowance_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 10_000)
}

fn vft_allowance_for_seed_u64(seed: u32) -> u64 {
    seed as u64 + 10_000
}

fn vft_transfer_amount(seed: u32) -> U256 {
    U256::from((seed % 7) as u64 + 1)
}

fn vft_transfer_amount_u64(seed: u32) -> u64 {
    (seed % 7) as u64 + 1
}
