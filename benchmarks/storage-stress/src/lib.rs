#![no_std]

use sails_rs::{cell::RefCell, collections::HashMap, prelude::*};
use sails_storage::gear::{
    FixedAllowanceMap as SailsFixedAllowanceMap, FixedBalanceMap as SailsFixedBalanceMap,
    StaticAllowanceTable as SailsStaticAllowanceTable,
    StaticBalanceTable as SailsStaticBalanceTable,
};

mod static_storage {
    include!(concat!(env!("OUT_DIR"), "/sails_static_storage.rs"));
}

const FIXED_CAPACITY: usize = 2048;

pub const STATIC_MEMORY_END_PAGE: u32 = static_storage::STATIC_MEMORY_END_PAGE;

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageBackend {
    HashMap,
    Fixed,
    RawStatic,
    SailsFixed,
    SailsStatic,
}

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageMap {
    Balance,
    Allowance,
}

#[sails_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageOp {
    InsertFresh,
    UpdateExisting,
    ReadExisting,
    ReadMissing,
    Remove,
}

#[sails_type]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageBenchResult {
    pub value: U256,
    pub len: u32,
    pub existed: bool,
}

struct StorageStressService<'a> {
    state: &'a RefCell<StorageStressState>,
}

impl<'a> StorageStressService<'a> {
    fn new(state: &'a RefCell<StorageStressState>) -> Self {
        Self { state }
    }
}

#[sails_rs::service]
impl StorageStressService<'_> {
    #[export]
    pub fn prepare(
        &mut self,
        backend: StorageBackend,
        map: StorageMap,
        len: u32,
    ) -> StorageBenchResult {
        self.state.borrow_mut().prepare(backend, map, len)
    }

    #[export]
    pub fn bench(
        &mut self,
        backend: StorageBackend,
        map: StorageMap,
        op: StorageOp,
        seed: u32,
    ) -> StorageBenchResult {
        self.state.borrow_mut().bench(backend, map, op, seed)
    }
}

pub struct StorageStressProgram {
    state: RefCell<StorageStressState>,
}

#[sails_rs::program]
impl StorageStressProgram {
    pub fn new_for_bench() -> Self {
        Self {
            state: RefCell::new(StorageStressState::new()),
        }
    }

    pub fn storage_stress(&self) -> StorageStressService<'_> {
        StorageStressService::new(&self.state)
    }
}

struct StorageStressState {
    storage: ActiveStorage,
}

enum ActiveStorage {
    Empty,
    HashBalances(HashMap<ActorId, U256>),
    HashAllowances(HashMap<(ActorId, ActorId), U256>),
    FixedBalances(FixedBalanceMap<FIXED_CAPACITY>),
    FixedAllowances(FixedAllowanceMap<FIXED_CAPACITY>),
    RawBalances(RawStaticBalanceMap<FIXED_CAPACITY>),
    RawAllowances(RawStaticAllowanceMap<FIXED_CAPACITY>),
    SailsFixedBalances(SailsFixedBalanceMap<FIXED_CAPACITY>),
    SailsFixedAllowances(SailsFixedAllowanceMap<FIXED_CAPACITY>),
    SailsStaticBalances(SailsStaticBalanceMap),
    SailsStaticAllowances(SailsStaticAllowanceMap),
}

impl StorageStressState {
    fn new() -> Self {
        Self {
            storage: ActiveStorage::Empty,
        }
    }

    fn prepare(
        &mut self,
        backend: StorageBackend,
        map: StorageMap,
        len: u32,
    ) -> StorageBenchResult {
        assert!((len as usize) < FIXED_CAPACITY);
        self.clear(backend, map);

        for seed in 1..=len {
            self.insert(backend, map, seed, value_for_seed(seed));
        }

        StorageBenchResult {
            value: value_for_seed(len),
            len,
            existed: false,
        }
    }

    fn bench(
        &mut self,
        backend: StorageBackend,
        map: StorageMap,
        op: StorageOp,
        seed: u32,
    ) -> StorageBenchResult {
        match op {
            StorageOp::InsertFresh => {
                let value = value_for_seed(seed);
                let existed = self.insert(backend, map, seed, value).is_some();
                StorageBenchResult {
                    value,
                    len: self.len(backend, map),
                    existed,
                }
            }
            StorageOp::UpdateExisting => {
                let value = updated_value_for_seed(seed);
                let existed = self.insert(backend, map, seed, value).is_some();
                StorageBenchResult {
                    value,
                    len: self.len(backend, map),
                    existed,
                }
            }
            StorageOp::ReadExisting | StorageOp::ReadMissing => {
                let value = self.get(backend, map, seed).unwrap_or_else(U256::zero);
                StorageBenchResult {
                    value,
                    len: self.len(backend, map),
                    existed: !value.is_zero(),
                }
            }
            StorageOp::Remove => {
                let value = self.remove(backend, map, seed).unwrap_or_else(U256::zero);
                StorageBenchResult {
                    value,
                    len: self.len(backend, map),
                    existed: !value.is_zero(),
                }
            }
        }
    }

    fn clear(&mut self, backend: StorageBackend, map: StorageMap) {
        self.storage = match (backend, map) {
            (StorageBackend::HashMap, StorageMap::Balance) => {
                ActiveStorage::HashBalances(HashMap::new())
            }
            (StorageBackend::HashMap, StorageMap::Allowance) => {
                ActiveStorage::HashAllowances(HashMap::new())
            }
            (StorageBackend::Fixed, StorageMap::Balance) => {
                ActiveStorage::FixedBalances(FixedBalanceMap::new())
            }
            (StorageBackend::Fixed, StorageMap::Allowance) => {
                ActiveStorage::FixedAllowances(FixedAllowanceMap::new())
            }
            (StorageBackend::RawStatic, StorageMap::Balance) => {
                ActiveStorage::RawBalances(RawStaticBalanceMap::new())
            }
            (StorageBackend::RawStatic, StorageMap::Allowance) => {
                ActiveStorage::RawAllowances(RawStaticAllowanceMap::new())
            }
            (StorageBackend::SailsFixed, StorageMap::Balance) => {
                ActiveStorage::SailsFixedBalances(SailsFixedBalanceMap::new())
            }
            (StorageBackend::SailsFixed, StorageMap::Allowance) => {
                ActiveStorage::SailsFixedAllowances(SailsFixedAllowanceMap::new())
            }
            (StorageBackend::SailsStatic, StorageMap::Balance) => {
                ActiveStorage::SailsStaticBalances(SailsStaticBalanceMap::new())
            }
            (StorageBackend::SailsStatic, StorageMap::Allowance) => {
                ActiveStorage::SailsStaticAllowances(SailsStaticAllowanceMap::new())
            }
        };
    }

    fn len(&self, backend: StorageBackend, map: StorageMap) -> u32 {
        match (backend, map, &self.storage) {
            (StorageBackend::HashMap, StorageMap::Balance, ActiveStorage::HashBalances(map)) => {
                map.len() as u32
            }
            (
                StorageBackend::HashMap,
                StorageMap::Allowance,
                ActiveStorage::HashAllowances(map),
            ) => map.len() as u32,
            (StorageBackend::Fixed, StorageMap::Balance, ActiveStorage::FixedBalances(map)) => {
                map.len()
            }
            (StorageBackend::Fixed, StorageMap::Allowance, ActiveStorage::FixedAllowances(map)) => {
                map.len()
            }
            (StorageBackend::RawStatic, StorageMap::Balance, ActiveStorage::RawBalances(map)) => {
                map.len()
            }
            (
                StorageBackend::RawStatic,
                StorageMap::Allowance,
                ActiveStorage::RawAllowances(map),
            ) => map.len(),
            (
                StorageBackend::SailsFixed,
                StorageMap::Balance,
                ActiveStorage::SailsFixedBalances(map),
            ) => map.len() as u32,
            (
                StorageBackend::SailsFixed,
                StorageMap::Allowance,
                ActiveStorage::SailsFixedAllowances(map),
            ) => map.len() as u32,
            (
                StorageBackend::SailsStatic,
                StorageMap::Balance,
                ActiveStorage::SailsStaticBalances(map),
            ) => map.len(),
            (
                StorageBackend::SailsStatic,
                StorageMap::Allowance,
                ActiveStorage::SailsStaticAllowances(map),
            ) => map.len(),
            _ => 0,
        }
    }

    fn insert(
        &mut self,
        backend: StorageBackend,
        map: StorageMap,
        seed: u32,
        value: U256,
    ) -> Option<U256> {
        match (backend, map, &mut self.storage) {
            (StorageBackend::HashMap, StorageMap::Balance, ActiveStorage::HashBalances(map)) => {
                map.insert(actor_for_seed(seed), value)
            }
            (
                StorageBackend::HashMap,
                StorageMap::Allowance,
                ActiveStorage::HashAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.insert((owner, spender), value)
            }
            (StorageBackend::Fixed, StorageMap::Balance, ActiveStorage::FixedBalances(map)) => map
                .try_insert(actor_for_seed(seed), value)
                .expect("fixed balance map capacity exceeded"),
            (StorageBackend::Fixed, StorageMap::Allowance, ActiveStorage::FixedAllowances(map)) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.try_insert(AllowanceKey { owner, spender }, value)
                    .expect("fixed allowance map capacity exceeded")
            }
            (StorageBackend::RawStatic, StorageMap::Balance, ActiveStorage::RawBalances(map)) => {
                map.try_insert(actor_for_seed(seed), value)
                    .expect("raw static balance map capacity exceeded")
            }
            (
                StorageBackend::RawStatic,
                StorageMap::Allowance,
                ActiveStorage::RawAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.try_insert(AllowanceKey { owner, spender }, value)
                    .expect("raw static allowance map capacity exceeded")
            }
            (
                StorageBackend::SailsFixed,
                StorageMap::Balance,
                ActiveStorage::SailsFixedBalances(map),
            ) => map
                .insert_actor_u256(actor_for_seed(seed), value)
                .expect("sails-storage fixed balance map failed"),
            (
                StorageBackend::SailsFixed,
                StorageMap::Allowance,
                ActiveStorage::SailsFixedAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.insert_allowance_u256(owner, spender, value)
                    .expect("sails-storage fixed allowance map failed")
            }
            (
                StorageBackend::SailsStatic,
                StorageMap::Balance,
                ActiveStorage::SailsStaticBalances(map),
            ) => map
                .try_insert(actor_for_seed(seed), value)
                .expect("sails-storage static balance map failed"),
            (
                StorageBackend::SailsStatic,
                StorageMap::Allowance,
                ActiveStorage::SailsStaticAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.try_insert(owner, spender, value)
                    .expect("sails-storage static allowance map failed")
            }
            _ => panic!("storage backend is not prepared"),
        }
    }

    fn get(&self, backend: StorageBackend, map: StorageMap, seed: u32) -> Option<U256> {
        match (backend, map, &self.storage) {
            (StorageBackend::HashMap, StorageMap::Balance, ActiveStorage::HashBalances(map)) => {
                map.get(&actor_for_seed(seed)).copied()
            }
            (
                StorageBackend::HashMap,
                StorageMap::Allowance,
                ActiveStorage::HashAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.get(&(owner, spender)).copied()
            }
            (StorageBackend::Fixed, StorageMap::Balance, ActiveStorage::FixedBalances(map)) => {
                map.get(&actor_for_seed(seed))
            }
            (StorageBackend::Fixed, StorageMap::Allowance, ActiveStorage::FixedAllowances(map)) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.get(&AllowanceKey { owner, spender })
            }
            (StorageBackend::RawStatic, StorageMap::Balance, ActiveStorage::RawBalances(map)) => {
                map.get(&actor_for_seed(seed))
            }
            (
                StorageBackend::RawStatic,
                StorageMap::Allowance,
                ActiveStorage::RawAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.get(&AllowanceKey { owner, spender })
            }
            (
                StorageBackend::SailsFixed,
                StorageMap::Balance,
                ActiveStorage::SailsFixedBalances(map),
            ) => map
                .get_actor_u256(&actor_for_seed(seed))
                .expect("sails-storage fixed balance map failed"),
            (
                StorageBackend::SailsFixed,
                StorageMap::Allowance,
                ActiveStorage::SailsFixedAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.get_allowance_u256(&owner, &spender)
                    .expect("sails-storage fixed allowance map failed")
            }
            (
                StorageBackend::SailsStatic,
                StorageMap::Balance,
                ActiveStorage::SailsStaticBalances(map),
            ) => map
                .get(&actor_for_seed(seed))
                .expect("sails-storage static balance map failed"),
            (
                StorageBackend::SailsStatic,
                StorageMap::Allowance,
                ActiveStorage::SailsStaticAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.get(&owner, &spender)
                    .expect("sails-storage static allowance map failed")
            }
            _ => None,
        }
    }

    fn remove(&mut self, backend: StorageBackend, map: StorageMap, seed: u32) -> Option<U256> {
        match (backend, map, &mut self.storage) {
            (StorageBackend::HashMap, StorageMap::Balance, ActiveStorage::HashBalances(map)) => {
                map.remove(&actor_for_seed(seed))
            }
            (
                StorageBackend::HashMap,
                StorageMap::Allowance,
                ActiveStorage::HashAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.remove(&(owner, spender))
            }
            (StorageBackend::Fixed, StorageMap::Balance, ActiveStorage::FixedBalances(map)) => {
                map.remove(&actor_for_seed(seed))
            }
            (StorageBackend::Fixed, StorageMap::Allowance, ActiveStorage::FixedAllowances(map)) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.remove(&AllowanceKey { owner, spender })
            }
            (StorageBackend::RawStatic, StorageMap::Balance, ActiveStorage::RawBalances(map)) => {
                map.remove(&actor_for_seed(seed))
            }
            (
                StorageBackend::RawStatic,
                StorageMap::Allowance,
                ActiveStorage::RawAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.remove(&AllowanceKey { owner, spender })
            }
            (
                StorageBackend::SailsFixed,
                StorageMap::Balance,
                ActiveStorage::SailsFixedBalances(map),
            ) => map
                .remove_actor_u256(&actor_for_seed(seed))
                .expect("sails-storage fixed balance map failed"),
            (
                StorageBackend::SailsFixed,
                StorageMap::Allowance,
                ActiveStorage::SailsFixedAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.remove_allowance_u256(&owner, &spender)
                    .expect("sails-storage fixed allowance map failed")
            }
            (
                StorageBackend::SailsStatic,
                StorageMap::Balance,
                ActiveStorage::SailsStaticBalances(map),
            ) => map
                .remove(&actor_for_seed(seed))
                .expect("sails-storage static balance map failed"),
            (
                StorageBackend::SailsStatic,
                StorageMap::Allowance,
                ActiveStorage::SailsStaticAllowances(map),
            ) => {
                let (owner, spender) = allowance_key_for_seed(seed);
                map.remove(&owner, &spender)
                    .expect("sails-storage static allowance map failed")
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct BalanceSlot {
    key: Option<ActorId>,
    value: Option<U256>,
    tombstone: bool,
}

impl BalanceSlot {
    const EMPTY: Self = Self {
        key: None,
        value: None,
        tombstone: false,
    };
}

struct FixedBalanceMap<const CAP: usize> {
    slots: [BalanceSlot; CAP],
    len: u32,
}

impl<const CAP: usize> FixedBalanceMap<CAP> {
    const fn new() -> Self {
        Self {
            slots: [BalanceSlot::EMPTY; CAP],
            len: 0,
        }
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn get(&self, key: &ActorId) -> Option<U256> {
        self.find_slot(*key)
            .ok()
            .and_then(|idx| self.slots[idx].value)
    }

    fn try_insert(&mut self, key: ActorId, value: U256) -> Result<Option<U256>, FixedMapFull> {
        match self.find_slot(key) {
            Ok(idx) => Ok(self.slots[idx].value.replace(value)),
            Err(idx) if idx < CAP => {
                self.slots[idx] = BalanceSlot {
                    key: Some(key),
                    value: Some(value),
                    tombstone: false,
                };
                self.len += 1;
                Ok(None)
            }
            Err(_) => Err(FixedMapFull),
        }
    }

    fn remove(&mut self, key: &ActorId) -> Option<U256> {
        let idx = self.find_slot(*key).ok()?;
        let previous = self.slots[idx].value.take();
        self.slots[idx].key = None;
        self.slots[idx].tombstone = true;
        self.len -= 1;
        previous
    }

    fn find_slot(&self, key: ActorId) -> Result<usize, usize> {
        find_slot(CAP, hash_actor(&key), |idx| {
            let slot = &self.slots[idx];
            match slot.key {
                Some(existing) if existing == key => Probe::Found,
                Some(_) => Probe::Occupied,
                None if slot.tombstone => Probe::Tombstone,
                None => Probe::Empty,
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct AllowanceKey {
    owner: ActorId,
    spender: ActorId,
}

#[derive(Clone, Copy)]
struct AllowanceSlot {
    key: Option<AllowanceKey>,
    value: Option<U256>,
    tombstone: bool,
}

impl AllowanceSlot {
    const EMPTY: Self = Self {
        key: None,
        value: None,
        tombstone: false,
    };
}

struct FixedAllowanceMap<const CAP: usize> {
    slots: [AllowanceSlot; CAP],
    len: u32,
}

impl<const CAP: usize> FixedAllowanceMap<CAP> {
    const fn new() -> Self {
        Self {
            slots: [AllowanceSlot::EMPTY; CAP],
            len: 0,
        }
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn get(&self, key: &AllowanceKey) -> Option<U256> {
        self.find_slot(*key)
            .ok()
            .and_then(|idx| self.slots[idx].value)
    }

    fn try_insert(&mut self, key: AllowanceKey, value: U256) -> Result<Option<U256>, FixedMapFull> {
        match self.find_slot(key) {
            Ok(idx) => Ok(self.slots[idx].value.replace(value)),
            Err(idx) if idx < CAP => {
                self.slots[idx] = AllowanceSlot {
                    key: Some(key),
                    value: Some(value),
                    tombstone: false,
                };
                self.len += 1;
                Ok(None)
            }
            Err(_) => Err(FixedMapFull),
        }
    }

    fn remove(&mut self, key: &AllowanceKey) -> Option<U256> {
        let idx = self.find_slot(*key).ok()?;
        let previous = self.slots[idx].value.take();
        self.slots[idx].key = None;
        self.slots[idx].tombstone = true;
        self.len -= 1;
        previous
    }

    fn find_slot(&self, key: AllowanceKey) -> Result<usize, usize> {
        find_slot(CAP, hash_allowance(&key), |idx| {
            let slot = &self.slots[idx];
            match slot.key {
                Some(existing) if existing == key => Probe::Found,
                Some(_) => Probe::Occupied,
                None if slot.tombstone => Probe::Tombstone,
                None => Probe::Empty,
            }
        })
    }
}

type RawStaticBalanceMap<const CAP: usize> = RawStaticMap<32, 32, CAP>;
type RawStaticAllowanceMap<const CAP: usize> = RawStaticMap<64, 32, CAP>;

#[derive(Clone, Copy)]
struct RawStaticSlot<const KEY_SIZE: usize, const VALUE_SIZE: usize> {
    key: [u8; KEY_SIZE],
    value: [u8; VALUE_SIZE],
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize> RawStaticSlot<KEY_SIZE, VALUE_SIZE> {
    const EMPTY: Self = Self {
        key: [0; KEY_SIZE],
        value: [0; VALUE_SIZE],
    };
}

struct RawStaticMap<const KEY_SIZE: usize, const VALUE_SIZE: usize, const CAP: usize> {
    slots: [RawStaticSlot<KEY_SIZE, VALUE_SIZE>; CAP],
    len: u32,
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize, const CAP: usize>
    RawStaticMap<KEY_SIZE, VALUE_SIZE, CAP>
{
    const fn new() -> Self {
        Self {
            slots: [RawStaticSlot::EMPTY; CAP],
            len: 0,
        }
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn get_raw_with_hash(&self, key: &[u8; KEY_SIZE], hash: usize) -> Option<[u8; VALUE_SIZE]> {
        let idx = self.find_slot_with_hash(key, hash).ok()?;
        let value = self.slots[idx].value;
        (!is_zero(&value)).then_some(value)
    }

    fn try_insert_raw(
        &mut self,
        key: [u8; KEY_SIZE],
        value: [u8; VALUE_SIZE],
        hash: usize,
    ) -> Result<Option<[u8; VALUE_SIZE]>, FixedMapFull> {
        debug_assert!(!is_zero(&key));

        if is_zero(&value) {
            return Ok(self.remove_with_hash(&key, hash));
        }

        match self.find_slot_with_hash(&key, hash) {
            Ok(idx) => {
                let previous = self.slots[idx].value;
                self.slots[idx].value = value;
                if is_zero(&previous) {
                    self.len += 1;
                    Ok(None)
                } else {
                    Ok(Some(previous))
                }
            }
            Err(idx) if idx < CAP => {
                self.slots[idx] = RawStaticSlot { key, value };
                self.len += 1;
                Ok(None)
            }
            Err(_) => Err(FixedMapFull),
        }
    }

    fn remove_with_hash(&mut self, key: &[u8; KEY_SIZE], hash: usize) -> Option<[u8; VALUE_SIZE]> {
        let idx = self.find_slot_with_hash(key, hash).ok()?;
        let previous = self.slots[idx].value;
        if is_zero(&previous) {
            None
        } else {
            self.slots[idx].value = [0; VALUE_SIZE];
            self.len -= 1;
            Some(previous)
        }
    }

    fn find_slot_with_hash(&self, key: &[u8; KEY_SIZE], hash: usize) -> Result<usize, usize> {
        find_slot(CAP, hash, |idx| {
            let slot = &self.slots[idx];
            if is_zero(&slot.key) {
                Probe::Empty
            } else if slot.key == *key {
                Probe::Found
            } else if is_zero(&slot.value) {
                Probe::Tombstone
            } else {
                Probe::Occupied
            }
        })
    }
}

impl<const CAP: usize> RawStaticBalanceMap<CAP> {
    fn get(&self, key: &ActorId) -> Option<U256> {
        let raw_key = raw_actor_key(*key);
        self.get_raw_with_hash(&raw_key, hash_actor(key))
            .map(u256_from_raw)
    }

    fn try_insert(&mut self, key: ActorId, value: U256) -> Result<Option<U256>, FixedMapFull> {
        let raw_key = raw_actor_key(key);
        let raw_value = raw_u256(value);
        self.try_insert_raw(raw_key, raw_value, hash_actor(&key))
            .map(|previous| previous.map(u256_from_raw))
    }

    fn remove(&mut self, key: &ActorId) -> Option<U256> {
        let raw_key = raw_actor_key(*key);
        self.remove_with_hash(&raw_key, hash_actor(key))
            .map(u256_from_raw)
    }
}

impl<const CAP: usize> RawStaticAllowanceMap<CAP> {
    fn get(&self, key: &AllowanceKey) -> Option<U256> {
        let raw_key = raw_allowance_key(*key);
        self.get_raw_with_hash(&raw_key, hash_allowance(key))
            .map(u256_from_raw)
    }

    fn try_insert(&mut self, key: AllowanceKey, value: U256) -> Result<Option<U256>, FixedMapFull> {
        let raw_key = raw_allowance_key(key);
        let raw_value = raw_u256(value);
        self.try_insert_raw(raw_key, raw_value, hash_allowance(&key))
            .map(|previous| previous.map(u256_from_raw))
    }

    fn remove(&mut self, key: &AllowanceKey) -> Option<U256> {
        let raw_key = raw_allowance_key(*key);
        self.remove_with_hash(&raw_key, hash_allowance(key))
            .map(u256_from_raw)
    }
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

    fn remove(&mut self, key: &ActorId) -> Result<Option<U256>, sails_storage::TableError> {
        let previous = self.table_mut().remove_actor_u256(key)?;
        if previous.is_some() {
            self.len -= 1;
        }

        Ok(previous)
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

    fn remove(
        &mut self,
        owner: &ActorId,
        spender: &ActorId,
    ) -> Result<Option<U256>, sails_storage::TableError> {
        let previous = self.table_mut().remove_allowance_u256(owner, spender)?;
        if previous.is_some() {
            self.len -= 1;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FixedMapFull;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Probe {
    Found,
    Occupied,
    Tombstone,
    Empty,
}

fn find_slot(cap: usize, hash: usize, probe: impl Fn(usize) -> Probe) -> Result<usize, usize> {
    debug_assert!(cap > 0);

    let mut first_tombstone = None;
    let mut idx = hash % cap;

    for _ in 0..cap {
        match probe(idx) {
            Probe::Found => return Ok(idx),
            Probe::Occupied => {}
            Probe::Tombstone => {
                if first_tombstone.is_none() {
                    first_tombstone = Some(idx);
                }
            }
            Probe::Empty => return Err(first_tombstone.unwrap_or(idx)),
        }

        idx = (idx + 1) % cap;
    }

    Err(first_tombstone.unwrap_or(cap))
}

fn actor_for_seed(seed: u32) -> ActorId {
    ActorId::from(seed as u64 + 1)
}

fn allowance_key_for_seed(seed: u32) -> (ActorId, ActorId) {
    (
        ActorId::from((seed as u64).wrapping_mul(2).wrapping_add(1)),
        ActorId::from((seed as u64).wrapping_mul(2).wrapping_add(2)),
    )
}

fn value_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1)
}

fn updated_value_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1_000_001)
}

fn hash_actor(actor: &ActorId) -> usize {
    let bytes: &[u8] = actor.as_ref();
    hash_bytes(bytes).wrapping_mul(0x9e37_79b9)
}

fn hash_bytes(bytes: &[u8]) -> usize {
    let mut hash = 0x811c_9dc5u32;

    for byte in bytes {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }

    hash as usize
}

fn hash_allowance(key: &AllowanceKey) -> usize {
    hash_actor(&key.owner).rotate_left(13).wrapping_mul(31) ^ hash_actor(&key.spender)
}

fn raw_actor_key(actor: ActorId) -> [u8; 32] {
    let mut key = [0u8; 32];
    key.copy_from_slice(actor.as_ref());
    key
}

fn raw_allowance_key(key: AllowanceKey) -> [u8; 64] {
    let mut raw = [0u8; 64];
    raw[..32].copy_from_slice(key.owner.as_ref());
    raw[32..].copy_from_slice(key.spender.as_ref());
    raw
}

fn raw_u256(value: U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    value.to_little_endian(&mut bytes);
    bytes
}

fn u256_from_raw(bytes: [u8; 32]) -> U256 {
    U256::from_little_endian(&bytes)
}

fn is_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::thread;

    #[test]
    fn fixed_balance_map_updates_existing_key() {
        let mut map = FixedBalanceMap::<8>::new();
        let key = actor_for_seed(1);

        assert_eq!(map.try_insert(key, value_for_seed(1)), Ok(None));
        assert_eq!(
            map.try_insert(key, updated_value_for_seed(1)),
            Ok(Some(value_for_seed(1)))
        );
        assert_eq!(map.get(&key), Some(updated_value_for_seed(1)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn fixed_balance_map_reuses_tombstone() {
        let mut map = FixedBalanceMap::<1>::new();
        let first = actor_for_seed(1);
        let second = actor_for_seed(2);

        assert_eq!(map.try_insert(first, value_for_seed(1)), Ok(None));
        assert_eq!(map.remove(&first), Some(value_for_seed(1)));
        assert_eq!(map.try_insert(second, value_for_seed(2)), Ok(None));
        assert_eq!(map.get(&second), Some(value_for_seed(2)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn fixed_balance_map_reports_full_table() {
        let mut map = FixedBalanceMap::<1>::new();

        assert_eq!(
            map.try_insert(actor_for_seed(1), value_for_seed(1)),
            Ok(None)
        );
        assert_eq!(
            map.try_insert(actor_for_seed(2), value_for_seed(2)),
            Err(FixedMapFull)
        );
    }

    #[test]
    fn fixed_allowance_map_removes_pair_key() {
        let mut map = FixedAllowanceMap::<8>::new();
        let (owner, spender) = allowance_key_for_seed(1);
        let key = AllowanceKey { owner, spender };

        assert_eq!(map.try_insert(key, value_for_seed(1)), Ok(None));
        assert_eq!(map.remove(&key), Some(value_for_seed(1)));
        assert_eq!(map.get(&key), None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn raw_static_balance_map_updates_existing_key() {
        let mut map = RawStaticBalanceMap::<8>::new();
        let key = actor_for_seed(1);

        assert_eq!(map.try_insert(key, value_for_seed(1)), Ok(None));
        assert_eq!(
            map.try_insert(key, updated_value_for_seed(1)),
            Ok(Some(value_for_seed(1)))
        );
        assert_eq!(map.get(&key), Some(updated_value_for_seed(1)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn raw_static_balance_map_reuses_tombstone() {
        let mut map = RawStaticBalanceMap::<1>::new();
        let first = actor_for_seed(1);
        let second = actor_for_seed(2);

        assert_eq!(map.try_insert(first, value_for_seed(1)), Ok(None));
        assert_eq!(map.remove(&first), Some(value_for_seed(1)));
        assert_eq!(map.try_insert(second, value_for_seed(2)), Ok(None));
        assert_eq!(map.get(&second), Some(value_for_seed(2)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn raw_static_balance_map_zero_value_removes_existing_key() {
        let mut map = RawStaticBalanceMap::<8>::new();
        let key = actor_for_seed(1);

        assert_eq!(map.try_insert(key, value_for_seed(1)), Ok(None));
        assert_eq!(
            map.try_insert(key, U256::zero()),
            Ok(Some(value_for_seed(1)))
        );
        assert_eq!(map.get(&key), None);
        assert_eq!(map.len(), 0);

        assert_eq!(map.try_insert(key, value_for_seed(2)), Ok(None));
        assert_eq!(map.get(&key), Some(value_for_seed(2)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn raw_static_balance_map_reports_full_table() {
        let mut map = RawStaticBalanceMap::<1>::new();

        assert_eq!(
            map.try_insert(actor_for_seed(1), value_for_seed(1)),
            Ok(None)
        );
        assert_eq!(
            map.try_insert(actor_for_seed(2), value_for_seed(2)),
            Err(FixedMapFull)
        );
    }

    #[test]
    fn raw_static_allowance_map_removes_pair_key() {
        let mut map = RawStaticAllowanceMap::<8>::new();
        let (owner, spender) = allowance_key_for_seed(1);
        let key = AllowanceKey { owner, spender };

        assert_eq!(map.try_insert(key, value_for_seed(1)), Ok(None));
        assert_eq!(map.remove(&key), Some(value_for_seed(1)));
        assert_eq!(map.get(&key), None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn raw_static_state_matches_fixed_for_representative_ops() {
        thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                for map in [StorageMap::Balance, StorageMap::Allowance] {
                    for op in [
                        StorageOp::InsertFresh,
                        StorageOp::UpdateExisting,
                        StorageOp::ReadExisting,
                        StorageOp::ReadMissing,
                        StorageOp::Remove,
                    ] {
                        let seed = match op {
                            StorageOp::InsertFresh | StorageOp::ReadMissing => 10_016,
                            StorageOp::UpdateExisting
                            | StorageOp::ReadExisting
                            | StorageOp::Remove => 4,
                        };

                        let fixed_result = {
                            let mut state = StorageStressState::new();
                            assert_eq!(state.prepare(StorageBackend::Fixed, map, 16).len, 16);
                            state.bench(StorageBackend::Fixed, map, op, seed)
                        };
                        for backend in [
                            StorageBackend::RawStatic,
                            StorageBackend::SailsFixed,
                            StorageBackend::SailsStatic,
                        ] {
                            let candidate_result = {
                                let mut state = StorageStressState::new();
                                assert_eq!(state.prepare(backend, map, 16).len, 16);
                                state.bench(backend, map, op, seed)
                            };

                            assert_eq!(fixed_result, candidate_result);
                        }
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
