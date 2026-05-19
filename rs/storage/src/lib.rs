#![no_std]

use core::{fmt, marker::PhantomData, ptr};

/// Errors returned by fixed and static storage tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableError {
    /// The table has no reusable slot for a new key.
    CapacityOverflow,
    /// The key cannot be represented by this table layout.
    InvalidKey,
    /// The provided memory layout overflows or does not fit the requested region.
    InvalidLayout,
    /// A static-memory slot state byte is not one of the supported values.
    InvalidSlotState,
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityOverflow => f.write_str("capacity overflow"),
            Self::InvalidKey => f.write_str("invalid key"),
            Self::InvalidLayout => f.write_str("invalid storage layout"),
            Self::InvalidSlotState => f.write_str("invalid slot state"),
        }
    }
}

/// The state byte stored before every static-memory slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SlotState {
    Empty = 0,
    Full = 1,
    Deleted = 2,
}

impl SlotState {
    fn from_byte(byte: u8) -> Result<Self, TableError> {
        match byte {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Full),
            2 => Ok(Self::Deleted),
            _ => Err(TableError::InvalidSlotState),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotMatch {
    Found,
    Occupied,
    Deleted,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lookup {
    Found(usize),
    Vacant(usize),
    Full,
}

fn find_slot(
    slots: usize,
    hash: usize,
    mut classify: impl FnMut(usize) -> Result<SlotMatch, TableError>,
) -> Result<Lookup, TableError> {
    if slots == 0 {
        return Ok(Lookup::Full);
    }

    let mut first_deleted = None;
    let mut index = hash % slots;

    for _ in 0..slots {
        match classify(index)? {
            SlotMatch::Found => return Ok(Lookup::Found(index)),
            SlotMatch::Occupied => {}
            SlotMatch::Deleted => {
                if first_deleted.is_none() {
                    first_deleted = Some(index);
                }
            }
            SlotMatch::Empty => return Ok(Lookup::Vacant(first_deleted.unwrap_or(index))),
        }

        index += 1;
        if index == slots {
            index = 0;
        }
    }

    Ok(first_deleted.map_or(Lookup::Full, Lookup::Vacant))
}

fn hash_bytes(bytes: &[u8]) -> usize {
    let mut hash = 0x811c_9dc5u32;

    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }

    hash as usize
}

/// Slot size used by the specialized actor-id static map.
pub const ACTOR_ID_U256_SLOT_SIZE: usize = 64;

/// Slot size used by the specialized allowance static map.
pub const ALLOWANCE_U256_SLOT_SIZE: usize = 96;

/// Gear page-sized tile used by the page-local actor-id static map.
pub const PAGE_LOCAL_ACTOR_U256_TILE_BYTES: usize = 16 * 1024;

/// Actor/value slots fitting in one page-local actor-id static map tile.
pub const PAGE_LOCAL_ACTOR_U256_SLOTS_PER_TILE: usize = 252;

/// Offset where page-local actor-id static map data slots begin inside a tile.
pub const PAGE_LOCAL_ACTOR_U256_DATA_OFFSET: usize = 256;

/// Slot size used by the experimental owner-local VFT account map.
#[cfg(feature = "experimental-vft-account")]
pub const VFT_ACCOUNT_U256_SLOT_SIZE: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FixedSlot<const KEY_SIZE: usize, const VALUE_SIZE: usize> {
    state: SlotState,
    key: [u8; KEY_SIZE],
    value: [u8; VALUE_SIZE],
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize> FixedSlot<KEY_SIZE, VALUE_SIZE> {
    const EMPTY: Self = Self {
        state: SlotState::Empty,
        key: [0; KEY_SIZE],
        value: [0; VALUE_SIZE],
    };
}

/// A fixed-capacity open-addressed map stored inside the program state.
///
/// This type is useful for tests, benchmarks, and small bounded state. For
/// lazy-page optimized storage, use [`StaticOpenAddressTable`] over a static
/// memory region.
pub struct FixedOpenAddressMap<const KEY_SIZE: usize, const VALUE_SIZE: usize, const CAP: usize> {
    slots: [FixedSlot<KEY_SIZE, VALUE_SIZE>; CAP],
    len: usize,
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize, const CAP: usize>
    FixedOpenAddressMap<KEY_SIZE, VALUE_SIZE, CAP>
{
    /// Creates an empty fixed-capacity map.
    pub const fn new() -> Self {
        Self {
            slots: [FixedSlot::EMPTY; CAP],
            len: 0,
        }
    }

    /// Returns the number of visible entries.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the map contains no visible entries.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the configured capacity.
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Returns the visible value for `key`.
    pub fn get(&self, key: &[u8; KEY_SIZE]) -> Result<Option<[u8; VALUE_SIZE]>, TableError> {
        match self.lookup(key)? {
            Lookup::Found(index) => Ok(Some(self.slots[index].value)),
            Lookup::Vacant(_) | Lookup::Full => Ok(None),
        }
    }

    /// Returns all visible key/value pairs in slot order.
    pub fn entries(&self) -> impl Iterator<Item = ([u8; KEY_SIZE], [u8; VALUE_SIZE])> + '_ {
        self.slots
            .iter()
            .filter_map(|slot| (slot.state == SlotState::Full).then_some((slot.key, slot.value)))
    }

    /// Inserts or updates `key`, returning the previous visible value.
    pub fn insert(
        &mut self,
        key: [u8; KEY_SIZE],
        value: [u8; VALUE_SIZE],
    ) -> Result<Option<[u8; VALUE_SIZE]>, TableError> {
        match self.lookup(&key)? {
            Lookup::Found(index) => {
                let previous = self.slots[index].value;
                self.slots[index] = FixedSlot {
                    state: SlotState::Full,
                    key,
                    value,
                };
                Ok(Some(previous))
            }
            Lookup::Vacant(index) => {
                self.slots[index] = FixedSlot {
                    state: SlotState::Full,
                    key,
                    value,
                };
                self.len += 1;
                Ok(None)
            }
            Lookup::Full => Err(TableError::CapacityOverflow),
        }
    }

    /// Removes the visible value for `key`, preserving the probe chain.
    pub fn remove(&mut self, key: &[u8; KEY_SIZE]) -> Result<Option<[u8; VALUE_SIZE]>, TableError> {
        let Lookup::Found(index) = self.lookup(key)? else {
            return Ok(None);
        };

        let previous = self.slots[index].value;
        self.slots[index].state = SlotState::Deleted;
        self.len -= 1;
        Ok(Some(previous))
    }

    fn lookup(&self, key: &[u8; KEY_SIZE]) -> Result<Lookup, TableError> {
        find_slot(CAP, hash_bytes(key), |index| {
            let slot = &self.slots[index];
            Ok(match slot.state {
                SlotState::Full if slot.key == *key => SlotMatch::Found,
                SlotState::Full => SlotMatch::Occupied,
                SlotState::Deleted => SlotMatch::Deleted,
                SlotState::Empty => SlotMatch::Empty,
            })
        })
    }
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize, const CAP: usize> Default
    for FixedOpenAddressMap<KEY_SIZE, VALUE_SIZE, CAP>
{
    fn default() -> Self {
        Self::new()
    }
}

/// A byte interval reserved for static storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaticRegion {
    base: usize,
    bytes: usize,
}

impl StaticRegion {
    /// Creates a byte region and validates `base + bytes`.
    pub fn new(base: usize, bytes: usize) -> Result<Self, TableError> {
        base.checked_add(bytes).ok_or(TableError::InvalidLayout)?;
        Ok(Self { base, bytes })
    }

    /// Returns the region base address.
    pub const fn base(self) -> usize {
        self.base
    }

    /// Returns the region byte length.
    pub const fn bytes(self) -> usize {
        self.bytes
    }

    /// Returns the first byte after this region.
    pub fn end(self) -> Result<usize, TableError> {
        self.base
            .checked_add(self.bytes)
            .ok_or(TableError::InvalidLayout)
    }
}

/// A typed static-memory region for an open-addressed table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableRegion<const KEY_SIZE: usize, const VALUE_SIZE: usize> {
    region: StaticRegion,
    slots: usize,
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize> TableRegion<KEY_SIZE, VALUE_SIZE> {
    /// Returns the byte region backing the table.
    pub const fn region(self) -> StaticRegion {
        self.region
    }

    /// Returns the table base address.
    pub const fn base(self) -> usize {
        self.region.base()
    }

    /// Returns the configured slot count.
    pub const fn slots(self) -> usize {
        self.slots
    }
}

/// Sequential static-memory layout builder.
pub struct StaticLayout {
    cursor: usize,
    end: usize,
}

impl StaticLayout {
    /// Creates a layout over `[base, base + bytes)`.
    pub fn new(base: usize, bytes: usize) -> Result<Self, TableError> {
        let end = base.checked_add(bytes).ok_or(TableError::InvalidLayout)?;
        Ok(Self { cursor: base, end })
    }

    /// Returns the next free address.
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the first byte after the layout region.
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Returns the number of bytes not yet reserved.
    pub fn remaining(&self) -> usize {
        self.end - self.cursor
    }

    /// Reserves a raw byte region.
    pub fn reserve_bytes(&mut self, bytes: usize) -> Result<StaticRegion, TableError> {
        self.reserve_aligned_bytes(bytes, 1)
    }

    /// Reserves a raw byte region with an aligned base address.
    pub fn reserve_aligned_bytes(
        &mut self,
        bytes: usize,
        align: usize,
    ) -> Result<StaticRegion, TableError> {
        if align == 0 || !align.is_power_of_two() {
            return Err(TableError::InvalidLayout);
        }

        let misalignment = self.cursor & (align - 1);
        let padding = if misalignment == 0 {
            0
        } else {
            align - misalignment
        };
        let base = self
            .cursor
            .checked_add(padding)
            .ok_or(TableError::InvalidLayout)?;
        let next = base.checked_add(bytes).ok_or(TableError::InvalidLayout)?;

        if next > self.end {
            return Err(TableError::InvalidLayout);
        }

        let region = StaticRegion { base, bytes };
        self.cursor = next;
        Ok(region)
    }

    /// Reserves a typed table region.
    pub fn reserve_table<const KEY_SIZE: usize, const VALUE_SIZE: usize>(
        &mut self,
        slots: usize,
    ) -> Result<TableRegion<KEY_SIZE, VALUE_SIZE>, TableError> {
        let bytes = StaticOpenAddressTable::<KEY_SIZE, VALUE_SIZE>::bytes_len(slots)?;
        let region = self.reserve_bytes(bytes)?;
        Ok(TableRegion { region, slots })
    }
}

/// A fixed open-addressed table backed by caller-owned static memory.
pub struct StaticOpenAddressTable<const KEY_SIZE: usize, const VALUE_SIZE: usize> {
    base: usize,
    slots: usize,
    _marker: PhantomData<*mut u8>,
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize> StaticOpenAddressTable<KEY_SIZE, VALUE_SIZE> {
    /// Returns the byte length of one slot.
    pub const fn slot_size() -> usize {
        1 + KEY_SIZE + VALUE_SIZE
    }

    /// Returns the byte length required for `slots`.
    pub fn bytes_len(slots: usize) -> Result<usize, TableError> {
        slots
            .checked_mul(Self::slot_size())
            .ok_or(TableError::InvalidLayout)
    }

    /// Creates a table over `slots * slot_size()` bytes at `base`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the memory interval is valid for reads and writes
    /// for the whole lifetime of the table and does not overlap other mutable
    /// state.
    pub unsafe fn new(base: usize, slots: usize) -> Result<Self, TableError> {
        let bytes = Self::bytes_len(slots)?;
        StaticRegion::new(base, bytes)?;
        Ok(Self {
            base,
            slots,
            _marker: PhantomData,
        })
    }

    /// Creates a table from a typed static-memory region.
    ///
    /// # Safety
    ///
    /// The caller must ensure the memory interval is valid for reads and writes
    /// for the whole lifetime of the table and does not overlap other mutable
    /// state.
    pub unsafe fn from_region(
        region: TableRegion<KEY_SIZE, VALUE_SIZE>,
    ) -> Result<Self, TableError> {
        let expected = Self::bytes_len(region.slots)?;
        if expected > region.region.bytes {
            return Err(TableError::InvalidLayout);
        }

        unsafe { Self::new(region.region.base, region.slots) }
    }

    /// Returns the configured base address.
    pub const fn base(&self) -> usize {
        self.base
    }

    /// Returns the configured slot count.
    pub const fn slots(&self) -> usize {
        self.slots
    }

    /// Returns the total byte length occupied by this table.
    pub fn bytes(&self) -> Result<usize, TableError> {
        Self::bytes_len(self.slots)
    }

    /// Returns the visible value for `key`.
    pub fn get(&self, key: &[u8; KEY_SIZE]) -> Result<Option<[u8; VALUE_SIZE]>, TableError> {
        match self.lookup(key)? {
            Lookup::Found(index) => unsafe { self.read_value(index).map(Some) },
            Lookup::Vacant(_) | Lookup::Full => Ok(None),
        }
    }

    /// Inserts or updates `key`, returning the previous visible value.
    pub fn insert(
        &self,
        key: &[u8; KEY_SIZE],
        value: &[u8; VALUE_SIZE],
    ) -> Result<Option<[u8; VALUE_SIZE]>, TableError> {
        match self.lookup(key)? {
            Lookup::Found(index) => {
                let previous = unsafe { self.read_value(index)? };
                unsafe {
                    self.write_key(index, key);
                    self.write_value(index, value);
                    self.write_state(index, SlotState::Full);
                }
                Ok(Some(previous))
            }
            Lookup::Vacant(index) => {
                unsafe {
                    self.write_key(index, key);
                    self.write_value(index, value);
                    self.write_state(index, SlotState::Full);
                }
                Ok(None)
            }
            Lookup::Full => Err(TableError::CapacityOverflow),
        }
    }

    /// Removes the visible value for `key`, preserving the probe chain.
    pub fn remove(&self, key: &[u8; KEY_SIZE]) -> Result<Option<[u8; VALUE_SIZE]>, TableError> {
        let Lookup::Found(index) = self.lookup(key)? else {
            return Ok(None);
        };

        let previous = unsafe { self.read_value(index)? };
        unsafe {
            self.write_state(index, SlotState::Deleted);
        }
        Ok(Some(previous))
    }

    /// Clears every slot to the empty state.
    pub fn clear(&self) -> Result<(), TableError> {
        let bytes = self.bytes()?;
        unsafe {
            ptr::write_bytes(self.base as *mut u8, 0, bytes);
        }
        Ok(())
    }

    fn lookup(&self, key: &[u8; KEY_SIZE]) -> Result<Lookup, TableError> {
        find_slot(self.slots, hash_bytes(key), |index| {
            let state = unsafe { self.read_state(index)? };
            Ok(match state {
                SlotState::Full if unsafe { self.key_matches(index, key) } => SlotMatch::Found,
                SlotState::Full => SlotMatch::Occupied,
                SlotState::Deleted => SlotMatch::Deleted,
                SlotState::Empty => SlotMatch::Empty,
            })
        })
    }

    unsafe fn read_state(&self, slot: usize) -> Result<SlotState, TableError> {
        let byte = unsafe { ptr::read(self.state_ptr(slot)) };
        SlotState::from_byte(byte)
    }

    unsafe fn key_matches(&self, slot: usize, key: &[u8; KEY_SIZE]) -> bool {
        unsafe { core::slice::from_raw_parts(self.key_ptr(slot), KEY_SIZE) == key.as_slice() }
    }

    unsafe fn read_value(&self, slot: usize) -> Result<[u8; VALUE_SIZE], TableError> {
        let mut bytes = [0u8; VALUE_SIZE];
        unsafe {
            ptr::copy_nonoverlapping(self.value_ptr(slot), bytes.as_mut_ptr(), VALUE_SIZE);
        }
        Ok(bytes)
    }

    unsafe fn write_state(&self, slot: usize, state: SlotState) {
        unsafe {
            ptr::write(self.state_ptr(slot), state as u8);
        }
    }

    unsafe fn write_key(&self, slot: usize, key: &[u8; KEY_SIZE]) {
        unsafe {
            ptr::copy_nonoverlapping(key.as_ptr(), self.key_ptr(slot), KEY_SIZE);
        }
    }

    unsafe fn write_value(&self, slot: usize, value: &[u8; VALUE_SIZE]) {
        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), VALUE_SIZE);
        }
    }

    fn state_ptr(&self, slot: usize) -> *mut u8 {
        self.slot_ptr(slot)
    }

    fn key_ptr(&self, slot: usize) -> *mut u8 {
        unsafe { self.slot_ptr(slot).add(1) }
    }

    fn value_ptr(&self, slot: usize) -> *mut u8 {
        unsafe { self.slot_ptr(slot).add(1 + KEY_SIZE) }
    }

    fn slot_ptr(&self, slot: usize) -> *mut u8 {
        (self.base + slot * Self::slot_size()) as *mut u8
    }
}

pub mod gear {
    use super::{
        ACTOR_ID_U256_SLOT_SIZE, ALLOWANCE_U256_SLOT_SIZE, FixedOpenAddressMap, Lookup,
        PAGE_LOCAL_ACTOR_U256_DATA_OFFSET, PAGE_LOCAL_ACTOR_U256_SLOTS_PER_TILE,
        PAGE_LOCAL_ACTOR_U256_TILE_BYTES, SlotState, StaticOpenAddressTable, StaticRegion,
        TableError,
    };
    #[cfg(feature = "experimental-vft-account")]
    use super::VFT_ACCOUNT_U256_SLOT_SIZE;
    use core::{marker::PhantomData, ptr};
    use gprimitives::{ActorId, U256};

    /// Fixed balance map keyed by `ActorId` and storing `U256`.
    pub type FixedBalanceMap<const CAP: usize> = FixedOpenAddressMap<32, 32, CAP>;
    /// Fixed allowance map keyed by `(owner, spender)` and storing `U256`.
    pub type FixedAllowanceMap<const CAP: usize> = FixedOpenAddressMap<64, 32, CAP>;
    /// Static balance table keyed by `ActorId` and storing `U256`.
    pub type StaticBalanceTable = StaticOpenAddressTable<32, 32>;
    /// Static allowance table keyed by `(owner, spender)` and storing `U256`.
    pub type StaticAllowanceTable = StaticOpenAddressTable<64, 32>;
    /// WAT-shaped static balance map using the current golden-ratio actor hash.
    pub type StaticActorIdU256Map<const LOG2_SLOTS: u8> =
        StaticActorIdU256MapWithHash<LOG2_SLOTS, GoldenActorHash>;
    /// WAT-shaped static balance map using an avalanche-mixed actor hash.
    pub type StaticMixedActorIdU256Map<const LOG2_SLOTS: u8> =
        StaticActorIdU256MapWithHash<LOG2_SLOTS, MixedActorHash>;
    /// Experimental static balance map keyed by a compact 64-bit actor tag.
    pub type StaticActorTagU256Map = StaticActorTagU256Table;
    /// Experimental static balance map keyed by a compact 64-bit actor tag and storing `u64`.
    pub type StaticActorTagU64Map = StaticActorTagU64Table;
    /// WAT-shaped static VFT balance map.
    pub type VftBalances<const LOG2_SLOTS: u8> = StaticActorIdU256Map<LOG2_SLOTS>;
    /// WAT-shaped static VFT allowance map.
    pub type VftAllowances<const LOG2_SLOTS: u8> = StaticAllowanceU256Map<LOG2_SLOTS>;
    /// Experimental owner-local static VFT account map.
    #[cfg(feature = "experimental-vft-account")]
    pub type StaticVftAccountMap<const LOG2_SLOTS: u8> = StaticVftAccountU256Map<LOG2_SLOTS>;

    /// Result returned by an optimized actor balance transfer.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ActorU256Transfer {
        pub from_balance: U256,
        pub to_balance: U256,
    }

    /// Result returned by an optimized actor transfer-from operation.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ActorU256TransferFrom {
        pub from_balance: U256,
        pub to_balance: U256,
        pub allowance: U256,
    }

    /// Result returned by a static-table actor balance transfer.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct StaticActorU256Transfer {
        pub from_balance: U256,
        pub to_balance: U256,
        pub inserted_to: bool,
    }

    /// Result returned by a static-table actor transfer-from operation.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct StaticActorU256TransferFrom {
        pub from_balance: U256,
        pub to_balance: U256,
        pub allowance: U256,
        pub inserted_to: bool,
    }

    /// Hash strategy used by specialized actor-id static maps.
    pub trait ActorKeyHash {
        /// Returns a 32-bit hash for a raw actor-id key.
        fn hash(key: &[u8; 32]) -> u32;
    }

    /// Current WAT-shaped actor-id hash.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct GoldenActorHash;

    impl ActorKeyHash for GoldenActorHash {
        fn hash(key: &[u8; 32]) -> u32 {
            hash_words(key)
        }
    }

    /// Avalanche-mixed actor-id hash candidate for high-cardinality VFT maps.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct MixedActorHash;

    impl ActorKeyHash for MixedActorHash {
        fn hash(key: &[u8; 32]) -> u32 {
            fmix32(fold_words(key))
        }
    }

    /// Converts an actor id into a raw 32-byte key.
    pub fn actor_key(actor: ActorId) -> [u8; 32] {
        let mut key = [0u8; 32];
        key.copy_from_slice(actor.as_ref());
        key
    }

    /// Converts an allowance pair into a raw 64-byte key.
    pub fn allowance_key(owner: ActorId, spender: ActorId) -> [u8; 64] {
        let mut key = [0u8; 64];
        key[..32].copy_from_slice(owner.as_ref());
        key[32..].copy_from_slice(spender.as_ref());
        key
    }

    /// Converts a `U256` into the little-endian value bytes used by the helpers.
    pub fn u256_value(value: U256) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        value.to_little_endian(&mut bytes);
        bytes
    }

    /// Converts little-endian value bytes into a `U256`.
    pub fn u256_from_value(bytes: [u8; 32]) -> U256 {
        U256::from_little_endian(&bytes)
    }

    /// A WAT-shaped static balance map keyed by `ActorId` and storing `U256`.
    ///
    /// Each slot is exactly 64 bytes: 32 bytes of key followed by 32 bytes of
    /// value. A zero key terminates probing, and a zero value is treated as an
    /// absent/tombstoned entry for an already-written key.
    pub struct StaticActorIdU256MapWithHash<const LOG2_SLOTS: u8, H: ActorKeyHash> {
        base: usize,
        slots: usize,
        mask: usize,
        _marker: PhantomData<(*mut u8, H)>,
    }

    impl<const LOG2_SLOTS: u8, H: ActorKeyHash> StaticActorIdU256MapWithHash<LOG2_SLOTS, H> {
        const VALUE_OFFSET: usize = 32;

        /// Returns the byte length of one slot.
        pub const fn slot_size() -> usize {
            ACTOR_ID_U256_SLOT_SIZE
        }

        /// Returns the configured slot count.
        pub fn slots() -> Result<usize, TableError> {
            static_map_slots(LOG2_SLOTS)
        }

        /// Returns the mask used for power-of-two probing.
        pub fn mask() -> Result<usize, TableError> {
            Ok(Self::slots()? - 1)
        }

        /// Returns the byte length required for this table.
        pub fn bytes_len() -> Result<usize, TableError> {
            Self::slots()?
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates a table over `2^LOG2_SLOTS * 64` bytes at `base`.
        ///
        /// # Safety
        ///
        /// The caller must ensure the memory interval is valid for reads and
        /// writes for the whole lifetime of the table and does not overlap
        /// other mutable state.
        pub unsafe fn new(base: usize) -> Result<Self, TableError> {
            let slots = Self::slots()?;
            let bytes = slots
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)?;
            StaticRegion::new(base, bytes)?;
            Ok(Self {
                base,
                slots,
                mask: slots - 1,
                _marker: PhantomData,
            })
        }

        /// Returns the configured base address.
        pub const fn base(&self) -> usize {
            self.base
        }

        /// Returns the total byte length occupied by this table.
        pub fn bytes(&self) -> Result<usize, TableError> {
            Self::bytes_len()
        }

        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            if is_zero_32(&key) {
                return Ok(None);
            }

            let LookupResult::Found(index) = self.lookup(&key)? else {
                return Ok(None);
            };
            Ok(self.visible_value(index).map(u256_from_value))
        }

        /// Inserts or updates a balance value by actor id.
        ///
        /// A zero value removes the visible value without writing a new absent
        /// key. Zero actor ids are rejected because zero key bytes mark empty
        /// slots in this layout.
        pub fn insert_actor_u256(
            &self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let key = actor_key(key);
            reject_zero_key(&key)?;
            if value.is_zero() {
                return self
                    .remove_key(&key)
                    .map(|previous| previous.map(u256_from_value));
            }

            let value = u256_value(value);
            match self.lookup(&key)? {
                LookupResult::Found(index) => {
                    let previous = self.visible_value(index);
                    unsafe {
                        self.write_value(index, &value);
                    }
                    Ok(previous.map(u256_from_value))
                }
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_key(index, &key);
                        self.write_value(index, &value);
                    }
                    Ok(None)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Removes a balance value by actor id.
        pub fn remove_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            reject_zero_key(&key)?;
            self.remove_key(&key)
                .map(|previous| previous.map(u256_from_value))
        }

        /// Moves `amount` from one actor balance to another using one lookup per key.
        ///
        /// Returns `Ok(None)` without modifying storage when the sender has no
        /// visible balance, has insufficient balance, the recipient insertion
        /// would overflow `U256`, or the table has no vacant slot for a new
        /// recipient.
        pub fn transfer_actor_u256(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<ActorU256Transfer>, TableError> {
            let from_key = actor_key(from);
            let to_key = actor_key(to);
            reject_zero_key(&from_key)?;
            reject_zero_key(&to_key)?;
            if amount.is_zero() {
                let from_balance = self.get_actor_u256(&from)?.unwrap_or_else(U256::zero);
                let to_balance = if from_key == to_key {
                    from_balance
                } else {
                    self.get_actor_u256(&to)?.unwrap_or_else(U256::zero)
                };
                return Ok(Some(ActorU256Transfer {
                    from_balance,
                    to_balance,
                }));
            }
            let LookupResult::Found(from_index) = self.lookup(&from_key)? else {
                return Ok(None);
            };
            let Some(from_value) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(None);
            };
            if from_value < amount {
                return Ok(None);
            }
            if from_key == to_key {
                return Ok(Some(ActorU256Transfer {
                    from_balance: from_value,
                    to_balance: from_value,
                }));
            }

            let to_lookup = self.lookup(&to_key)?;
            let to_value = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(None),
            };
            let (to_value, overflow) = to_value.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let from_value = from_value - amount;
            let from_value_bytes = u256_value(from_value);
            let to_value_bytes = u256_value(to_value);
            unsafe {
                self.write_value(from_index, &from_value_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_value_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_value_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(Some(ActorU256Transfer {
                from_balance: from_value,
                to_balance: to_value,
            }))
        }

        /// Moves `amount` after validating a spender allowance.
        ///
        /// This validates allowance, sender balance, recipient overflow, and
        /// recipient capacity before writing any value.
        pub fn transfer_actor_u256_from<const ALLOWANCE_LOG2_SLOTS: u8>(
            &self,
            allowances: &StaticAllowanceU256Map<ALLOWANCE_LOG2_SLOTS>,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<ActorU256TransferFrom>, TableError> {
            let owner_key = actor_key(owner);
            let spender_key = actor_key(spender);
            let to_key = actor_key(to);
            reject_zero_key(&owner_key)?;
            reject_zero_key(&spender_key)?;
            reject_zero_key(&to_key)?;

            if amount.is_zero() {
                let from_balance = self.get_actor_u256(&owner)?.unwrap_or_else(U256::zero);
                let to_balance = if owner_key == to_key {
                    from_balance
                } else {
                    self.get_actor_u256(&to)?.unwrap_or_else(U256::zero)
                };
                let allowance = allowances
                    .get_allowance_u256(&owner, &spender)?
                    .unwrap_or_else(U256::zero);
                return Ok(Some(ActorU256TransferFrom {
                    from_balance,
                    to_balance,
                    allowance,
                }));
            }
            let LookupResult::Found(allowance_index) =
                allowances.lookup(&owner_key, &spender_key)?
            else {
                return Ok(None);
            };
            let Some(allowance) = allowances
                .visible_value(allowance_index)
                .map(u256_from_value)
            else {
                return Ok(None);
            };
            if allowance < amount {
                return Ok(None);
            }

            let LookupResult::Found(from_index) = self.lookup(&owner_key)? else {
                return Ok(None);
            };
            let Some(from_balance) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(None);
            };
            if from_balance < amount {
                return Ok(None);
            }

            let allowance = allowance - amount;
            if owner_key == to_key {
                let allowance_bytes = u256_value(allowance);
                unsafe {
                    allowances.write_value(allowance_index, &allowance_bytes);
                }
                return Ok(Some(ActorU256TransferFrom {
                    from_balance,
                    to_balance: from_balance,
                    allowance,
                }));
            }

            let to_lookup = self.lookup(&to_key)?;
            let to_balance = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(None),
            };
            let (to_balance, overflow) = to_balance.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let from_balance = from_balance - amount;
            let from_balance_bytes = u256_value(from_balance);
            let to_balance_bytes = u256_value(to_balance);
            let allowance_bytes = u256_value(allowance);
            unsafe {
                self.write_value(from_index, &from_balance_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_balance_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_balance_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
                allowances.write_value(allowance_index, &allowance_bytes);
            }

            Ok(Some(ActorU256TransferFrom {
                from_balance,
                to_balance,
                allowance,
            }))
        }

        /// Moves a known nonzero amount between known distinct, nonzero actors.
        ///
        /// # Safety
        ///
        /// The caller must ensure `from` and `to` are nonzero actor ids, `from != to`,
        /// and `amount != 0`. This skips validation intended for the safe public path.
        pub unsafe fn transfer_actor_u256_nonzero_distinct_unchecked(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<ActorU256Transfer>, TableError> {
            let from_key = actor_key(from);
            let to_key = actor_key(to);

            let LookupResult::Found(from_index) = self.lookup(&from_key)? else {
                return Ok(None);
            };
            let Some(from_value) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(None);
            };
            if from_value < amount {
                return Ok(None);
            }

            let to_lookup = self.lookup(&to_key)?;
            let to_value = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(None),
            };
            let (to_value, overflow) = to_value.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let from_value = from_value - amount;
            let from_value_bytes = u256_value(from_value);
            let to_value_bytes = u256_value(to_value);
            unsafe {
                self.write_value(from_index, &from_value_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_value_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_value_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(Some(ActorU256Transfer {
                from_balance: from_value,
                to_balance: to_value,
            }))
        }

        /// Moves a known nonzero amount after checking allowance for known distinct, nonzero actors.
        ///
        /// # Safety
        ///
        /// The caller must ensure `owner`, `spender`, and `to` are nonzero actor ids,
        /// `owner != to`, and `amount != 0`. This skips validation intended for the
        /// safe public path.
        pub unsafe fn transfer_actor_u256_from_nonzero_distinct_unchecked<
            const ALLOWANCE_LOG2_SLOTS: u8,
        >(
            &self,
            allowances: &StaticAllowanceU256Map<ALLOWANCE_LOG2_SLOTS>,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<ActorU256TransferFrom>, TableError> {
            let owner_key = actor_key(owner);
            let spender_key = actor_key(spender);
            let to_key = actor_key(to);

            let LookupResult::Found(allowance_index) =
                allowances.lookup(&owner_key, &spender_key)?
            else {
                return Ok(None);
            };
            let Some(allowance) = allowances.visible_value(allowance_index).map(u256_from_value)
            else {
                return Ok(None);
            };
            if allowance < amount {
                return Ok(None);
            }

            let LookupResult::Found(from_index) = self.lookup(&owner_key)? else {
                return Ok(None);
            };
            let Some(from_balance) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(None);
            };
            if from_balance < amount {
                return Ok(None);
            }

            let to_lookup = self.lookup(&to_key)?;
            let to_balance = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(None),
            };
            let (to_balance, overflow) = to_balance.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let allowance = allowance - amount;
            let from_balance = from_balance - amount;
            let allowance_bytes = u256_value(allowance);
            let from_balance_bytes = u256_value(from_balance);
            let to_balance_bytes = u256_value(to_balance);
            unsafe {
                allowances.write_value(allowance_index, &allowance_bytes);
                self.write_value(from_index, &from_balance_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_balance_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_balance_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(Some(ActorU256TransferFrom {
                from_balance,
                to_balance,
                allowance,
            }))
        }

        /// Moves a known nonzero amount and returns only whether it succeeded.
        ///
        /// # Safety
        ///
        /// The caller must ensure `from` and `to` are nonzero actor ids, `from != to`,
        /// and `amount != 0`. This skips validation intended for the safe public path.
        pub unsafe fn transfer_actor_u256_bool_nonzero_distinct_unchecked(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let from_key = actor_key(from);
            let to_key = actor_key(to);

            let LookupResult::Found(from_index) = self.lookup(&from_key)? else {
                return Ok(false);
            };
            let Some(from_value) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(false);
            };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(&to_key)?;
            let to_value = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(false),
            };
            let (to_value, overflow) = to_value.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }

            let from_value_bytes = u256_value(from_value - amount);
            let to_value_bytes = u256_value(to_value);
            unsafe {
                self.write_value(from_index, &from_value_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_value_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_value_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Moves a known nonzero amount after checking allowance and returns only success.
        ///
        /// # Safety
        ///
        /// The caller must ensure `owner`, `spender`, and `to` are nonzero actor ids,
        /// `owner != to`, and `amount != 0`. This skips validation intended for the
        /// safe public path.
        pub unsafe fn transfer_actor_u256_from_bool_nonzero_distinct_unchecked<
            const ALLOWANCE_LOG2_SLOTS: u8,
        >(
            &self,
            allowances: &StaticAllowanceU256Map<ALLOWANCE_LOG2_SLOTS>,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let owner_key = actor_key(owner);
            let spender_key = actor_key(spender);
            let to_key = actor_key(to);

            let LookupResult::Found(allowance_index) =
                allowances.lookup(&owner_key, &spender_key)?
            else {
                return Ok(false);
            };
            let Some(allowance) = allowances.visible_value(allowance_index).map(u256_from_value)
            else {
                return Ok(false);
            };
            if allowance < amount {
                return Ok(false);
            }

            let LookupResult::Found(from_index) = self.lookup(&owner_key)? else {
                return Ok(false);
            };
            let Some(from_balance) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(false);
            };
            if from_balance < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(&to_key)?;
            let to_balance = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(false),
            };
            let (to_balance, overflow) = to_balance.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }

            let allowance_bytes = u256_value(allowance - amount);
            let from_balance_bytes = u256_value(from_balance - amount);
            let to_balance_bytes = u256_value(to_balance);
            unsafe {
                allowances.write_value(allowance_index, &allowance_bytes);
                self.write_value(from_index, &from_balance_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_balance_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_balance_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Clears every slot to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            let bytes = self.bytes()?;
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, bytes);
            }
            Ok(())
        }

        fn remove_key(&self, key: &[u8; 32]) -> Result<Option<[u8; 32]>, TableError> {
            let LookupResult::Found(index) = self.lookup(key)? else {
                return Ok(None);
            };

            let previous = self.visible_value(index);
            if previous.is_some() {
                unsafe {
                    self.write_value(index, &[0; 32]);
                }
            }
            Ok(previous)
        }

        fn lookup(&self, key: &[u8; 32]) -> Result<LookupResult, TableError> {
            let key_words = words_32(key);
            let mut index = static_map_index(H::hash(key), LOG2_SLOTS);

            for _ in 0..self.slots {
                let stored_key = unsafe { read_words_32(self.key_ptr(index)) };
                if stored_key == key_words {
                    return Ok(LookupResult::Found(index));
                }
                if words_are_zero(stored_key) {
                    return Ok(LookupResult::Vacant(index));
                }

                index = (index + 1) & self.mask;
            }

            Ok(LookupResult::Full)
        }

        fn visible_value(&self, slot: usize) -> Option<[u8; 32]> {
            let value = unsafe { self.read_value(slot) };
            if is_zero_32(&value) {
                None
            } else {
                Some(value)
            }
        }

        unsafe fn read_value(&self, slot: usize) -> [u8; 32] {
            let mut bytes = [0u8; 32];
            unsafe {
                ptr::copy_nonoverlapping(self.value_ptr(slot), bytes.as_mut_ptr(), 32);
            }
            bytes
        }

        unsafe fn write_key(&self, slot: usize, key: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(key.as_ptr(), self.key_ptr(slot), 32);
            }
        }

        unsafe fn write_value(&self, slot: usize, value: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), 32);
            }
        }

        fn key_ptr(&self, slot: usize) -> *mut u8 {
            self.slot_ptr(slot)
        }

        fn value_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::VALUE_OFFSET) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            (self.base + slot * Self::slot_size()) as *mut u8
        }
    }

    /// Experimental static balance map keyed by a compact 64-bit actor fingerprint.
    ///
    /// Each slot is exactly 40 bytes: an 8-byte nonzero actor tag followed by a
    /// 32-byte `U256` value. This is a benchmark architecture candidate: it
    /// reduces slot size and key-compare work, but it does not store the full
    /// `ActorId`, so a production version needs an explicit collision policy.
    pub struct StaticActorTagU256Table {
        base: usize,
        slots: usize,
        mask: usize,
        _marker: PhantomData<*mut u8>,
    }

    impl StaticActorTagU256Table {
        const TAG_SIZE: usize = 8;
        const VALUE_OFFSET: usize = 8;

        /// Returns the byte length of one slot.
        pub const fn slot_size() -> usize {
            40
        }

        /// Returns the byte length required for `slots`.
        pub fn bytes_len(slots: usize) -> Result<usize, TableError> {
            slots
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates a table over `slots * 40` bytes at `base`.
        ///
        /// # Safety
        ///
        /// The caller must ensure the memory interval is valid for reads and
        /// writes for the whole lifetime of the table and does not overlap
        /// other mutable state.
        pub unsafe fn new(base: usize, slots: usize) -> Result<Self, TableError> {
            if slots == 0 || !slots.is_power_of_two() {
                return Err(TableError::InvalidLayout);
            }
            StaticRegion::new(base, Self::bytes_len(slots)?)?;
            Ok(Self {
                base,
                slots,
                mask: slots - 1,
                _marker: PhantomData,
            })
        }

        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let tag = actor_tag(*key);
            let LookupResult::Found(index) = self.lookup(tag)? else {
                return Ok(None);
            };
            Ok(self.visible_value(index).map(u256_from_value))
        }

        /// Inserts or updates a balance value by actor id.
        pub fn insert_actor_u256(
            &self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let tag = actor_tag(key);
            let value = u256_value(value);
            if is_zero_32(&value) {
                return self
                    .remove_tag(tag)
                    .map(|previous| previous.map(u256_from_value));
            }

            match self.lookup(tag)? {
                LookupResult::Found(index) => {
                    let previous = self.visible_value(index);
                    unsafe {
                        self.write_value(index, &value);
                    }
                    Ok(previous.map(u256_from_value))
                }
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_tag(index, tag);
                        self.write_value(index, &value);
                    }
                    Ok(None)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Removes a balance value by actor id.
        pub fn remove_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            self.remove_tag(actor_tag(*key))
                .map(|previous| previous.map(u256_from_value))
        }

        /// Returns an allowance value by owner and spender tags.
        pub fn get_allowance_u256(
            &self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            let tag = allowance_tag(*owner, *spender);
            let LookupResult::Found(index) = self.lookup(tag)? else {
                return Ok(None);
            };
            Ok(self.visible_value(index).map(u256_from_value))
        }

        /// Inserts or updates an allowance value by owner and spender tags.
        pub fn insert_allowance_u256(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let tag = allowance_tag(owner, spender);
            let value = u256_value(value);
            if is_zero_32(&value) {
                return self
                    .remove_tag(tag)
                    .map(|previous| previous.map(u256_from_value));
            }

            match self.lookup(tag)? {
                LookupResult::Found(index) => {
                    let previous = self.visible_value(index);
                    unsafe {
                        self.write_value(index, &value);
                    }
                    Ok(previous.map(u256_from_value))
                }
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_tag(index, tag);
                        self.write_value(index, &value);
                    }
                    Ok(None)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Inserts or updates an allowance and returns only whether it was written.
        pub fn insert_allowance_u256_bool(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<bool, TableError> {
            if value.is_zero() {
                return Ok(false);
            }

            let tag = allowance_tag(owner, spender);
            let value = u256_value(value);
            match self.lookup(tag)? {
                LookupResult::Found(index) => unsafe {
                    self.write_value(index, &value);
                    Ok(true)
                },
                LookupResult::Vacant(index) => unsafe {
                    self.write_tag(index, tag);
                    self.write_value(index, &value);
                    Ok(true)
                },
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Experimental low-limb allowance insert for benchmark values.
        ///
        /// # Safety
        ///
        /// The caller must ensure `value` fits in `u64` and the target slot has
        /// zeroed high limbs when inserting a fresh key.
        pub unsafe fn insert_allowance_u256_low64_bool_unchecked(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<bool, TableError> {
            let tag = allowance_tag(owner, spender);
            let value = value.low_u64();
            match self.lookup(tag)? {
                LookupResult::Found(index) => unsafe {
                    self.write_value_low64(index, value);
                    Ok(true)
                },
                LookupResult::Vacant(index) => unsafe {
                    self.write_tag(index, tag);
                    self.write_value_low64(index, value);
                    Ok(true)
                },
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Moves a nonzero amount between distinct actors and returns only success.
        pub fn transfer_actor_u256_bool(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let from_tag = actor_tag(from);
            let to_tag = actor_tag(to);
            if from_tag == to_tag || amount.is_zero() {
                return Ok(false);
            }

            let LookupResult::Found(from_index) = self.lookup(from_tag)? else {
                return Ok(false);
            };
            let Some(from_value) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(false);
            };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag)?;
            let to_value = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(false),
            };
            let (to_value, overflow) = to_value.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }

            let from_value_bytes = u256_value(from_value - amount);
            let to_value_bytes = u256_value(to_value);
            unsafe {
                self.write_value(from_index, &from_value_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_value_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, &to_value_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Moves a nonzero amount after checking allowance and returns only success.
        pub fn transfer_actor_u256_from_bool<const ALLOWANCE_LOG2_SLOTS: u8>(
            &self,
            allowances: &StaticAllowanceU256Map<ALLOWANCE_LOG2_SLOTS>,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let owner_key = actor_key(owner);
            let spender_key = actor_key(spender);
            let LookupResult::Found(allowance_index) =
                allowances.lookup(&owner_key, &spender_key)?
            else {
                return Ok(false);
            };
            let Some(allowance) = allowances.visible_value(allowance_index).map(u256_from_value)
            else {
                return Ok(false);
            };
            if allowance < amount {
                return Ok(false);
            }

            let transferred = self.transfer_actor_u256_bool(owner, to, amount)?;
            if !transferred {
                return Ok(false);
            }

            let allowance_bytes = u256_value(allowance - amount);
            unsafe {
                allowances.write_value(allowance_index, &allowance_bytes);
            }
            Ok(true)
        }

        /// Moves a nonzero amount after checking a compact-tag allowance table.
        pub fn transfer_actor_u256_from_tag_bool(
            &self,
            allowances: &StaticActorTagU256Table,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let allowance_tag = allowance_tag(owner, spender);
            let LookupResult::Found(allowance_index) = allowances.lookup(allowance_tag)? else {
                return Ok(false);
            };
            let Some(allowance) = allowances.visible_value(allowance_index).map(u256_from_value)
            else {
                return Ok(false);
            };
            if allowance < amount {
                return Ok(false);
            }

            let from_tag = actor_tag(owner);
            let to_tag = actor_tag(to);
            if from_tag == to_tag || amount.is_zero() {
                return Ok(false);
            }

            let LookupResult::Found(from_index) = self.lookup(from_tag)? else {
                return Ok(false);
            };
            let Some(from_value) = self.visible_value(from_index).map(u256_from_value) else {
                return Ok(false);
            };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag)?;
            let to_value = match to_lookup {
                LookupResult::Found(index) => self
                    .visible_value(index)
                    .map_or_else(U256::zero, u256_from_value),
                LookupResult::Vacant(_) => U256::zero(),
                LookupResult::Full => return Ok(false),
            };
            let (to_value, overflow) = to_value.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }

            let allowance_bytes = u256_value(allowance - amount);
            let from_value_bytes = u256_value(from_value - amount);
            let to_value_bytes = u256_value(to_value);
            unsafe {
                allowances.write_value(allowance_index, &allowance_bytes);
                self.write_value(from_index, &from_value_bytes);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, &to_value_bytes),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, &to_value_bytes);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Experimental low-limb transfer fast path for benchmark values.
        ///
        /// # Safety
        ///
        /// The caller must ensure balances and `amount` fit in `u64` and high
        /// limbs are zero. This is not a general `U256` transfer implementation.
        pub unsafe fn transfer_actor_u256_low64_bool_unchecked(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let amount = amount.low_u64();
            let from_tag = actor_tag(from);
            let to_tag = actor_tag(to);
            let LookupResult::Found(from_index) = self.lookup(from_tag)? else {
                return Ok(false);
            };
            let from_value = unsafe { self.read_value_low64(from_index) };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag)?;
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value_low64(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => return Ok(false),
            };
            let Some(to_value) = to_value.checked_add(amount) else {
                return Ok(false);
            };

            unsafe {
                self.write_value_low64(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value_low64(index, to_value),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value_low64(index, to_value);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Experimental low-limb transfer-from fast path for benchmark values.
        ///
        /// # Safety
        ///
        /// The caller must ensure balances, allowances, and `amount` fit in `u64`
        /// and high limbs are zero. This is not a general `U256` implementation.
        pub unsafe fn transfer_actor_u256_from_tag_low64_bool_unchecked(
            &self,
            allowances: &StaticActorTagU256Table,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let amount = amount.low_u64();
            let allowance_tag = allowance_tag(owner, spender);
            let LookupResult::Found(allowance_index) = allowances.lookup(allowance_tag)? else {
                return Ok(false);
            };
            let allowance = unsafe { allowances.read_value_low64(allowance_index) };
            if allowance < amount {
                return Ok(false);
            }

            let from_tag = actor_tag(owner);
            let to_tag = actor_tag(to);
            let LookupResult::Found(from_index) = self.lookup(from_tag)? else {
                return Ok(false);
            };
            let from_value = unsafe { self.read_value_low64(from_index) };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag)?;
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value_low64(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => return Ok(false),
            };
            let Some(to_value) = to_value.checked_add(amount) else {
                return Ok(false);
            };

            unsafe {
                allowances.write_value_low64(allowance_index, allowance - amount);
                self.write_value_low64(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value_low64(index, to_value),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value_low64(index, to_value);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Clears every slot to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, Self::bytes_len(self.slots)?);
            }
            Ok(())
        }

        fn remove_tag(&self, tag: u64) -> Result<Option<[u8; 32]>, TableError> {
            let LookupResult::Found(index) = self.lookup(tag)? else {
                return Ok(None);
            };
            let previous = self.visible_value(index);
            if previous.is_some() {
                unsafe {
                    self.write_value(index, &[0; 32]);
                }
            }
            Ok(previous)
        }

        fn lookup(&self, tag: u64) -> Result<LookupResult, TableError> {
            let mut index = (tag as usize) & self.mask;
            for _ in 0..self.slots {
                let stored = unsafe { self.read_tag(index) };
                if stored == tag {
                    return Ok(LookupResult::Found(index));
                }
                if stored == 0 {
                    return Ok(LookupResult::Vacant(index));
                }
                index += 1;
                if index == self.slots {
                    index = 0;
                }
            }

            Ok(LookupResult::Full)
        }

        fn visible_value(&self, slot: usize) -> Option<[u8; 32]> {
            let value = unsafe { self.read_value(slot) };
            if is_zero_32(&value) {
                None
            } else {
                Some(value)
            }
        }

        unsafe fn read_tag(&self, slot: usize) -> u64 {
            let mut bytes = [0u8; Self::TAG_SIZE];
            unsafe {
                ptr::copy_nonoverlapping(self.tag_ptr(slot), bytes.as_mut_ptr(), Self::TAG_SIZE);
            }
            u64::from_le_bytes(bytes)
        }

        unsafe fn read_value(&self, slot: usize) -> [u8; 32] {
            let mut bytes = [0u8; 32];
            unsafe {
                ptr::copy_nonoverlapping(self.value_ptr(slot), bytes.as_mut_ptr(), 32);
            }
            bytes
        }

        unsafe fn read_value_low64(&self, slot: usize) -> u64 {
            unsafe { ptr::read_unaligned(self.value_ptr(slot).cast::<u64>()) }
        }

        unsafe fn write_tag(&self, slot: usize, tag: u64) {
            let bytes = tag.to_le_bytes();
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.tag_ptr(slot), Self::TAG_SIZE);
            }
        }

        unsafe fn write_value(&self, slot: usize, value: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), 32);
            }
        }

        unsafe fn write_value_low64(&self, slot: usize, value: u64) {
            unsafe {
                ptr::write_unaligned(self.value_ptr(slot).cast::<u64>(), value);
            }
        }

        fn tag_ptr(&self, slot: usize) -> *mut u8 {
            self.slot_ptr(slot)
        }

        fn value_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::VALUE_OFFSET) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            (self.base + slot * Self::slot_size()) as *mut u8
        }
    }

    /// Experimental compact static map keyed by a 64-bit actor tag and storing `u64`.
    ///
    /// Each slot is 16 bytes: 8 bytes of nonzero tag and 8 bytes of value. This
    /// is only a benchmark architecture candidate for low-limb VFT values.
    pub struct StaticActorTagU64Table {
        base: usize,
        slots: usize,
        mask: usize,
        _marker: PhantomData<*mut u8>,
    }

    impl StaticActorTagU64Table {
        const VALUE_OFFSET: usize = 8;

        /// Returns the byte length of one slot.
        pub const fn slot_size() -> usize {
            16
        }

        /// Returns the byte length required for `slots`.
        pub fn bytes_len(slots: usize) -> Result<usize, TableError> {
            slots
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates a table over `slots * 16` bytes at `base`.
        ///
        /// # Safety
        ///
        /// The caller must ensure the memory interval is valid for reads and
        /// writes for the whole lifetime of the table and does not overlap
        /// other mutable state.
        pub unsafe fn new(base: usize, slots: usize) -> Result<Self, TableError> {
            if slots == 0 || !slots.is_power_of_two() {
                return Err(TableError::InvalidLayout);
            }
            StaticRegion::new(base, Self::bytes_len(slots)?)?;
            Ok(Self {
                base,
                slots,
                mask: slots - 1,
                _marker: PhantomData,
            })
        }

        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let tag = actor_tag(*key);
            let LookupResult::Found(index) = self.lookup(tag) else {
                return Ok(None);
            };
            Ok(Some(U256::from(unsafe { self.read_value(index) })))
        }

        /// Inserts or updates a low-limb balance value by actor id.
        pub fn insert_actor_u256(
            &self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let tag = actor_tag(key);
            let value = value.low_u64();
            match self.lookup(tag) {
                LookupResult::Found(index) => {
                    let previous = unsafe { self.read_value(index) };
                    unsafe {
                        self.write_value(index, value);
                    }
                    Ok(Some(U256::from(previous)))
                }
                LookupResult::Vacant(index) => unsafe {
                    self.write_tag(index, tag);
                    self.write_value(index, value);
                    Ok(None)
                },
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Removes a value by actor id.
        pub fn remove_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let tag = actor_tag(*key);
            let LookupResult::Found(index) = self.lookup(tag) else {
                return Ok(None);
            };
            let previous = unsafe { self.read_value(index) };
            unsafe {
                self.write_value(index, 0);
            }
            Ok(Some(U256::from(previous)))
        }

        /// Inserts or updates a low-limb allowance value and returns success.
        pub fn insert_allowance_u256_bool(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<bool, TableError> {
            let tag = allowance_tag(owner, spender);
            let value = value.low_u64();
            match self.lookup(tag) {
                LookupResult::Found(index) => unsafe {
                    self.write_value(index, value);
                    Ok(true)
                },
                LookupResult::Vacant(index) => unsafe {
                    self.write_tag(index, tag);
                    self.write_value(index, value);
                    Ok(true)
                },
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Inserts or updates a raw tag value and returns success.
        pub fn insert_tag_value_bool(&self, tag: u64, value: u64) -> Result<bool, TableError> {
            let tag = if tag == 0 { 1 } else { tag };
            match self.lookup(tag) {
                LookupResult::Found(index) => unsafe {
                    self.write_value(index, value);
                    Ok(true)
                },
                LookupResult::Vacant(index) => unsafe {
                    self.write_tag(index, tag);
                    self.write_value(index, value);
                    Ok(true)
                },
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Inserts or updates a raw tag value when capacity is known to be available.
        ///
        /// # Safety
        ///
        /// The caller must ensure `tag` is unique to the logical key and the table
        /// has at least one vacant slot if this is a new tag.
        pub unsafe fn insert_tag_value_bool_capacity_unchecked(
            &self,
            tag: u64,
            value: u64,
        ) -> bool {
            let tag = if tag == 0 { 1 } else { tag };
            match unsafe { self.lookup_capacity_unchecked(tag) } {
                LookupResult::Found(index) => unsafe {
                    self.write_value(index, value);
                    true
                },
                LookupResult::Vacant(index) => unsafe {
                    self.write_tag(index, tag);
                    self.write_value(index, value);
                    true
                },
                LookupResult::Full => unsafe { core::hint::unreachable_unchecked() },
            }
        }

        /// Returns an allowance value by owner and spender.
        pub fn get_allowance_u256(
            &self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            let tag = allowance_tag(*owner, *spender);
            let LookupResult::Found(index) = self.lookup(tag) else {
                return Ok(None);
            };
            Ok(Some(U256::from(unsafe { self.read_value(index) })))
        }

        /// Moves a low-limb amount and returns success.
        pub fn transfer_actor_u256_bool(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let amount = amount.low_u64();
            let from_tag = actor_tag(from);
            let to_tag = actor_tag(to);
            let LookupResult::Found(from_index) = self.lookup(from_tag) else {
                return Ok(false);
            };
            let from_value = unsafe { self.read_value(from_index) };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag);
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => return Ok(false),
            };
            let Some(to_value) = to_value.checked_add(amount) else {
                return Ok(false);
            };

            unsafe {
                self.write_value(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, to_value),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, to_value);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Moves a low-limb amount between raw tags and returns success.
        pub fn transfer_tags_bool(
            &self,
            from_tag: u64,
            to_tag: u64,
            amount: u64,
        ) -> Result<bool, TableError> {
            let from_tag = if from_tag == 0 { 1 } else { from_tag };
            let to_tag = if to_tag == 0 { 1 } else { to_tag };
            let LookupResult::Found(from_index) = self.lookup(from_tag) else {
                return Ok(false);
            };
            let from_value = unsafe { self.read_value(from_index) };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag);
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => return Ok(false),
            };
            let Some(to_value) = to_value.checked_add(amount) else {
                return Ok(false);
            };

            unsafe {
                self.write_value(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, to_value),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, to_value);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Moves a low-limb amount between raw tags when capacity is known.
        ///
        /// # Safety
        ///
        /// The caller must ensure nonzero tags identify logical accounts, the table
        /// has capacity for a missing recipient, and recipient overflow is impossible
        /// for the token supply being represented.
        pub unsafe fn transfer_tags_bool_capacity_unchecked(
            &self,
            from_tag: u64,
            to_tag: u64,
            amount: u64,
        ) -> bool {
            let from_tag = if from_tag == 0 { 1 } else { from_tag };
            let to_tag = if to_tag == 0 { 1 } else { to_tag };
            let LookupResult::Found(from_index) =
                (unsafe { self.lookup_capacity_unchecked(from_tag) })
            else {
                return false;
            };
            let from_value = unsafe { self.read_value(from_index) };
            if from_value < amount {
                return false;
            }

            let to_lookup = unsafe { self.lookup_capacity_unchecked(to_tag) };
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => unsafe { core::hint::unreachable_unchecked() },
            };

            unsafe {
                self.write_value(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, to_value + amount),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, amount);
                    }
                    LookupResult::Full => core::hint::unreachable_unchecked(),
                }
            }
            true
        }

        /// Moves a low-limb amount after checking a low-limb allowance.
        pub fn transfer_actor_u256_from_bool(
            &self,
            allowances: &StaticActorTagU64Table,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            let amount = amount.low_u64();
            let allowance_tag = allowance_tag(owner, spender);
            let LookupResult::Found(allowance_index) = allowances.lookup(allowance_tag) else {
                return Ok(false);
            };
            let allowance = unsafe { allowances.read_value(allowance_index) };
            if allowance < amount {
                return Ok(false);
            }

            let from_tag = actor_tag(owner);
            let to_tag = actor_tag(to);
            let LookupResult::Found(from_index) = self.lookup(from_tag) else {
                return Ok(false);
            };
            let from_value = unsafe { self.read_value(from_index) };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag);
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => return Ok(false),
            };
            let Some(to_value) = to_value.checked_add(amount) else {
                return Ok(false);
            };

            unsafe {
                allowances.write_value(allowance_index, allowance - amount);
                self.write_value(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, to_value),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, to_value);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Moves a low-limb amount after checking a raw allowance tag.
        pub fn transfer_from_tags_bool(
            &self,
            allowances: &StaticActorTagU64Table,
            allowance_tag: u64,
            from_tag: u64,
            to_tag: u64,
            amount: u64,
        ) -> Result<bool, TableError> {
            let allowance_tag = if allowance_tag == 0 { 1 } else { allowance_tag };
            let LookupResult::Found(allowance_index) = allowances.lookup(allowance_tag) else {
                return Ok(false);
            };
            let allowance = unsafe { allowances.read_value(allowance_index) };
            if allowance < amount {
                return Ok(false);
            }

            let from_tag = if from_tag == 0 { 1 } else { from_tag };
            let to_tag = if to_tag == 0 { 1 } else { to_tag };
            let LookupResult::Found(from_index) = self.lookup(from_tag) else {
                return Ok(false);
            };
            let from_value = unsafe { self.read_value(from_index) };
            if from_value < amount {
                return Ok(false);
            }

            let to_lookup = self.lookup(to_tag);
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => return Ok(false),
            };
            let Some(to_value) = to_value.checked_add(amount) else {
                return Ok(false);
            };

            unsafe {
                allowances.write_value(allowance_index, allowance - amount);
                self.write_value(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, to_value),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, to_value);
                    }
                    LookupResult::Full => unreachable!("full table handled before writes"),
                }
            }
            Ok(true)
        }

        /// Moves a low-limb amount after checking a raw allowance tag when capacity is known.
        ///
        /// # Safety
        ///
        /// The caller must ensure nonzero tags identify logical accounts, both
        /// tables have capacity for the operation, and recipient overflow is
        /// impossible for the token supply being represented.
        pub unsafe fn transfer_from_tags_bool_capacity_unchecked(
            &self,
            allowances: &StaticActorTagU64Table,
            allowance_tag: u64,
            from_tag: u64,
            to_tag: u64,
            amount: u64,
        ) -> bool {
            let allowance_tag = if allowance_tag == 0 { 1 } else { allowance_tag };
            let LookupResult::Found(allowance_index) =
                (unsafe { allowances.lookup_capacity_unchecked(allowance_tag) })
            else {
                return false;
            };
            let allowance = unsafe { allowances.read_value(allowance_index) };
            if allowance < amount {
                return false;
            }

            let from_tag = if from_tag == 0 { 1 } else { from_tag };
            let to_tag = if to_tag == 0 { 1 } else { to_tag };
            let LookupResult::Found(from_index) =
                (unsafe { self.lookup_capacity_unchecked(from_tag) })
            else {
                return false;
            };
            let from_value = unsafe { self.read_value(from_index) };
            if from_value < amount {
                return false;
            }

            let to_lookup = unsafe { self.lookup_capacity_unchecked(to_tag) };
            let to_value = match to_lookup {
                LookupResult::Found(index) => unsafe { self.read_value(index) },
                LookupResult::Vacant(_) => 0,
                LookupResult::Full => unsafe { core::hint::unreachable_unchecked() },
            };

            unsafe {
                allowances.write_value(allowance_index, allowance - amount);
                self.write_value(from_index, from_value - amount);
                match to_lookup {
                    LookupResult::Found(index) => self.write_value(index, to_value + amount),
                    LookupResult::Vacant(index) => {
                        self.write_tag(index, to_tag);
                        self.write_value(index, amount);
                    }
                    LookupResult::Full => core::hint::unreachable_unchecked(),
                }
            }
            true
        }

        /// Clears every slot to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, Self::bytes_len(self.slots)?);
            }
            Ok(())
        }

        fn lookup(&self, tag: u64) -> LookupResult {
            let mut index = (tag as usize) & self.mask;
            for _ in 0..self.slots {
                let stored = unsafe { self.read_tag(index) };
                if stored == tag {
                    return LookupResult::Found(index);
                }
                if stored == 0 {
                    return LookupResult::Vacant(index);
                }
                index = (index + 1) & self.mask;
            }
            LookupResult::Full
        }

        unsafe fn lookup_capacity_unchecked(&self, tag: u64) -> LookupResult {
            let mut index = (tag as usize) & self.mask;
            loop {
                let stored = unsafe { self.read_tag(index) };
                if stored == tag {
                    return LookupResult::Found(index);
                }
                if stored == 0 {
                    return LookupResult::Vacant(index);
                }
                index = (index + 1) & self.mask;
            }
        }

        unsafe fn read_tag(&self, slot: usize) -> u64 {
            unsafe { ptr::read_unaligned(self.tag_ptr(slot).cast::<u64>()) }
        }

        unsafe fn read_value(&self, slot: usize) -> u64 {
            unsafe { ptr::read_unaligned(self.value_ptr(slot).cast::<u64>()) }
        }

        unsafe fn write_tag(&self, slot: usize, tag: u64) {
            unsafe {
                ptr::write_unaligned(self.tag_ptr(slot).cast::<u64>(), tag);
            }
        }

        unsafe fn write_value(&self, slot: usize, value: u64) {
            unsafe {
                ptr::write_unaligned(self.value_ptr(slot).cast::<u64>(), value);
            }
        }

        fn tag_ptr(&self, slot: usize) -> *mut u8 {
            self.slot_ptr(slot)
        }

        fn value_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::VALUE_OFFSET) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            (self.base + slot * Self::slot_size()) as *mut u8
        }
    }

    /// A static balance map with separate control bytes and 64-byte value slots.
    ///
    /// Each data slot is exactly 64 bytes: 32 bytes of key followed by 32
    /// bytes of value. The separate control region stores empty/deleted state
    /// and a 7-bit hash fingerprint, so most probes can reject a slot without
    /// loading its full 32-byte key.
    pub struct StaticControlActorIdU256Map<const LOG2_SLOTS: u8> {
        control_base: usize,
        slots_base: usize,
        slots: usize,
        mask: usize,
        _marker: PhantomData<*mut u8>,
    }

    impl<const LOG2_SLOTS: u8> StaticControlActorIdU256Map<LOG2_SLOTS> {
        const CONTROL_EMPTY: u8 = 0x00;
        const CONTROL_DELETED: u8 = 0xff;
        const CONTROL_FULL_MAX: u8 = 0x80;
        const VALUE_OFFSET: usize = 32;

        /// Returns the byte length of one data slot.
        pub const fn slot_size() -> usize {
            ACTOR_ID_U256_SLOT_SIZE
        }

        /// Returns the configured slot count.
        pub fn slots() -> Result<usize, TableError> {
            static_map_slots(LOG2_SLOTS)
        }

        /// Returns the mask used for power-of-two probing.
        pub fn mask() -> Result<usize, TableError> {
            Ok(Self::slots()? - 1)
        }

        /// Returns the byte length required for the control region.
        pub fn control_bytes_len() -> Result<usize, TableError> {
            Self::slots()
        }

        /// Returns the byte length required for the data slot region.
        pub fn slots_bytes_len() -> Result<usize, TableError> {
            Self::slots()?
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)
        }

        /// Returns the logical byte length required by both regions.
        pub fn bytes_len() -> Result<usize, TableError> {
            Self::control_bytes_len()?
                .checked_add(Self::slots_bytes_len()?)
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates a control-byte table over separate control and data regions.
        ///
        /// # Safety
        ///
        /// The caller must ensure both memory intervals are valid for reads and
        /// writes for the whole lifetime of the table and do not overlap other
        /// mutable state.
        pub unsafe fn new(control_base: usize, slots_base: usize) -> Result<Self, TableError> {
            let slots = Self::slots()?;
            let control_region = StaticRegion::new(control_base, slots)?;
            let slots_bytes = slots
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)?;
            let slots_region = StaticRegion::new(slots_base, slots_bytes)?;
            if regions_overlap(control_region, slots_region)? {
                return Err(TableError::InvalidLayout);
            }

            Ok(Self {
                control_base,
                slots_base,
                slots,
                mask: slots - 1,
                _marker: PhantomData,
            })
        }

        /// Returns the configured control region base address.
        pub const fn control_base(&self) -> usize {
            self.control_base
        }

        /// Returns the configured data slot region base address.
        pub const fn slots_base(&self) -> usize {
            self.slots_base
        }

        /// Returns the logical byte length occupied by both regions.
        pub fn bytes(&self) -> Result<usize, TableError> {
            Self::bytes_len()
        }

        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            if is_zero_32(&key) {
                return Ok(None);
            }

            let LookupResult::Found(index) = self.lookup(&key)? else {
                return Ok(None);
            };
            Ok(self.visible_value(index).map(u256_from_value))
        }

        /// Inserts or updates a balance value by actor id.
        ///
        /// A zero value removes the visible value without writing a new absent
        /// key. Zero actor ids are rejected because zeroed control memory
        /// already represents empty slots.
        pub fn insert_actor_u256(
            &self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let key = actor_key(key);
            reject_zero_key(&key)?;
            if value.is_zero() {
                return self
                    .remove_key(&key)
                    .map(|previous| previous.map(u256_from_value));
            }

            let value = u256_value(value);
            let hash = hash_actor_key(&key);
            match self.lookup_with_hash(&key, hash)? {
                LookupResult::Found(index) => {
                    let previous = self.visible_value(index);
                    unsafe {
                        self.write_value(index, &value);
                    }
                    Ok(previous.map(u256_from_value))
                }
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_key(index, &key);
                        self.write_value(index, &value);
                        self.write_control(index, control_fingerprint(hash));
                    }
                    Ok(None)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Removes a balance value by actor id.
        pub fn remove_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            reject_zero_key(&key)?;
            self.remove_key(&key)
                .map(|previous| previous.map(u256_from_value))
        }

        /// Clears every slot to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            unsafe {
                ptr::write_bytes(self.control_base as *mut u8, 0, Self::control_bytes_len()?);
                ptr::write_bytes(self.slots_base as *mut u8, 0, Self::slots_bytes_len()?);
            }
            Ok(())
        }

        fn remove_key(&self, key: &[u8; 32]) -> Result<Option<[u8; 32]>, TableError> {
            let LookupResult::Found(index) = self.lookup(key)? else {
                return Ok(None);
            };

            let previous = self.visible_value(index);
            unsafe {
                self.write_control(index, Self::CONTROL_DELETED);
            }
            Ok(previous)
        }

        fn lookup(&self, key: &[u8; 32]) -> Result<LookupResult, TableError> {
            self.lookup_with_hash(key, hash_actor_key(key))
        }

        fn lookup_with_hash(&self, key: &[u8; 32], hash: u32) -> Result<LookupResult, TableError> {
            let fingerprint = control_fingerprint(hash);
            let key_words = words_32(key);
            let mut first_deleted = None;
            let mut index = static_map_index(hash, LOG2_SLOTS);

            for _ in 0..self.slots {
                let control = unsafe { self.read_control(index) };
                match control {
                    Self::CONTROL_EMPTY => {
                        return Ok(LookupResult::Vacant(first_deleted.unwrap_or(index)));
                    }
                    Self::CONTROL_DELETED => {
                        if first_deleted.is_none() {
                            first_deleted = Some(index);
                        }
                    }
                    1..=Self::CONTROL_FULL_MAX => {
                        if control == fingerprint
                            && unsafe { read_words_32(self.key_ptr(index)) } == key_words
                        {
                            return Ok(LookupResult::Found(index));
                        }
                    }
                    _ => return Err(TableError::InvalidSlotState),
                }

                index = (index + 1) & self.mask;
            }

            Ok(first_deleted.map_or(LookupResult::Full, LookupResult::Vacant))
        }

        fn visible_value(&self, slot: usize) -> Option<[u8; 32]> {
            let value = unsafe { self.read_value(slot) };
            if is_zero_32(&value) {
                None
            } else {
                Some(value)
            }
        }

        unsafe fn read_control(&self, slot: usize) -> u8 {
            unsafe { ptr::read(self.control_ptr(slot)) }
        }

        unsafe fn read_value(&self, slot: usize) -> [u8; 32] {
            let mut bytes = [0u8; 32];
            unsafe {
                ptr::copy_nonoverlapping(self.value_ptr(slot), bytes.as_mut_ptr(), 32);
            }
            bytes
        }

        unsafe fn write_control(&self, slot: usize, control: u8) {
            unsafe {
                ptr::write(self.control_ptr(slot), control);
            }
        }

        unsafe fn write_key(&self, slot: usize, key: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(key.as_ptr(), self.key_ptr(slot), 32);
            }
        }

        unsafe fn write_value(&self, slot: usize, value: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), 32);
            }
        }

        fn control_ptr(&self, slot: usize) -> *mut u8 {
            (self.control_base + slot) as *mut u8
        }

        fn key_ptr(&self, slot: usize) -> *mut u8 {
            self.slot_ptr(slot)
        }

        fn value_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::VALUE_OFFSET) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            (self.slots_base + slot * Self::slot_size()) as *mut u8
        }
    }

    /// A page-local static balance map keyed by `ActorId` and storing `U256`.
    ///
    /// Each 16 KiB tile stores its control bytes and data slots together:
    /// 252 control bytes, 4 padding bytes, then 252 64-byte key/value slots.
    /// This keeps the control byte, actor key, and value for a slot inside the
    /// same Gear page while preserving control-byte rejection for missing keys.
    pub struct StaticPageLocalActorIdU256Map<const LOG2_TILES: u8> {
        base: usize,
        tiles: usize,
        tile_mask: usize,
        slots: usize,
        _marker: PhantomData<*mut u8>,
    }

    impl<const LOG2_TILES: u8> StaticPageLocalActorIdU256Map<LOG2_TILES> {
        const CONTROL_EMPTY: u8 = 0x00;
        const CONTROL_DELETED: u8 = 0xff;
        const CONTROL_FULL_MAX: u8 = 0x80;
        const VALUE_OFFSET: usize = 32;

        /// Returns the byte length of one data slot.
        pub const fn slot_size() -> usize {
            ACTOR_ID_U256_SLOT_SIZE
        }

        /// Returns the byte length of one page-local tile.
        pub const fn tile_bytes() -> usize {
            PAGE_LOCAL_ACTOR_U256_TILE_BYTES
        }

        /// Returns the data offset inside each tile.
        pub const fn data_offset() -> usize {
            PAGE_LOCAL_ACTOR_U256_DATA_OFFSET
        }

        /// Returns the number of actor/value slots inside each tile.
        pub const fn slots_per_tile() -> usize {
            PAGE_LOCAL_ACTOR_U256_SLOTS_PER_TILE
        }

        /// Returns the configured tile count.
        pub fn tiles() -> Result<usize, TableError> {
            static_map_slots(LOG2_TILES)
        }

        /// Returns the configured slot count.
        pub fn slots() -> Result<usize, TableError> {
            Self::tiles()?
                .checked_mul(Self::slots_per_tile())
                .ok_or(TableError::InvalidLayout)
        }

        /// Returns the byte length required for this map.
        pub fn bytes_len() -> Result<usize, TableError> {
            Self::tiles()?
                .checked_mul(Self::tile_bytes())
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates a page-local table over `2^LOG2_TILES * 16 KiB` bytes.
        ///
        /// # Safety
        ///
        /// The caller must ensure the memory interval is valid for reads and
        /// writes for the whole lifetime of the table and does not overlap
        /// other mutable state.
        pub unsafe fn new(base: usize) -> Result<Self, TableError> {
            let tiles = Self::tiles()?;
            let bytes = tiles
                .checked_mul(Self::tile_bytes())
                .ok_or(TableError::InvalidLayout)?;
            StaticRegion::new(base, bytes)?;
            let slots = tiles
                .checked_mul(Self::slots_per_tile())
                .ok_or(TableError::InvalidLayout)?;

            Ok(Self {
                base,
                tiles,
                tile_mask: tiles - 1,
                slots,
                _marker: PhantomData,
            })
        }

        /// Returns the configured base address.
        pub const fn base(&self) -> usize {
            self.base
        }

        /// Returns the configured tile count.
        pub const fn tile_count(&self) -> usize {
            self.tiles
        }

        /// Returns the logical byte length occupied by all tiles.
        pub fn bytes(&self) -> Result<usize, TableError> {
            Self::bytes_len()
        }

        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            if is_zero_32(&key) {
                return Ok(None);
            }

            let LookupResult::Found(index) = self.lookup(&key)? else {
                return Ok(None);
            };
            Ok(self.visible_value(index).map(u256_from_value))
        }

        /// Inserts or updates a balance value by actor id.
        ///
        /// A zero value removes the visible value without writing a new absent
        /// key. Zero actor ids are rejected because zeroed control memory
        /// already represents empty slots.
        pub fn insert_actor_u256(
            &self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let key = actor_key(key);
            reject_zero_key(&key)?;
            if value.is_zero() {
                return self
                    .remove_key(&key)
                    .map(|previous| previous.map(u256_from_value));
            }

            let value = u256_value(value);
            let hash = hash_actor_key(&key);
            match self.lookup_with_hash(&key, hash)? {
                LookupResult::Found(index) => {
                    let previous = self.visible_value(index);
                    unsafe {
                        self.write_value(index, &value);
                    }
                    Ok(previous.map(u256_from_value))
                }
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_key(index, &key);
                        self.write_value(index, &value);
                        self.write_control(index, control_fingerprint(hash));
                    }
                    Ok(None)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Removes a balance value by actor id.
        pub fn remove_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            reject_zero_key(&key)?;
            self.remove_key(&key)
                .map(|previous| previous.map(u256_from_value))
        }

        /// Clears every tile to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, Self::bytes_len()?);
            }
            Ok(())
        }

        fn remove_key(&self, key: &[u8; 32]) -> Result<Option<[u8; 32]>, TableError> {
            let LookupResult::Found(index) = self.lookup(key)? else {
                return Ok(None);
            };

            let previous = self.visible_value(index);
            unsafe {
                self.write_control(index, Self::CONTROL_DELETED);
            }
            Ok(previous)
        }

        fn lookup(&self, key: &[u8; 32]) -> Result<LookupResult, TableError> {
            self.lookup_with_hash(key, hash_actor_key(key))
        }

        fn lookup_with_hash(&self, key: &[u8; 32], hash: u32) -> Result<LookupResult, TableError> {
            let fingerprint = control_fingerprint(hash);
            let key_words = words_32(key);
            let mut first_deleted = None;
            let mut tile = static_map_index(hash, LOG2_TILES);
            let mut slot = (hash as usize) % Self::slots_per_tile();

            for _ in 0..self.slots {
                let index = tile * Self::slots_per_tile() + slot;
                let control = unsafe { self.read_control(index) };
                match control {
                    Self::CONTROL_EMPTY => {
                        return Ok(LookupResult::Vacant(first_deleted.unwrap_or(index)));
                    }
                    Self::CONTROL_DELETED => {
                        if first_deleted.is_none() {
                            first_deleted = Some(index);
                        }
                    }
                    1..=Self::CONTROL_FULL_MAX => {
                        if control == fingerprint
                            && unsafe { read_words_32(self.key_ptr(index)) } == key_words
                        {
                            return Ok(LookupResult::Found(index));
                        }
                    }
                    _ => return Err(TableError::InvalidSlotState),
                }

                slot += 1;
                if slot == Self::slots_per_tile() {
                    slot = 0;
                    tile = (tile + 1) & self.tile_mask;
                }
            }

            Ok(first_deleted.map_or(LookupResult::Full, LookupResult::Vacant))
        }

        fn visible_value(&self, slot: usize) -> Option<[u8; 32]> {
            let value = unsafe { self.read_value(slot) };
            if is_zero_32(&value) {
                None
            } else {
                Some(value)
            }
        }

        unsafe fn read_control(&self, slot: usize) -> u8 {
            unsafe { ptr::read(self.control_ptr(slot)) }
        }

        unsafe fn read_value(&self, slot: usize) -> [u8; 32] {
            let mut bytes = [0u8; 32];
            unsafe {
                ptr::copy_nonoverlapping(self.value_ptr(slot), bytes.as_mut_ptr(), 32);
            }
            bytes
        }

        unsafe fn write_control(&self, slot: usize, control: u8) {
            unsafe {
                ptr::write(self.control_ptr(slot), control);
            }
        }

        unsafe fn write_key(&self, slot: usize, key: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(key.as_ptr(), self.key_ptr(slot), 32);
            }
        }

        unsafe fn write_value(&self, slot: usize, value: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), 32);
            }
        }

        fn control_ptr(&self, slot: usize) -> *mut u8 {
            let (tile, tile_slot) = self.tile_slot(slot);
            (self.base + tile * Self::tile_bytes() + tile_slot) as *mut u8
        }

        fn key_ptr(&self, slot: usize) -> *mut u8 {
            self.slot_ptr(slot)
        }

        fn value_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::VALUE_OFFSET) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            let (tile, tile_slot) = self.tile_slot(slot);
            (self.base
                + tile * Self::tile_bytes()
                + Self::data_offset()
                + tile_slot * Self::slot_size()) as *mut u8
        }

        fn tile_slot(&self, slot: usize) -> (usize, usize) {
            (slot / Self::slots_per_tile(), slot % Self::slots_per_tile())
        }
    }

    /// A grouped-control static balance map keyed by `ActorId` and storing `U256`.
    ///
    /// Each group stores a compact control run followed by its 64-byte key/value
    /// slots. Increasing `LOG2_GROUP_PAGES` makes control scanning denser for
    /// misses while keeping hits closer to their data than a fully separated
    /// global control region.
    pub struct StaticGroupedControlActorIdU256Map<const LOG2_GROUPS: u8, const LOG2_GROUP_PAGES: u8> {
        base: usize,
        groups: usize,
        group_mask: usize,
        group_bytes: usize,
        slots_per_group: usize,
        data_offset: usize,
        slots: usize,
        _marker: PhantomData<*mut u8>,
    }

    impl<const LOG2_GROUPS: u8, const LOG2_GROUP_PAGES: u8>
        StaticGroupedControlActorIdU256Map<LOG2_GROUPS, LOG2_GROUP_PAGES>
    {
        const CONTROL_EMPTY: u8 = 0x00;
        const CONTROL_DELETED: u8 = 0xff;
        const CONTROL_FULL_MAX: u8 = 0x80;
        const VALUE_OFFSET: usize = 32;

        /// Returns the byte length of one data slot.
        pub const fn slot_size() -> usize {
            ACTOR_ID_U256_SLOT_SIZE
        }

        /// Returns the configured group count.
        pub fn groups() -> Result<usize, TableError> {
            static_map_slots(LOG2_GROUPS)
        }

        /// Returns the number of Gear pages inside each group.
        pub fn group_pages() -> Result<usize, TableError> {
            static_map_slots(LOG2_GROUP_PAGES)
        }

        /// Returns the number of actor/value slots inside each group.
        pub fn slots_per_group() -> Result<usize, TableError> {
            Self::group_pages()?
                .checked_mul(PAGE_LOCAL_ACTOR_U256_SLOTS_PER_TILE)
                .ok_or(TableError::InvalidLayout)
        }

        /// Returns the data offset inside each group.
        pub fn data_offset() -> Result<usize, TableError> {
            Self::group_pages()?
                .checked_mul(PAGE_LOCAL_ACTOR_U256_DATA_OFFSET)
                .ok_or(TableError::InvalidLayout)
        }

        /// Returns the byte length of one group.
        pub fn group_bytes() -> Result<usize, TableError> {
            Self::group_pages()?
                .checked_mul(PAGE_LOCAL_ACTOR_U256_TILE_BYTES)
                .ok_or(TableError::InvalidLayout)
        }

        /// Returns the configured slot count.
        pub fn slots() -> Result<usize, TableError> {
            Self::groups()?
                .checked_mul(Self::slots_per_group()?)
                .ok_or(TableError::InvalidLayout)
        }

        /// Returns the byte length required for this map.
        pub fn bytes_len() -> Result<usize, TableError> {
            Self::groups()?
                .checked_mul(Self::group_bytes()?)
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates a grouped-control table over the configured static region.
        ///
        /// # Safety
        ///
        /// The caller must ensure the memory interval is valid for reads and
        /// writes for the whole lifetime of the table and does not overlap
        /// other mutable state.
        pub unsafe fn new(base: usize) -> Result<Self, TableError> {
            let groups = Self::groups()?;
            let group_bytes = Self::group_bytes()?;
            let slots_per_group = Self::slots_per_group()?;
            let data_offset = Self::data_offset()?;
            let bytes = groups
                .checked_mul(group_bytes)
                .ok_or(TableError::InvalidLayout)?;
            StaticRegion::new(base, bytes)?;
            let slots = groups
                .checked_mul(slots_per_group)
                .ok_or(TableError::InvalidLayout)?;

            Ok(Self {
                base,
                groups,
                group_mask: groups - 1,
                group_bytes,
                slots_per_group,
                data_offset,
                slots,
                _marker: PhantomData,
            })
        }

        /// Returns the configured base address.
        pub const fn base(&self) -> usize {
            self.base
        }

        /// Returns the configured group count.
        pub const fn group_count(&self) -> usize {
            self.groups
        }

        /// Returns the byte length occupied by all groups.
        pub fn bytes(&self) -> Result<usize, TableError> {
            Self::bytes_len()
        }

        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            if is_zero_32(&key) {
                return Ok(None);
            }

            let LookupResult::Found(index) = self.lookup(&key)? else {
                return Ok(None);
            };
            Ok(self.visible_value(index).map(u256_from_value))
        }

        /// Inserts or updates a balance value by actor id.
        ///
        /// A zero value removes the visible value without writing a new absent
        /// key. Zero actor ids are rejected because zeroed control memory
        /// already represents empty slots.
        pub fn insert_actor_u256(
            &self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let key = actor_key(key);
            reject_zero_key(&key)?;
            if value.is_zero() {
                return self
                    .remove_key(&key)
                    .map(|previous| previous.map(u256_from_value));
            }

            let value = u256_value(value);
            let hash = hash_actor_key(&key);
            match self.lookup_with_hash(&key, hash)? {
                LookupResult::Found(index) => {
                    let previous = self.visible_value(index);
                    unsafe {
                        self.write_value(index, &value);
                    }
                    Ok(previous.map(u256_from_value))
                }
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_key(index, &key);
                        self.write_value(index, &value);
                        self.write_control(index, control_fingerprint(hash));
                    }
                    Ok(None)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Removes a balance value by actor id.
        pub fn remove_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            let key = actor_key(*key);
            reject_zero_key(&key)?;
            self.remove_key(&key)
                .map(|previous| previous.map(u256_from_value))
        }

        /// Clears every group to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, Self::bytes_len()?);
            }
            Ok(())
        }

        fn remove_key(&self, key: &[u8; 32]) -> Result<Option<[u8; 32]>, TableError> {
            let LookupResult::Found(index) = self.lookup(key)? else {
                return Ok(None);
            };

            let previous = self.visible_value(index);
            unsafe {
                self.write_control(index, Self::CONTROL_DELETED);
            }
            Ok(previous)
        }

        fn lookup(&self, key: &[u8; 32]) -> Result<LookupResult, TableError> {
            self.lookup_with_hash(key, hash_actor_key(key))
        }

        fn lookup_with_hash(&self, key: &[u8; 32], hash: u32) -> Result<LookupResult, TableError> {
            let fingerprint = control_fingerprint(hash);
            let key_words = words_32(key);
            let mut first_deleted = None;
            let mut group = static_map_index(hash, LOG2_GROUPS);
            let mut group_slot = (hash as usize) % self.slots_per_group;

            for _ in 0..self.slots {
                let index = group * self.slots_per_group + group_slot;
                let control = unsafe { self.read_control(index) };
                match control {
                    Self::CONTROL_EMPTY => {
                        return Ok(LookupResult::Vacant(first_deleted.unwrap_or(index)));
                    }
                    Self::CONTROL_DELETED => {
                        if first_deleted.is_none() {
                            first_deleted = Some(index);
                        }
                    }
                    1..=Self::CONTROL_FULL_MAX => {
                        if control == fingerprint
                            && unsafe { read_words_32(self.key_ptr(index)) } == key_words
                        {
                            return Ok(LookupResult::Found(index));
                        }
                    }
                    _ => return Err(TableError::InvalidSlotState),
                }

                group_slot += 1;
                if group_slot == self.slots_per_group {
                    group_slot = 0;
                    group = (group + 1) & self.group_mask;
                }
            }

            Ok(first_deleted.map_or(LookupResult::Full, LookupResult::Vacant))
        }

        fn visible_value(&self, slot: usize) -> Option<[u8; 32]> {
            let value = unsafe { self.read_value(slot) };
            if is_zero_32(&value) {
                None
            } else {
                Some(value)
            }
        }

        unsafe fn read_control(&self, slot: usize) -> u8 {
            unsafe { ptr::read(self.control_ptr(slot)) }
        }

        unsafe fn read_value(&self, slot: usize) -> [u8; 32] {
            let mut bytes = [0u8; 32];
            unsafe {
                ptr::copy_nonoverlapping(self.value_ptr(slot), bytes.as_mut_ptr(), 32);
            }
            bytes
        }

        unsafe fn write_control(&self, slot: usize, control: u8) {
            unsafe {
                ptr::write(self.control_ptr(slot), control);
            }
        }

        unsafe fn write_key(&self, slot: usize, key: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(key.as_ptr(), self.key_ptr(slot), 32);
            }
        }

        unsafe fn write_value(&self, slot: usize, value: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), 32);
            }
        }

        fn control_ptr(&self, slot: usize) -> *mut u8 {
            let (group, group_slot) = self.group_slot(slot);
            (self.base + group * self.group_bytes + group_slot) as *mut u8
        }

        fn key_ptr(&self, slot: usize) -> *mut u8 {
            self.slot_ptr(slot)
        }

        fn value_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::VALUE_OFFSET) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            let (group, group_slot) = self.group_slot(slot);
            (self.base
                + group * self.group_bytes
                + self.data_offset
                + group_slot * Self::slot_size()) as *mut u8
        }

        fn group_slot(&self, slot: usize) -> (usize, usize) {
            (slot / self.slots_per_group, slot % self.slots_per_group)
        }
    }

    /// A WAT-shaped static allowance map keyed by `(owner, spender)` and storing `U256`.
    ///
    /// Each slot is exactly 96 bytes: 32 bytes of owner, 32 bytes of spender,
    /// and 32 bytes of value. A zero owner marks an empty slot, and a zero
    /// value is treated as an absent/tombstoned entry for an existing key.
    pub struct StaticAllowanceU256Map<const LOG2_SLOTS: u8> {
        base: usize,
        slots: usize,
        mask: usize,
        _marker: PhantomData<*mut u8>,
    }

    impl<const LOG2_SLOTS: u8> StaticAllowanceU256Map<LOG2_SLOTS> {
        const SPENDER_OFFSET: usize = 32;
        const VALUE_OFFSET: usize = 64;

        /// Returns the byte length of one slot.
        pub const fn slot_size() -> usize {
            ALLOWANCE_U256_SLOT_SIZE
        }

        /// Returns the configured slot count.
        pub fn slots() -> Result<usize, TableError> {
            static_map_slots(LOG2_SLOTS)
        }

        /// Returns the mask used for power-of-two probing.
        pub fn mask() -> Result<usize, TableError> {
            Ok(Self::slots()? - 1)
        }

        /// Returns the byte length required for this table.
        pub fn bytes_len() -> Result<usize, TableError> {
            Self::slots()?
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates a table over `2^LOG2_SLOTS * 96` bytes at `base`.
        ///
        /// # Safety
        ///
        /// The caller must ensure the memory interval is valid for reads and
        /// writes for the whole lifetime of the table and does not overlap
        /// other mutable state.
        pub unsafe fn new(base: usize) -> Result<Self, TableError> {
            let slots = Self::slots()?;
            let bytes = slots
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)?;
            StaticRegion::new(base, bytes)?;
            Ok(Self {
                base,
                slots,
                mask: slots - 1,
                _marker: PhantomData,
            })
        }

        /// Returns the configured base address.
        pub const fn base(&self) -> usize {
            self.base
        }

        /// Returns the total byte length occupied by this table.
        pub fn bytes(&self) -> Result<usize, TableError> {
            Self::bytes_len()
        }

        /// Returns an allowance value by owner and spender.
        pub fn get_allowance_u256(
            &self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            let owner = actor_key(*owner);
            let spender = actor_key(*spender);
            if is_zero_32(&owner) || is_zero_32(&spender) {
                return Ok(None);
            }

            let LookupResult::Found(index) = self.lookup(&owner, &spender)? else {
                return Ok(None);
            };
            Ok(self.visible_value(index).map(u256_from_value))
        }

        /// Inserts or updates an allowance value by owner and spender.
        ///
        /// A zero value removes the visible value without writing a new absent
        /// key. Zero owner or spender ids are rejected because zero key bytes
        /// are reserved by this layout.
        pub fn insert_allowance_u256(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let owner = actor_key(owner);
            let spender = actor_key(spender);
            reject_zero_key(&owner)?;
            reject_zero_key(&spender)?;
            let value = u256_value(value);

            if is_zero_32(&value) {
                return self
                    .remove_key(&owner, &spender)
                    .map(|previous| previous.map(u256_from_value));
            }

            match self.lookup(&owner, &spender)? {
                LookupResult::Found(index) => {
                    let previous = self.visible_value(index);
                    unsafe {
                        self.write_value(index, &value);
                    }
                    Ok(previous.map(u256_from_value))
                }
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_owner(index, &owner);
                        self.write_spender(index, &spender);
                        self.write_value(index, &value);
                    }
                    Ok(None)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        /// Removes an allowance value by owner and spender.
        pub fn remove_allowance_u256(
            &self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            let owner = actor_key(*owner);
            let spender = actor_key(*spender);
            reject_zero_key(&owner)?;
            reject_zero_key(&spender)?;
            self.remove_key(&owner, &spender)
                .map(|previous| previous.map(u256_from_value))
        }

        /// Decreases an allowance using one lookup.
        ///
        /// Returns `Ok(None)` without modifying storage when the allowance is
        /// absent or smaller than `amount`.
        pub fn decrease_allowance_u256(
            &self,
            owner: ActorId,
            spender: ActorId,
            amount: U256,
        ) -> Result<Option<U256>, TableError> {
            let owner = actor_key(owner);
            let spender = actor_key(spender);
            reject_zero_key(&owner)?;
            reject_zero_key(&spender)?;
            if amount.is_zero() {
                let LookupResult::Found(index) = self.lookup(&owner, &spender)? else {
                    return Ok(Some(U256::zero()));
                };
                return Ok(Some(
                    self.visible_value(index)
                        .map_or_else(U256::zero, u256_from_value),
                ));
            }
            let LookupResult::Found(index) = self.lookup(&owner, &spender)? else {
                return Ok(None);
            };
            let Some(value) = self.visible_value(index).map(u256_from_value) else {
                return Ok(None);
            };
            if value < amount {
                return Ok(None);
            }

            let value = value - amount;
            let value_bytes = u256_value(value);
            unsafe {
                self.write_value(index, &value_bytes);
            }
            Ok(Some(value))
        }

        /// Clears every slot to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            let bytes = self.bytes()?;
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, bytes);
            }
            Ok(())
        }

        fn remove_key(
            &self,
            owner: &[u8; 32],
            spender: &[u8; 32],
        ) -> Result<Option<[u8; 32]>, TableError> {
            let LookupResult::Found(index) = self.lookup(owner, spender)? else {
                return Ok(None);
            };

            let previous = self.visible_value(index);
            if previous.is_some() {
                unsafe {
                    self.write_value(index, &[0; 32]);
                }
            }
            Ok(previous)
        }

        fn lookup(&self, owner: &[u8; 32], spender: &[u8; 32]) -> Result<LookupResult, TableError> {
            let owner_words = words_32(owner);
            let spender_words = words_32(spender);
            let mut index = static_map_index(hash_allowance_key(owner, spender), LOG2_SLOTS);

            for _ in 0..self.slots {
                let stored_owner = unsafe { read_words_32(self.owner_ptr(index)) };
                if words_are_zero(stored_owner) {
                    return Ok(LookupResult::Vacant(index));
                }
                if stored_owner == owner_words
                    && unsafe { read_words_32(self.spender_ptr(index)) } == spender_words
                {
                    return Ok(LookupResult::Found(index));
                }

                index = (index + 1) & self.mask;
            }

            Ok(LookupResult::Full)
        }

        fn visible_value(&self, slot: usize) -> Option<[u8; 32]> {
            if unsafe { words_at_zero(self.value_ptr(slot)) } {
                None
            } else {
                Some(unsafe { self.read_value(slot) })
            }
        }

        unsafe fn read_value(&self, slot: usize) -> [u8; 32] {
            let mut bytes = [0u8; 32];
            unsafe {
                ptr::copy_nonoverlapping(self.value_ptr(slot), bytes.as_mut_ptr(), 32);
            }
            bytes
        }

        unsafe fn write_owner(&self, slot: usize, owner: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(owner.as_ptr(), self.owner_ptr(slot), 32);
            }
        }

        unsafe fn write_spender(&self, slot: usize, spender: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(spender.as_ptr(), self.spender_ptr(slot), 32);
            }
        }

        unsafe fn write_value(&self, slot: usize, value: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), 32);
            }
        }

        fn owner_ptr(&self, slot: usize) -> *mut u8 {
            self.slot_ptr(slot)
        }

        fn spender_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::SPENDER_OFFSET) }
        }

        fn value_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::VALUE_OFFSET) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            (self.base + slot * Self::slot_size()) as *mut u8
        }
    }

    /// Experimental owner-local VFT account map.
    ///
    /// Each account slot stores one owner, one balance, and two inline
    /// `(spender, allowance)` pairs. Additional spender allowances should be
    /// stored in an overflow [`StaticAllowanceU256Map`].
    #[cfg(feature = "experimental-vft-account")]
    pub struct StaticVftAccountU256Map<const LOG2_SLOTS: u8> {
        base: usize,
        slots: usize,
        mask: usize,
        _marker: PhantomData<*mut u8>,
    }

    #[cfg(feature = "experimental-vft-account")]
    impl<const LOG2_SLOTS: u8> StaticVftAccountU256Map<LOG2_SLOTS> {
        const OWNER_STATE_OFFSET: usize = 0;
        const SPENDER0_STATE_OFFSET: usize = 1;
        const SPENDER1_STATE_OFFSET: usize = 2;
        const OWNER_OFFSET: usize = 8;
        const BALANCE_OFFSET: usize = 40;
        const SPENDER0_OFFSET: usize = 72;
        const ALLOWANCE0_OFFSET: usize = 104;
        const SPENDER1_OFFSET: usize = 136;
        const ALLOWANCE1_OFFSET: usize = 168;

        /// Returns the byte length of one account slot.
        pub const fn slot_size() -> usize {
            VFT_ACCOUNT_U256_SLOT_SIZE
        }

        /// Returns the configured slot count.
        pub fn slots() -> Result<usize, TableError> {
            static_map_slots(LOG2_SLOTS)
        }

        /// Returns the mask used for power-of-two probing.
        pub fn mask() -> Result<usize, TableError> {
            Ok(Self::slots()? - 1)
        }

        /// Returns the byte length required for this account map.
        pub fn bytes_len() -> Result<usize, TableError> {
            Self::slots()?
                .checked_mul(Self::slot_size())
                .ok_or(TableError::InvalidLayout)
        }

        /// Creates an account map over `2^LOG2_SLOTS * 200` bytes at `base`.
        ///
        /// # Safety
        ///
        /// The caller must ensure the memory interval is valid for reads and
        /// writes for the whole lifetime of the map and does not overlap other
        /// mutable state.
        pub unsafe fn new(base: usize) -> Result<Self, TableError> {
            let slots = Self::slots()?;
            let bytes = Self::bytes_len()?;
            StaticRegion::new(base, bytes)?;
            Ok(Self {
                base,
                slots,
                mask: slots - 1,
                _marker: PhantomData,
            })
        }

        /// Returns the configured base address.
        pub const fn base(&self) -> usize {
            self.base
        }

        /// Returns the total byte length occupied by this table.
        pub fn bytes(&self) -> Result<usize, TableError> {
            Self::bytes_len()
        }

        /// Returns the visible balance for `owner`.
        pub fn get_balance(&self, owner: ActorId) -> Result<Option<U256>, TableError> {
            let owner = actor_key(owner);
            if is_zero_32(&owner) {
                return Ok(None);
            }
            let LookupResult::Found(index) = self.lookup(&owner)? else {
                return Ok(None);
            };
            Ok(Some(unsafe { self.read_u256(index, Self::BALANCE_OFFSET) }))
        }

        /// Inserts or updates an owner balance.
        pub fn insert_balance(
            &self,
            owner: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            let owner = actor_key(owner);
            reject_zero_key(&owner)?;
            let index = self.lookup_or_insert(&owner)?;
            let previous = unsafe { self.read_u256(index, Self::BALANCE_OFFSET) };
            unsafe {
                self.write_u256(index, Self::BALANCE_OFFSET, value);
            }
            Ok((!previous.is_zero()).then_some(previous))
        }

        /// Returns the inline allowance for `owner` and `spender`.
        pub fn get_inline_allowance(
            &self,
            owner: ActorId,
            spender: ActorId,
        ) -> Result<Option<U256>, TableError> {
            let owner = actor_key(owner);
            let spender = actor_key(spender);
            if is_zero_32(&owner) || is_zero_32(&spender) {
                return Ok(None);
            }
            let LookupResult::Found(index) = self.lookup(&owner)? else {
                return Ok(None);
            };
            Ok(self
                .inline_allowance_offset(index, &spender)?
                .map(|offset| unsafe { self.read_u256(index, offset) }))
        }

        /// Inserts or updates an inline allowance.
        ///
        /// Returns `Ok(Some(previous))` when the allowance fit in an inline
        /// slot. Returns `Ok(None)` when both inline slots are occupied by
        /// other spenders and the caller should use overflow storage.
        pub fn insert_inline_allowance(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<Option<Option<U256>>, TableError> {
            let owner = actor_key(owner);
            let spender = actor_key(spender);
            reject_zero_key(&owner)?;
            reject_zero_key(&spender)?;
            let index = self.lookup_or_insert(&owner)?;

            if let Some((state_offset, spender_offset, value_offset)) =
                self.inline_slot(index, &spender)?
            {
                let previous = unsafe { self.read_u256(index, value_offset) };
                unsafe {
                    if value.is_zero() {
                        self.clear_inline_slot(index, state_offset, spender_offset, value_offset);
                    } else {
                        self.write_u256(index, value_offset, value);
                    }
                }
                return Ok(Some(Some(previous)));
            }
            if value.is_zero() {
                return Ok(Some(None));
            }

            if self.read_state(index, Self::SPENDER0_STATE_OFFSET)? != SlotState::Full {
                unsafe {
                    self.write_state(index, Self::SPENDER0_STATE_OFFSET, SlotState::Full);
                    self.write_actor(index, Self::SPENDER0_OFFSET, &spender);
                    self.write_u256(index, Self::ALLOWANCE0_OFFSET, value);
                }
                return Ok(Some(None));
            }
            if self.read_state(index, Self::SPENDER1_STATE_OFFSET)? != SlotState::Full {
                unsafe {
                    self.write_state(index, Self::SPENDER1_STATE_OFFSET, SlotState::Full);
                    self.write_actor(index, Self::SPENDER1_OFFSET, &spender);
                    self.write_u256(index, Self::ALLOWANCE1_OFFSET, value);
                }
                return Ok(Some(None));
            }

            Ok(None)
        }

        /// Moves `amount` from `from` to `to`.
        pub fn transfer(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            if from == to || amount.is_zero() {
                return Ok(false);
            }
            let from = actor_key(from);
            let to = actor_key(to);
            reject_zero_key(&from)?;
            reject_zero_key(&to)?;

            let LookupResult::Found(from_index) = self.lookup(&from)? else {
                return Ok(false);
            };
            let from_balance = unsafe { self.read_u256(from_index, Self::BALANCE_OFFSET) };
            if from_balance < amount {
                return Ok(false);
            }

            let to_index = self.lookup_or_insert(&to)?;
            let to_balance = unsafe { self.read_u256(to_index, Self::BALANCE_OFFSET) };
            let (to_balance, overflow) = to_balance.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }
            unsafe {
                self.write_u256(from_index, Self::BALANCE_OFFSET, from_balance - amount);
                self.write_u256(to_index, Self::BALANCE_OFFSET, to_balance);
            }
            Ok(true)
        }

        /// Clears every account slot to the empty state.
        pub fn clear(&self) -> Result<(), TableError> {
            let bytes = self.bytes()?;
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, bytes);
            }
            Ok(())
        }

        fn lookup(&self, owner: &[u8; 32]) -> Result<LookupResult, TableError> {
            let owner_words = words_32(owner);
            let mut index = self.account_index(owner);
            for _ in 0..self.slots {
                match self.read_state(index, Self::OWNER_STATE_OFFSET)? {
                    SlotState::Empty => return Ok(LookupResult::Vacant(index)),
                    SlotState::Full => {
                        if unsafe { read_words_32(self.owner_ptr(index)) } == owner_words {
                            return Ok(LookupResult::Found(index));
                        }
                    }
                    SlotState::Deleted => {}
                }
                index = (index + 1) & self.mask;
            }
            Ok(LookupResult::Full)
        }

        fn lookup_or_insert(&self, owner: &[u8; 32]) -> Result<usize, TableError> {
            match self.lookup(owner)? {
                LookupResult::Found(index) => Ok(index),
                LookupResult::Vacant(index) => {
                    unsafe {
                        self.write_state(index, Self::OWNER_STATE_OFFSET, SlotState::Full);
                        self.write_actor(index, Self::OWNER_OFFSET, owner);
                    }
                    Ok(index)
                }
                LookupResult::Full => Err(TableError::CapacityOverflow),
            }
        }

        fn inline_allowance_offset(
            &self,
            index: usize,
            spender: &[u8; 32],
        ) -> Result<Option<usize>, TableError> {
            Ok(self
                .inline_slot(index, spender)?
                .map(|(_, _, value_offset)| value_offset))
        }

        fn inline_slot(
            &self,
            index: usize,
            spender: &[u8; 32],
        ) -> Result<Option<(usize, usize, usize)>, TableError> {
            let spender_words = words_32(spender);
            if self.read_state(index, Self::SPENDER0_STATE_OFFSET)? == SlotState::Full
                && unsafe { read_words_32(self.spender_ptr(index, Self::SPENDER0_OFFSET)) }
                    == spender_words
            {
                return Ok(Some((
                    Self::SPENDER0_STATE_OFFSET,
                    Self::SPENDER0_OFFSET,
                    Self::ALLOWANCE0_OFFSET,
                )));
            }
            if self.read_state(index, Self::SPENDER1_STATE_OFFSET)? == SlotState::Full
                && unsafe { read_words_32(self.spender_ptr(index, Self::SPENDER1_OFFSET)) }
                    == spender_words
            {
                return Ok(Some((
                    Self::SPENDER1_STATE_OFFSET,
                    Self::SPENDER1_OFFSET,
                    Self::ALLOWANCE1_OFFSET,
                )));
            }
            Ok(None)
        }

        fn account_index(&self, owner: &[u8; 32]) -> usize {
            let tag = unsafe { ptr::read_unaligned(owner.as_ptr().add(12).cast::<u64>()) };
            let hash = if tag != 0 {
                tag
            } else {
                unsafe {
                    ptr::read_unaligned(owner.as_ptr().cast::<u64>())
                        ^ ptr::read_unaligned(owner.as_ptr().add(8).cast::<u64>())
                        ^ ptr::read_unaligned(owner.as_ptr().add(16).cast::<u64>())
                        ^ ptr::read_unaligned(owner.as_ptr().add(24).cast::<u64>())
                }
            };
            hash as usize & self.mask
        }

        fn read_state(&self, slot: usize, offset: usize) -> Result<SlotState, TableError> {
            SlotState::from_byte(unsafe { ptr::read(self.slot_ptr(slot).add(offset)) })
        }

        unsafe fn read_u256(&self, slot: usize, offset: usize) -> U256 {
            let mut bytes = [0u8; 32];
            unsafe {
                ptr::copy_nonoverlapping(self.slot_ptr(slot).add(offset), bytes.as_mut_ptr(), 32);
            }
            U256::from_little_endian(&bytes)
        }

        unsafe fn write_state(&self, slot: usize, offset: usize, state: SlotState) {
            unsafe {
                ptr::write(self.slot_ptr(slot).add(offset), state as u8);
            }
        }

        unsafe fn write_actor(&self, slot: usize, offset: usize, actor: &[u8; 32]) {
            unsafe {
                ptr::copy_nonoverlapping(actor.as_ptr(), self.slot_ptr(slot).add(offset), 32);
            }
        }

        unsafe fn write_u256(&self, slot: usize, offset: usize, value: U256) {
            let bytes = u256_value(value);
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.slot_ptr(slot).add(offset), 32);
            }
        }

        unsafe fn clear_inline_slot(
            &self,
            slot: usize,
            state_offset: usize,
            spender_offset: usize,
            value_offset: usize,
        ) {
            unsafe {
                self.write_state(slot, state_offset, SlotState::Empty);
                ptr::write_bytes(self.slot_ptr(slot).add(spender_offset), 0, 32);
                ptr::write_bytes(self.slot_ptr(slot).add(value_offset), 0, 32);
            }
        }

        fn owner_ptr(&self, slot: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(Self::OWNER_OFFSET) }
        }

        fn spender_ptr(&self, slot: usize, offset: usize) -> *mut u8 {
            unsafe { self.slot_ptr(slot).add(offset) }
        }

        fn slot_ptr(&self, slot: usize) -> *mut u8 {
            (self.base + slot * Self::slot_size()) as *mut u8
        }
    }

    /// Experimental VFT storage with owner-local hot allowances.
    #[cfg(feature = "experimental-vft-account")]
    pub struct StaticVftAccountStorage<
        const ACCOUNT_LOG2_SLOTS: u8,
        const OVERFLOW_ALLOWANCE_LOG2_SLOTS: u8,
    > {
        accounts: StaticVftAccountMap<ACCOUNT_LOG2_SLOTS>,
        overflow_allowances: VftAllowances<OVERFLOW_ALLOWANCE_LOG2_SLOTS>,
    }

    #[cfg(feature = "experimental-vft-account")]
    impl<const ACCOUNT_LOG2_SLOTS: u8, const OVERFLOW_ALLOWANCE_LOG2_SLOTS: u8>
        StaticVftAccountStorage<ACCOUNT_LOG2_SLOTS, OVERFLOW_ALLOWANCE_LOG2_SLOTS>
    {
        /// Creates owner-local VFT storage over separately reserved static regions.
        ///
        /// # Safety
        ///
        /// The caller must ensure both memory intervals are valid for reads and
        /// writes for the whole lifetime of the storage and do not overlap other
        /// mutable state.
        pub unsafe fn new(account_base: usize, overflow_allowance_base: usize) -> Result<Self, TableError> {
            let account_region = StaticRegion::new(
                account_base,
                StaticVftAccountMap::<ACCOUNT_LOG2_SLOTS>::bytes_len()?,
            )?;
            let overflow_region = StaticRegion::new(
                overflow_allowance_base,
                VftAllowances::<OVERFLOW_ALLOWANCE_LOG2_SLOTS>::bytes_len()?,
            )?;
            if regions_overlap(account_region, overflow_region)? {
                return Err(TableError::InvalidLayout);
            }

            Ok(Self {
                accounts: unsafe { StaticVftAccountMap::<ACCOUNT_LOG2_SLOTS>::new(account_base)? },
                overflow_allowances: unsafe {
                    VftAllowances::<OVERFLOW_ALLOWANCE_LOG2_SLOTS>::new(overflow_allowance_base)?
                },
            })
        }

        /// Returns the account map backing this storage.
        pub fn accounts(&self) -> &StaticVftAccountMap<ACCOUNT_LOG2_SLOTS> {
            &self.accounts
        }

        /// Returns the overflow allowance map backing this storage.
        pub fn overflow_allowances(
            &self,
        ) -> &VftAllowances<OVERFLOW_ALLOWANCE_LOG2_SLOTS> {
            &self.overflow_allowances
        }

        /// Returns the visible balance of `account`.
        pub fn balance_of(&self, account: ActorId) -> Result<U256, TableError> {
            Ok(self
                .accounts
                .get_balance(account)?
                .unwrap_or_else(U256::zero))
        }

        /// Returns the visible allowance from `owner` to `spender`.
        pub fn allowance(&self, owner: ActorId, spender: ActorId) -> Result<U256, TableError> {
            if let Some(allowance) = self.accounts.get_inline_allowance(owner, spender)? {
                return Ok(allowance);
            }
            Ok(self
                .overflow_allowances
                .get_allowance_u256(&owner, &spender)?
                .unwrap_or_else(U256::zero))
        }

        /// Sets an allowance and returns whether the visible value changed.
        pub fn approve(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<bool, TableError> {
            if owner == spender {
                return Ok(false);
            }
            match self.accounts.insert_inline_allowance(owner, spender, value)? {
                Some(previous) => {
                    if value.is_zero() {
                        self.overflow_allowances
                            .insert_allowance_u256(owner, spender, U256::zero())?;
                    }
                    Ok(previous.unwrap_or_else(U256::zero) != value)
                }
                None => {
                    let previous = self
                        .overflow_allowances
                        .insert_allowance_u256(owner, spender, value)?
                        .unwrap_or_else(U256::zero);
                    Ok(previous != value)
                }
            }
        }

        /// Adds `amount` to `to` and returns whether state changed.
        pub fn mint(&self, to: ActorId, amount: U256) -> Result<bool, TableError> {
            if amount.is_zero() {
                return Ok(false);
            }

            let previous = self.balance_of(to)?;
            let (next, overflow) = previous.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }

            self.accounts.insert_balance(to, next)?;
            Ok(true)
        }

        /// Removes `amount` from `from` and returns whether state changed.
        pub fn burn(&self, from: ActorId, amount: U256) -> Result<bool, TableError> {
            if amount.is_zero() {
                return Ok(false);
            }

            let previous = self.balance_of(from)?;
            if previous < amount {
                return Ok(false);
            }

            self.accounts.insert_balance(from, previous - amount)?;
            Ok(true)
        }

        /// Moves `amount` from `from` to `to` and returns whether state changed.
        pub fn transfer(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            self.accounts.transfer(from, to, amount)
        }

        /// Moves `amount` after validating and decreasing `spender` allowance.
        pub fn transfer_from(
            &self,
            spender: ActorId,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            if spender == from {
                return self.transfer(from, to, amount);
            }
            if from == to || amount.is_zero() {
                return Ok(false);
            }

            let Some(from_index) = self.accounts.lookup(&actor_key(from))?.found() else {
                return Ok(false);
            };
            let spender_key = actor_key(spender);
            let inline_slot = self.accounts.inline_slot(from_index, &spender_key)?;
            let allowance = if let Some((_, _, offset)) = inline_slot {
                unsafe { self.accounts.read_u256(from_index, offset) }
            } else {
                self.overflow_allowances
                    .get_allowance_u256(&from, &spender)?
                    .unwrap_or_else(U256::zero)
            };
            if allowance < amount {
                return Ok(false);
            }

            let owner_balance = unsafe {
                self.accounts.read_u256(
                    from_index,
                    StaticVftAccountMap::<ACCOUNT_LOG2_SLOTS>::BALANCE_OFFSET,
                )
            };
            if owner_balance < amount {
                return Ok(false);
            }

            let to_key = actor_key(to);
            reject_zero_key(&to_key)?;
            let to_index = self.accounts.lookup_or_insert(&to_key)?;
            let to_balance = unsafe {
                self.accounts.read_u256(
                    to_index,
                    StaticVftAccountMap::<ACCOUNT_LOG2_SLOTS>::BALANCE_OFFSET,
                )
            };
            let (to_balance, overflow) = to_balance.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }

            unsafe {
                if let Some((state_offset, spender_offset, value_offset)) = inline_slot {
                    let allowance = allowance - amount;
                    if allowance.is_zero() {
                        self.accounts.clear_inline_slot(
                            from_index,
                            state_offset,
                            spender_offset,
                            value_offset,
                        );
                    } else {
                        self.accounts
                            .write_u256(from_index, value_offset, allowance);
                    }
                } else {
                    self.overflow_allowances
                        .insert_allowance_u256(from, spender, allowance - amount)?;
                }
                self.accounts.write_u256(
                    from_index,
                    StaticVftAccountMap::<ACCOUNT_LOG2_SLOTS>::BALANCE_OFFSET,
                    owner_balance - amount,
                );
                self.accounts.write_u256(
                    to_index,
                    StaticVftAccountMap::<ACCOUNT_LOG2_SLOTS>::BALANCE_OFFSET,
                    to_balance,
                );
            }
            Ok(true)
        }

        /// Clears both account and overflow allowance maps.
        pub fn clear(&self) -> Result<(), TableError> {
            self.accounts.clear()?;
            self.overflow_allowances.clear()
        }
    }

    #[cfg(feature = "experimental-vft-account")]
    impl LookupResult {
        fn found(self) -> Option<usize> {
            match self {
                Self::Found(index) => Some(index),
                Self::Vacant(_) | Self::Full => None,
            }
        }
    }

    /// Core VFT storage over WAT-shaped static balance and allowance maps.
    ///
    /// This is intended for high-cardinality token hot paths where transfer,
    /// transfer-from, and approve should avoid allocator-backed maps. It does
    /// not track metadata, events, total supply, allowance expiry, or holder
    /// listing indexes; keep those outside this transfer-shaped storage layer.
    pub struct StaticVftStorage<const BALANCE_LOG2_SLOTS: u8, const ALLOWANCE_LOG2_SLOTS: u8> {
        balances: VftBalances<BALANCE_LOG2_SLOTS>,
        allowances: VftAllowances<ALLOWANCE_LOG2_SLOTS>,
    }

    impl<const BALANCE_LOG2_SLOTS: u8, const ALLOWANCE_LOG2_SLOTS: u8>
        StaticVftStorage<BALANCE_LOG2_SLOTS, ALLOWANCE_LOG2_SLOTS>
    {
        /// Creates VFT storage over separately reserved static regions.
        ///
        /// # Safety
        ///
        /// The caller must ensure both memory intervals are valid for reads and
        /// writes for the whole lifetime of the storage and do not overlap other
        /// mutable state.
        pub unsafe fn new(balance_base: usize, allowance_base: usize) -> Result<Self, TableError> {
            let balance_region = StaticRegion::new(
                balance_base,
                VftBalances::<BALANCE_LOG2_SLOTS>::bytes_len()?,
            )?;
            let allowance_region = StaticRegion::new(
                allowance_base,
                VftAllowances::<ALLOWANCE_LOG2_SLOTS>::bytes_len()?,
            )?;
            if regions_overlap(balance_region, allowance_region)? {
                return Err(TableError::InvalidLayout);
            }

            Ok(Self {
                balances: unsafe { VftBalances::<BALANCE_LOG2_SLOTS>::new(balance_base)? },
                allowances: unsafe { VftAllowances::<ALLOWANCE_LOG2_SLOTS>::new(allowance_base)? },
            })
        }

        /// Returns the balance map backing this storage.
        pub fn balances(&self) -> &VftBalances<BALANCE_LOG2_SLOTS> {
            &self.balances
        }

        /// Returns the allowance map backing this storage.
        pub fn allowances(&self) -> &VftAllowances<ALLOWANCE_LOG2_SLOTS> {
            &self.allowances
        }

        /// Returns the visible balance of `account`.
        pub fn balance_of(&self, account: ActorId) -> Result<U256, TableError> {
            Ok(self
                .balances
                .get_actor_u256(&account)?
                .unwrap_or_else(U256::zero))
        }

        /// Returns the visible allowance from `owner` to `spender`.
        pub fn allowance(&self, owner: ActorId, spender: ActorId) -> Result<U256, TableError> {
            Ok(self
                .allowances
                .get_allowance_u256(&owner, &spender)?
                .unwrap_or_else(U256::zero))
        }

        /// Sets an allowance and returns whether the visible value changed.
        pub fn approve(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<bool, TableError> {
            if owner == spender {
                return Ok(false);
            }

            let previous = self
                .allowances
                .insert_allowance_u256(owner, spender, value)?
                .unwrap_or_else(U256::zero);
            Ok(previous != value)
        }

        /// Adds `amount` to `to` and returns whether state changed.
        pub fn mint(&self, to: ActorId, amount: U256) -> Result<bool, TableError> {
            if amount.is_zero() {
                return Ok(false);
            }

            let previous = self.balance_of(to)?;
            let (next, overflow) = previous.overflowing_add(amount);
            if overflow {
                return Ok(false);
            }

            self.balances.insert_actor_u256(to, next)?;
            Ok(true)
        }

        /// Removes `amount` from `from` and returns whether state changed.
        pub fn burn(&self, from: ActorId, amount: U256) -> Result<bool, TableError> {
            if amount.is_zero() {
                return Ok(false);
            }

            let previous = self.balance_of(from)?;
            if previous < amount {
                return Ok(false);
            }

            self.balances.insert_actor_u256(from, previous - amount)?;
            Ok(true)
        }

        /// Moves `amount` from `from` to `to` and returns whether state changed.
        pub fn transfer(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            if from == to || amount.is_zero() {
                return Ok(false);
            }

            Ok(self
                .balances
                .transfer_actor_u256(from, to, amount)?
                .is_some())
        }

        /// Moves `amount` after validating and decreasing `spender` allowance.
        pub fn transfer_from(
            &self,
            spender: ActorId,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<bool, TableError> {
            if spender == from {
                return self.transfer(from, to, amount);
            }
            if from == to || amount.is_zero() {
                return Ok(false);
            }

            Ok(self
                .balances
                .transfer_actor_u256_from(&self.allowances, from, spender, to, amount)?
                .is_some())
        }

        /// Clears both balance and allowance maps.
        pub fn clear(&self) -> Result<(), TableError> {
            self.balances.clear()?;
            self.allowances.clear()
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum LookupResult {
        Found(usize),
        Vacant(usize),
        Full,
    }

    fn static_map_slots(log2_slots: u8) -> Result<usize, TableError> {
        let shift = u32::from(log2_slots);
        if shift > 32 || shift >= usize::BITS {
            return Err(TableError::InvalidLayout);
        }

        1usize.checked_shl(shift).ok_or(TableError::InvalidLayout)
    }

    fn static_map_index(hash: u32, log2_slots: u8) -> usize {
        if log2_slots == 0 {
            0
        } else {
            (hash >> (32 - u32::from(log2_slots))) as usize
        }
    }

    fn hash_actor_key(key: &[u8; 32]) -> u32 {
        hash_words(key)
    }

    fn actor_tag(actor: ActorId) -> u64 {
        let key = actor_key(actor);
        let lane = unsafe { ptr::read_unaligned(key.as_ptr().add(12).cast::<u64>()) };
        let tag = if lane != 0 {
            lane
        } else {
            u64::from(fmix32(fold_words(&key)))
                | (u64::from(fmix32(fold_words_rotated(&key))) << 32)
        };
        if tag == 0 { 1 } else { tag }
    }

    fn allowance_tag(owner: ActorId, spender: ActorId) -> u64 {
        let owner = actor_tag(owner);
        let spender = actor_tag(spender);
        let tag = owner
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(spender.wrapping_mul(0xBF58_476D_1CE4_E5B9));
        if tag == 0 { 1 } else { tag }
    }

    fn hash_allowance_key(owner: &[u8; 32], spender: &[u8; 32]) -> u32 {
        fold_words(owner)
            .wrapping_mul(HASH_GOLDEN_RATIO)
            .wrapping_add(fold_words(spender).wrapping_mul(HASH_PAIR_SPENDER))
    }

    const HASH_GOLDEN_RATIO: u32 = 0x9E37_79B9;
    const HASH_PAIR_SPENDER: u32 = 0x85EB_CA6B;

    fn hash_words(bytes: &[u8; 32]) -> u32 {
        fold_words(bytes).wrapping_mul(HASH_GOLDEN_RATIO)
    }

    fn fmix32(mut hash: u32) -> u32 {
        hash ^= hash >> 16;
        hash = hash.wrapping_mul(0x85EB_CA6B);
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(0xC2B2_AE35);
        hash ^ (hash >> 16)
    }

    fn fold_words(bytes: &[u8; 32]) -> u32 {
        let ptr = bytes.as_ptr();
        unsafe {
            ptr::read_unaligned(ptr.cast::<u32>())
                ^ ptr::read_unaligned(ptr.add(4).cast::<u32>())
                ^ ptr::read_unaligned(ptr.add(8).cast::<u32>())
                ^ ptr::read_unaligned(ptr.add(12).cast::<u32>())
                ^ ptr::read_unaligned(ptr.add(16).cast::<u32>())
                ^ ptr::read_unaligned(ptr.add(20).cast::<u32>())
                ^ ptr::read_unaligned(ptr.add(24).cast::<u32>())
                ^ ptr::read_unaligned(ptr.add(28).cast::<u32>())
        }
    }

    fn fold_words_rotated(bytes: &[u8; 32]) -> u32 {
        let ptr = bytes.as_ptr();
        unsafe {
            ptr::read_unaligned(ptr.cast::<u32>()).rotate_left(5)
                ^ ptr::read_unaligned(ptr.add(4).cast::<u32>()).rotate_left(9)
                ^ ptr::read_unaligned(ptr.add(8).cast::<u32>()).rotate_left(13)
                ^ ptr::read_unaligned(ptr.add(12).cast::<u32>()).rotate_left(17)
                ^ ptr::read_unaligned(ptr.add(16).cast::<u32>()).rotate_left(21)
                ^ ptr::read_unaligned(ptr.add(20).cast::<u32>()).rotate_left(25)
                ^ ptr::read_unaligned(ptr.add(24).cast::<u32>()).rotate_left(29)
                ^ ptr::read_unaligned(ptr.add(28).cast::<u32>()).rotate_left(3)
        }
    }

    fn reject_zero_key(bytes: &[u8; 32]) -> Result<(), TableError> {
        if is_zero_32(bytes) {
            Err(TableError::InvalidKey)
        } else {
            Ok(())
        }
    }

    fn control_fingerprint(hash: u32) -> u8 {
        ((hash as u8) & 0x7f) + 1
    }

    fn regions_overlap(left: StaticRegion, right: StaticRegion) -> Result<bool, TableError> {
        Ok(left.base() < right.end()? && right.base() < left.end()?)
    }

    fn words_32(bytes: &[u8; 32]) -> [u64; 4] {
        [
            u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            u64::from_le_bytes([
                bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ]),
            u64::from_le_bytes([
                bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22],
                bytes[23],
            ]),
            u64::from_le_bytes([
                bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30],
                bytes[31],
            ]),
        ]
    }

    unsafe fn read_words_32(ptr: *const u8) -> [u64; 4] {
        unsafe {
            [
                ptr::read_unaligned(ptr.cast::<u64>()),
                ptr::read_unaligned(ptr.add(8).cast::<u64>()),
                ptr::read_unaligned(ptr.add(16).cast::<u64>()),
                ptr::read_unaligned(ptr.add(24).cast::<u64>()),
            ]
        }
    }

    unsafe fn words_at_zero(ptr: *const u8) -> bool {
        words_are_zero(unsafe { read_words_32(ptr) })
    }

    fn words_are_zero(words: [u64; 4]) -> bool {
        (words[0] | words[1] | words[2] | words[3]) == 0
    }

    fn is_zero_32(bytes: &[u8; 32]) -> bool {
        bytes.iter().all(|byte| *byte == 0)
    }

    impl<const CAP: usize> FixedBalanceMap<CAP> {
        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            self.get(&actor_key(*key))
                .map(|value| value.map(u256_from_value))
        }

        /// Inserts or updates a balance value by actor id.
        pub fn insert_actor_u256(
            &mut self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            self.insert(actor_key(key), u256_value(value))
                .map(|previous| previous.map(u256_from_value))
        }

        /// Removes a balance value by actor id.
        pub fn remove_actor_u256(&mut self, key: &ActorId) -> Result<Option<U256>, TableError> {
            self.remove(&actor_key(*key))
                .map(|previous| previous.map(u256_from_value))
        }
    }

    impl<const CAP: usize> FixedAllowanceMap<CAP> {
        /// Returns an allowance value by owner and spender.
        pub fn get_allowance_u256(
            &self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            self.get(&allowance_key(*owner, *spender))
                .map(|value| value.map(u256_from_value))
        }

        /// Inserts or updates an allowance value by owner and spender.
        pub fn insert_allowance_u256(
            &mut self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            self.insert(allowance_key(owner, spender), u256_value(value))
                .map(|previous| previous.map(u256_from_value))
        }

        /// Removes an allowance value by owner and spender.
        pub fn remove_allowance_u256(
            &mut self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            self.remove(&allowance_key(*owner, *spender))
                .map(|previous| previous.map(u256_from_value))
        }
    }

    impl StaticBalanceTable {
        /// Returns a balance value by actor id.
        pub fn get_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            self.get(&actor_key(*key))
                .map(|value| value.map(u256_from_value))
        }

        /// Inserts or updates a balance value by actor id.
        pub fn insert_actor_u256(
            &self,
            key: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            self.insert(&actor_key(key), &u256_value(value))
                .map(|previous| previous.map(u256_from_value))
        }

        /// Removes a balance value by actor id.
        pub fn remove_actor_u256(&self, key: &ActorId) -> Result<Option<U256>, TableError> {
            self.remove(&actor_key(*key))
                .map(|previous| previous.map(u256_from_value))
        }

        /// Moves `amount` from one actor balance to another using one lookup per key.
        pub fn transfer_actor_u256(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<StaticActorU256Transfer>, TableError> {
            let from_key = actor_key(from);
            let to_key = actor_key(to);
            if amount.is_zero() {
                let from_balance = self.get_actor_u256(&from)?.unwrap_or_else(U256::zero);
                let to_balance = if from_key == to_key {
                    from_balance
                } else {
                    self.get_actor_u256(&to)?.unwrap_or_else(U256::zero)
                };
                return Ok(Some(StaticActorU256Transfer {
                    from_balance,
                    to_balance,
                    inserted_to: false,
                }));
            }

            let Lookup::Found(from_index) = self.lookup(&from_key)? else {
                return Ok(None);
            };
            let from_value = u256_from_value(unsafe { self.read_value(from_index)? });
            if from_value < amount {
                return Ok(None);
            }
            if from_key == to_key {
                return Ok(Some(StaticActorU256Transfer {
                    from_balance: from_value,
                    to_balance: from_value,
                    inserted_to: false,
                }));
            }

            let to_lookup = self.lookup(&to_key)?;
            let (to_value, inserted_to) = match to_lookup {
                Lookup::Found(index) => {
                    (u256_from_value(unsafe { self.read_value(index)? }), false)
                }
                Lookup::Vacant(_) => (U256::zero(), true),
                Lookup::Full => return Ok(None),
            };
            let (to_value, overflow) = to_value.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let from_value = from_value - amount;
            let from_value_bytes = u256_value(from_value);
            let to_value_bytes = u256_value(to_value);
            unsafe {
                self.write_value(from_index, &from_value_bytes);
                match to_lookup {
                    Lookup::Found(index) => self.write_value(index, &to_value_bytes),
                    Lookup::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_value_bytes);
                        self.write_state(index, SlotState::Full);
                    }
                    Lookup::Full => unreachable!("full table handled before writes"),
                }
            }

            Ok(Some(StaticActorU256Transfer {
                from_balance: from_value,
                to_balance: to_value,
                inserted_to,
            }))
        }

        /// Moves `amount` after validating and decreasing a static-table allowance.
        pub fn transfer_actor_u256_from(
            &self,
            allowances: &StaticAllowanceTable,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<StaticActorU256TransferFrom>, TableError> {
            let owner_key = actor_key(owner);
            let to_key = actor_key(to);
            if amount.is_zero() {
                let from_balance = self.get_actor_u256(&owner)?.unwrap_or_else(U256::zero);
                let to_balance = if owner_key == to_key {
                    from_balance
                } else {
                    self.get_actor_u256(&to)?.unwrap_or_else(U256::zero)
                };
                let allowance = allowances
                    .get_allowance_u256(&owner, &spender)?
                    .unwrap_or_else(U256::zero);
                return Ok(Some(StaticActorU256TransferFrom {
                    from_balance,
                    to_balance,
                    allowance,
                    inserted_to: false,
                }));
            }

            let allowance_key = allowance_key(owner, spender);
            let Lookup::Found(allowance_index) = allowances.lookup(&allowance_key)? else {
                return Ok(None);
            };
            let allowance = u256_from_value(unsafe { allowances.read_value(allowance_index)? });
            if allowance < amount {
                return Ok(None);
            }

            let Lookup::Found(from_index) = self.lookup(&owner_key)? else {
                return Ok(None);
            };
            let from_balance = u256_from_value(unsafe { self.read_value(from_index)? });
            if from_balance < amount {
                return Ok(None);
            }

            let allowance = allowance - amount;
            if owner_key == to_key {
                let allowance_bytes = u256_value(allowance);
                unsafe {
                    allowances.write_value(allowance_index, &allowance_bytes);
                }
                return Ok(Some(StaticActorU256TransferFrom {
                    from_balance,
                    to_balance: from_balance,
                    allowance,
                    inserted_to: false,
                }));
            }

            let to_lookup = self.lookup(&to_key)?;
            let (to_balance, inserted_to) = match to_lookup {
                Lookup::Found(index) => {
                    (u256_from_value(unsafe { self.read_value(index)? }), false)
                }
                Lookup::Vacant(_) => (U256::zero(), true),
                Lookup::Full => return Ok(None),
            };
            let (to_balance, overflow) = to_balance.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let from_balance = from_balance - amount;
            let from_balance_bytes = u256_value(from_balance);
            let to_balance_bytes = u256_value(to_balance);
            let allowance_bytes = u256_value(allowance);
            unsafe {
                self.write_value(from_index, &from_balance_bytes);
                match to_lookup {
                    Lookup::Found(index) => self.write_value(index, &to_balance_bytes),
                    Lookup::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_balance_bytes);
                        self.write_state(index, SlotState::Full);
                    }
                    Lookup::Full => unreachable!("full table handled before writes"),
                }
                allowances.write_value(allowance_index, &allowance_bytes);
            }

            Ok(Some(StaticActorU256TransferFrom {
                from_balance,
                to_balance,
                allowance,
                inserted_to,
            }))
        }

        /// Moves a known nonzero amount between known distinct, nonzero actors.
        ///
        /// # Safety
        ///
        /// The caller must ensure `from` and `to` are nonzero actor ids, `from != to`,
        /// and `amount != 0`. This skips validation intended for the safe public path.
        pub unsafe fn transfer_actor_u256_nonzero_distinct_unchecked(
            &self,
            from: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<StaticActorU256Transfer>, TableError> {
            let from_key = actor_key(from);
            let to_key = actor_key(to);

            let Lookup::Found(from_index) = self.lookup(&from_key)? else {
                return Ok(None);
            };
            let from_value = u256_from_value(unsafe { self.read_value(from_index)? });
            if from_value < amount {
                return Ok(None);
            }

            let to_lookup = self.lookup(&to_key)?;
            let (to_value, inserted_to) = match to_lookup {
                Lookup::Found(index) => {
                    (u256_from_value(unsafe { self.read_value(index)? }), false)
                }
                Lookup::Vacant(_) => (U256::zero(), true),
                Lookup::Full => return Ok(None),
            };
            let (to_value, overflow) = to_value.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let from_value = from_value - amount;
            let from_value_bytes = u256_value(from_value);
            let to_value_bytes = u256_value(to_value);
            unsafe {
                self.write_value(from_index, &from_value_bytes);
                match to_lookup {
                    Lookup::Found(index) => self.write_value(index, &to_value_bytes),
                    Lookup::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_value_bytes);
                        self.write_state(index, SlotState::Full);
                    }
                    Lookup::Full => unreachable!("full table handled before writes"),
                }
            }

            Ok(Some(StaticActorU256Transfer {
                from_balance: from_value,
                to_balance: to_value,
                inserted_to,
            }))
        }

        /// Moves a known nonzero amount after checking allowance for known distinct, nonzero actors.
        ///
        /// # Safety
        ///
        /// The caller must ensure `owner`, `spender`, and `to` are nonzero actor ids,
        /// `owner != to`, and `amount != 0`. This skips validation intended for the
        /// safe public path.
        pub unsafe fn transfer_actor_u256_from_nonzero_distinct_unchecked(
            &self,
            allowances: &StaticAllowanceTable,
            owner: ActorId,
            spender: ActorId,
            to: ActorId,
            amount: U256,
        ) -> Result<Option<StaticActorU256TransferFrom>, TableError> {
            let owner_key = actor_key(owner);
            let to_key = actor_key(to);
            let allowance_key = allowance_key(owner, spender);

            let Lookup::Found(allowance_index) = allowances.lookup(&allowance_key)? else {
                return Ok(None);
            };
            let allowance = u256_from_value(unsafe { allowances.read_value(allowance_index)? });
            if allowance < amount {
                return Ok(None);
            }

            let Lookup::Found(from_index) = self.lookup(&owner_key)? else {
                return Ok(None);
            };
            let from_balance = u256_from_value(unsafe { self.read_value(from_index)? });
            if from_balance < amount {
                return Ok(None);
            }

            let to_lookup = self.lookup(&to_key)?;
            let (to_balance, inserted_to) = match to_lookup {
                Lookup::Found(index) => {
                    (u256_from_value(unsafe { self.read_value(index)? }), false)
                }
                Lookup::Vacant(_) => (U256::zero(), true),
                Lookup::Full => return Ok(None),
            };
            let (to_balance, overflow) = to_balance.overflowing_add(amount);
            if overflow {
                return Ok(None);
            }

            let allowance = allowance - amount;
            let from_balance = from_balance - amount;
            let from_balance_bytes = u256_value(from_balance);
            let to_balance_bytes = u256_value(to_balance);
            let allowance_bytes = u256_value(allowance);
            unsafe {
                self.write_value(from_index, &from_balance_bytes);
                match to_lookup {
                    Lookup::Found(index) => self.write_value(index, &to_balance_bytes),
                    Lookup::Vacant(index) => {
                        self.write_key(index, &to_key);
                        self.write_value(index, &to_balance_bytes);
                        self.write_state(index, SlotState::Full);
                    }
                    Lookup::Full => unreachable!("full table handled before writes"),
                }
                allowances.write_value(allowance_index, &allowance_bytes);
            }

            Ok(Some(StaticActorU256TransferFrom {
                from_balance,
                to_balance,
                allowance,
                inserted_to,
            }))
        }
    }

    impl StaticAllowanceTable {
        /// Returns an allowance value by owner and spender.
        pub fn get_allowance_u256(
            &self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            self.get(&allowance_key(*owner, *spender))
                .map(|value| value.map(u256_from_value))
        }

        /// Inserts or updates an allowance value by owner and spender.
        pub fn insert_allowance_u256(
            &self,
            owner: ActorId,
            spender: ActorId,
            value: U256,
        ) -> Result<Option<U256>, TableError> {
            self.insert(&allowance_key(owner, spender), &u256_value(value))
                .map(|previous| previous.map(u256_from_value))
        }

        /// Removes an allowance value by owner and spender.
        pub fn remove_allowance_u256(
            &self,
            owner: &ActorId,
            spender: &ActorId,
        ) -> Result<Option<U256>, TableError> {
            self.remove(&allowance_key(*owner, *spender))
                .map(|previous| previous.map(u256_from_value))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::gear::{
        StaticActorU256Transfer, StaticActorU256TransferFrom, StaticAllowanceTable,
        StaticBalanceTable,
    };
    use gprimitives::{ActorId, U256};
    use std::{vec, vec::Vec};

    type Fixed = FixedOpenAddressMap<1, 1, 2>;

    fn colliding_keys(slots: usize) -> ([u8; 1], [u8; 1]) {
        for left in 0u8..=u8::MAX {
            for right in left.wrapping_add(1)..=u8::MAX {
                if hash_bytes(&[left]) % slots == hash_bytes(&[right]) % slots {
                    return ([left], [right]);
                }
            }
        }

        unreachable!("single-byte key space must contain collisions")
    }

    #[test]
    fn fixed_map_updates_existing_key() {
        let mut map = Fixed::new();

        assert_eq!(map.insert([0], [0]), Ok(None));
        assert_eq!(map.insert([0], [7]), Ok(Some([0])));
        assert_eq!(map.get(&[0]), Ok(Some([7])));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn fixed_map_reuses_tombstone() {
        let (first, second) = colliding_keys(1);
        let mut map = FixedOpenAddressMap::<1, 1, 1>::new();

        assert_eq!(map.insert(first, [1]), Ok(None));
        assert_eq!(map.remove(&first), Ok(Some([1])));
        assert_eq!(map.insert(second, [2]), Ok(None));
        assert_eq!(map.get(&second), Ok(Some([2])));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn fixed_map_reports_full_table() {
        let mut map = FixedOpenAddressMap::<1, 1, 1>::new();

        assert_eq!(map.insert([1], [1]), Ok(None));
        assert_eq!(map.insert([2], [2]), Err(TableError::CapacityOverflow));
    }

    #[test]
    fn fixed_map_stores_zero_key_and_zero_value() {
        let mut map = Fixed::new();

        assert_eq!(map.insert([0], [0]), Ok(None));
        assert_eq!(map.get(&[0]), Ok(Some([0])));
        assert_eq!(map.remove(&[0]), Ok(Some([0])));
        assert_eq!(map.get(&[0]), Ok(None));
    }

    #[test]
    fn static_table_updates_existing_key() {
        let mut memory = [0u8; 8];
        let table = unsafe {
            StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 2).unwrap()
        };

        assert_eq!(table.insert(&[0], &[0]), Ok(None));
        assert_eq!(table.insert(&[0], &[7]), Ok(Some([0])));
        assert_eq!(table.get(&[0]), Ok(Some([7])));
    }

    #[test]
    fn static_table_reuses_tombstone() {
        let (first, second) = colliding_keys(1);
        let mut memory = [0u8; 3];
        let table = unsafe {
            StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 1).unwrap()
        };

        assert_eq!(table.insert(&first, &[1]), Ok(None));
        assert_eq!(table.remove(&first), Ok(Some([1])));
        assert_eq!(table.insert(&second, &[2]), Ok(None));
        assert_eq!(table.get(&second), Ok(Some([2])));
    }

    #[test]
    fn static_table_reports_full_table() {
        let mut memory = [0u8; 3];
        let table = unsafe {
            StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 1).unwrap()
        };

        assert_eq!(table.insert(&[1], &[1]), Ok(None));
        assert_eq!(table.insert(&[2], &[2]), Err(TableError::CapacityOverflow));
    }

    #[test]
    fn static_table_reports_invalid_slot_state() {
        let mut memory = [9u8, 0, 0];
        let table = unsafe {
            StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 1).unwrap()
        };

        assert_eq!(table.get(&[0]), Err(TableError::InvalidSlotState));
    }

    #[test]
    fn static_table_clear_resets_deleted_slots() {
        let mut memory = [0u8; 3];
        let table = unsafe {
            StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 1).unwrap()
        };

        table.insert(&[1], &[1]).unwrap();
        table.remove(&[1]).unwrap();
        table.clear().unwrap();
        assert_eq!(memory, [0, 0, 0]);
    }

    #[test]
    fn static_balance_table_transfers_without_result_rereads() {
        let mut memory = vec![0u8; StaticBalanceTable::bytes_len(4).unwrap()];
        let table = unsafe { StaticBalanceTable::new(memory.as_mut_ptr() as usize, 4).unwrap() };
        let from = ActorId::from(1u64);
        let to = ActorId::from(2u64);

        assert_eq!(table.insert_actor_u256(from, U256::from(10)), Ok(None));
        assert_eq!(
            table.transfer_actor_u256(from, to, U256::from(3)),
            Ok(Some(StaticActorU256Transfer {
                from_balance: U256::from(7),
                to_balance: U256::from(3),
                inserted_to: true,
            }))
        );
        assert_eq!(table.get_actor_u256(&from), Ok(Some(U256::from(7))));
        assert_eq!(table.get_actor_u256(&to), Ok(Some(U256::from(3))));
        assert_eq!(
            table.transfer_actor_u256(from, to, U256::from(2)),
            Ok(Some(StaticActorU256Transfer {
                from_balance: U256::from(5),
                to_balance: U256::from(5),
                inserted_to: false,
            }))
        );
        assert_eq!(table.transfer_actor_u256(from, to, U256::from(8)), Ok(None));
        assert_eq!(table.get_actor_u256(&from), Ok(Some(U256::from(5))));
        assert_eq!(table.get_actor_u256(&to), Ok(Some(U256::from(5))));
    }

    #[test]
    fn static_balance_table_transfer_rejects_full_recipient_without_mutating() {
        let mut memory = vec![0u8; StaticBalanceTable::bytes_len(1).unwrap()];
        let table = unsafe { StaticBalanceTable::new(memory.as_mut_ptr() as usize, 1).unwrap() };
        let from = ActorId::from(1u64);
        let to = ActorId::from(2u64);

        assert_eq!(table.insert_actor_u256(from, U256::from(10)), Ok(None));
        assert_eq!(table.transfer_actor_u256(from, to, U256::from(3)), Ok(None));
        assert_eq!(table.get_actor_u256(&from), Ok(Some(U256::from(10))));
        assert_eq!(table.get_actor_u256(&to), Ok(None));
    }

    #[test]
    fn static_balance_table_transfer_from_preserves_state_on_failure() {
        let mut balance_memory = vec![0u8; StaticBalanceTable::bytes_len(4).unwrap()];
        let balances =
            unsafe { StaticBalanceTable::new(balance_memory.as_mut_ptr() as usize, 4).unwrap() };
        let mut allowance_memory = vec![0u8; StaticAllowanceTable::bytes_len(4).unwrap()];
        let allowances = unsafe {
            StaticAllowanceTable::new(allowance_memory.as_mut_ptr() as usize, 4).unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);
        let to = ActorId::from(3u64);

        assert_eq!(balances.insert_actor_u256(owner, U256::from(10)), Ok(None));
        assert_eq!(
            allowances.insert_allowance_u256(owner, spender, U256::from(5)),
            Ok(None)
        );
        assert_eq!(
            balances.transfer_actor_u256_from(&allowances, owner, spender, to, U256::from(4)),
            Ok(Some(StaticActorU256TransferFrom {
                from_balance: U256::from(6),
                to_balance: U256::from(4),
                allowance: U256::from(1),
                inserted_to: true,
            }))
        );
        assert_eq!(balances.get_actor_u256(&owner), Ok(Some(U256::from(6))));
        assert_eq!(balances.get_actor_u256(&to), Ok(Some(U256::from(4))));
        assert_eq!(
            allowances.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(1)))
        );
        assert_eq!(
            balances.transfer_actor_u256_from(&allowances, owner, spender, to, U256::from(2)),
            Ok(None)
        );
        assert_eq!(balances.get_actor_u256(&owner), Ok(Some(U256::from(6))));
        assert_eq!(balances.get_actor_u256(&to), Ok(Some(U256::from(4))));
        assert_eq!(
            allowances.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(1)))
        );
    }

    #[test]
    fn static_balance_table_transfer_from_rejects_missing_allowance_without_mutating() {
        let mut balance_memory = vec![0u8; StaticBalanceTable::bytes_len(2).unwrap()];
        let balances =
            unsafe { StaticBalanceTable::new(balance_memory.as_mut_ptr() as usize, 2).unwrap() };
        let mut allowance_memory = vec![0u8; StaticAllowanceTable::bytes_len(1).unwrap()];
        let allowances = unsafe {
            StaticAllowanceTable::new(allowance_memory.as_mut_ptr() as usize, 1).unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);
        let to = ActorId::from(3u64);

        assert_eq!(balances.insert_actor_u256(owner, U256::from(10)), Ok(None));
        assert_eq!(
            balances.transfer_actor_u256_from(&allowances, owner, spender, to, U256::from(3)),
            Ok(None)
        );
        assert_eq!(balances.get_actor_u256(&owner), Ok(Some(U256::from(10))));
        assert_eq!(balances.get_actor_u256(&to), Ok(None));
        assert_eq!(allowances.get_allowance_u256(&owner, &spender), Ok(None));
    }

    #[test]
    fn static_layout_reserves_non_overlapping_regions() {
        let mut layout = StaticLayout::new(100, 16).unwrap();
        let table = layout.reserve_table::<1, 1>(2).unwrap();
        let value = layout.reserve_bytes(4).unwrap();

        assert_eq!(table.base(), 100);
        assert_eq!(table.region().bytes(), 6);
        assert_eq!(value.base(), 106);
        assert_eq!(value.bytes(), 4);
        assert_eq!(layout.remaining(), 6);
    }

    #[test]
    fn static_layout_reserves_aligned_regions() {
        let mut layout = StaticLayout::new(100, 32).unwrap();
        let value = layout.reserve_aligned_bytes(4, 16).unwrap();

        assert_eq!(value.base(), 112);
        assert_eq!(value.bytes(), 4);
        assert_eq!(layout.cursor(), 116);
        assert_eq!(
            layout.reserve_aligned_bytes(1, 3).err(),
            Some(TableError::InvalidLayout)
        );
    }

    #[test]
    fn static_layout_rejects_overflow_and_out_of_bounds() {
        assert_eq!(
            StaticLayout::new(usize::MAX, 1).err(),
            Some(TableError::InvalidLayout)
        );

        let mut layout = StaticLayout::new(0, 2).unwrap();
        assert_eq!(
            layout.reserve_table::<1, 1>(1).err(),
            Some(TableError::InvalidLayout)
        );
    }

    #[test]
    fn fixed_and_static_tables_match_representative_ops() {
        let mut fixed = FixedOpenAddressMap::<1, 1, 4>::new();
        let mut memory = [0u8; 12];
        let static_table = unsafe {
            StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 4).unwrap()
        };

        for (key, value) in [([0], [0]), ([1], [2]), ([2], [3])] {
            assert_eq!(fixed.insert(key, value), static_table.insert(&key, &value));
        }

        assert_eq!(fixed.remove(&[1]), static_table.remove(&[1]));
        assert_eq!(fixed.insert([3], [4]), static_table.insert(&[3], &[4]));

        for key in [[0], [1], [2], [3]] {
            assert_eq!(fixed.get(&key), static_table.get(&key));
        }
    }

    #[test]
    fn fixed_map_entries_return_visible_pairs_only() {
        let mut map = FixedOpenAddressMap::<1, 1, 4>::new();

        map.insert([1], [10]).unwrap();
        map.insert([2], [20]).unwrap();
        map.remove(&[1]).unwrap();
        map.insert([3], [30]).unwrap();

        let mut entries = map.entries().collect::<Vec<_>>();
        entries.sort_unstable_by_key(|(key, _)| *key);

        assert_eq!(entries, vec![([2], [20]), ([3], [30])]);
    }
}

#[cfg(test)]
mod gear_tests {
    extern crate std;

    use super::{PAGE_LOCAL_ACTOR_U256_TILE_BYTES, TableError, gear::*};
    use gprimitives::{ActorId, U256};
    use std::{ptr, vec, vec::Vec};

    #[test]
    fn static_actor_map_insert_update_and_remove() {
        let mut memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let table =
            unsafe { StaticActorIdU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let actor = ActorId::from(1u64);

        assert_eq!(table.get_actor_u256(&actor), Ok(None));
        assert_eq!(table.insert_actor_u256(actor, U256::from(10)), Ok(None));
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(10))));
        assert_eq!(
            table.insert_actor_u256(actor, U256::from(20)),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.remove_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_actor_map_zero_value_is_tombstone() {
        let mut memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let table =
            unsafe { StaticActorIdU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let actor = ActorId::from(2u64);

        assert_eq!(table.insert_actor_u256(actor, U256::from(7)), Ok(None));
        assert_eq!(
            table.insert_actor_u256(actor, U256::zero()),
            Ok(Some(U256::from(7)))
        );
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
        assert_eq!(table.insert_actor_u256(actor, U256::from(8)), Ok(None));
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(8))));
    }

    #[test]
    fn static_actor_map_transfers_balance_in_one_operation() {
        let mut memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let table =
            unsafe { StaticActorIdU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let from = ActorId::from(1u64);
        let to = ActorId::from(2u64);

        assert_eq!(table.insert_actor_u256(from, U256::from(10)), Ok(None));
        assert_eq!(
            table.transfer_actor_u256(from, to, U256::from(3)),
            Ok(Some(ActorU256Transfer {
                from_balance: U256::from(7),
                to_balance: U256::from(3),
            }))
        );
        assert_eq!(table.get_actor_u256(&from), Ok(Some(U256::from(7))));
        assert_eq!(table.get_actor_u256(&to), Ok(Some(U256::from(3))));
        assert_eq!(table.transfer_actor_u256(from, to, U256::from(8)), Ok(None));
        assert_eq!(table.get_actor_u256(&from), Ok(Some(U256::from(7))));
        assert_eq!(table.get_actor_u256(&to), Ok(Some(U256::from(3))));
    }

    #[test]
    fn static_actor_map_transfers_from_allowance_in_one_operation() {
        let mut balance_memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let balances = unsafe {
            StaticActorIdU256Map::<2>::new(balance_memory.as_mut_ptr() as usize).unwrap()
        };
        let mut allowance_memory = vec![0u8; StaticAllowanceU256Map::<2>::bytes_len().unwrap()];
        let allowances = unsafe {
            StaticAllowanceU256Map::<2>::new(allowance_memory.as_mut_ptr() as usize).unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);
        let to = ActorId::from(3u64);

        assert_eq!(balances.insert_actor_u256(owner, U256::from(10)), Ok(None));
        assert_eq!(
            allowances.insert_allowance_u256(owner, spender, U256::from(5)),
            Ok(None)
        );
        assert_eq!(
            balances.transfer_actor_u256_from(&allowances, owner, spender, to, U256::from(4)),
            Ok(Some(ActorU256TransferFrom {
                from_balance: U256::from(6),
                to_balance: U256::from(4),
                allowance: U256::from(1),
            }))
        );
        assert_eq!(balances.get_actor_u256(&owner), Ok(Some(U256::from(6))));
        assert_eq!(balances.get_actor_u256(&to), Ok(Some(U256::from(4))));
        assert_eq!(
            allowances.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(1)))
        );
    }

    #[test]
    fn static_actor_map_transfer_from_preserves_state_on_failure() {
        let mut balance_memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let balances = unsafe {
            StaticActorIdU256Map::<2>::new(balance_memory.as_mut_ptr() as usize).unwrap()
        };
        let mut allowance_memory = vec![0u8; StaticAllowanceU256Map::<2>::bytes_len().unwrap()];
        let allowances = unsafe {
            StaticAllowanceU256Map::<2>::new(allowance_memory.as_mut_ptr() as usize).unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);
        let to = ActorId::from(3u64);

        assert_eq!(balances.insert_actor_u256(owner, U256::from(3)), Ok(None));
        assert_eq!(
            allowances.insert_allowance_u256(owner, spender, U256::from(10)),
            Ok(None)
        );
        assert_eq!(
            balances.transfer_actor_u256_from(&allowances, owner, spender, to, U256::from(4)),
            Ok(None)
        );
        assert_eq!(balances.get_actor_u256(&owner), Ok(Some(U256::from(3))));
        assert_eq!(balances.get_actor_u256(&to), Ok(None));
        assert_eq!(
            allowances.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(10)))
        );

        assert_eq!(
            allowances.insert_allowance_u256(owner, spender, U256::from(2)),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(
            balances.transfer_actor_u256_from(&allowances, owner, spender, to, U256::from(3)),
            Ok(None)
        );
        assert_eq!(balances.get_actor_u256(&owner), Ok(Some(U256::from(3))));
        assert_eq!(balances.get_actor_u256(&to), Ok(None));
        assert_eq!(
            allowances.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(2)))
        );
    }

    #[test]
    fn static_vft_storage_handles_core_hot_path() {
        let mut balance_memory = vec![0u8; VftBalances::<3>::bytes_len().unwrap()];
        let mut allowance_memory = vec![0u8; VftAllowances::<3>::bytes_len().unwrap()];
        let storage = unsafe {
            StaticVftStorage::<3, 3>::new(
                balance_memory.as_mut_ptr() as usize,
                allowance_memory.as_mut_ptr() as usize,
            )
            .unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);
        let to = ActorId::from(3u64);

        assert_eq!(storage.mint(owner, U256::from(10)), Ok(true));
        assert_eq!(storage.transfer(owner, to, U256::from(3)), Ok(true));
        assert_eq!(storage.balance_of(owner), Ok(U256::from(7)));
        assert_eq!(storage.balance_of(to), Ok(U256::from(3)));
        assert_eq!(storage.transfer(owner, owner, U256::from(1)), Ok(false));
        assert_eq!(storage.transfer(owner, to, U256::zero()), Ok(false));

        assert_eq!(storage.approve(owner, spender, U256::from(5)), Ok(true));
        assert_eq!(storage.approve(owner, spender, U256::from(5)), Ok(false));
        assert_eq!(storage.allowance(owner, spender), Ok(U256::from(5)));
        assert_eq!(
            storage.transfer_from(spender, owner, to, U256::from(4)),
            Ok(true)
        );
        assert_eq!(storage.balance_of(owner), Ok(U256::from(3)));
        assert_eq!(storage.balance_of(to), Ok(U256::from(7)));
        assert_eq!(storage.allowance(owner, spender), Ok(U256::from(1)));

        assert_eq!(storage.burn(to, U256::from(7)), Ok(true));
        assert_eq!(storage.balance_of(to), Ok(U256::zero()));
    }

    #[test]
    fn static_vft_storage_preserves_state_on_failed_transfer_from() {
        let mut balance_memory = vec![0u8; VftBalances::<3>::bytes_len().unwrap()];
        let mut allowance_memory = vec![0u8; VftAllowances::<3>::bytes_len().unwrap()];
        let storage = unsafe {
            StaticVftStorage::<3, 3>::new(
                balance_memory.as_mut_ptr() as usize,
                allowance_memory.as_mut_ptr() as usize,
            )
            .unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);
        let to = ActorId::from(3u64);

        assert_eq!(storage.mint(owner, U256::from(3)), Ok(true));
        assert_eq!(storage.approve(owner, spender, U256::from(10)), Ok(true));
        assert_eq!(
            storage.transfer_from(spender, owner, to, U256::from(4)),
            Ok(false)
        );
        assert_eq!(storage.balance_of(owner), Ok(U256::from(3)));
        assert_eq!(storage.balance_of(to), Ok(U256::zero()));
        assert_eq!(storage.allowance(owner, spender), Ok(U256::from(10)));

        assert_eq!(storage.approve(owner, spender, U256::from(2)), Ok(true));
        assert_eq!(
            storage.transfer_from(spender, owner, to, U256::from(3)),
            Ok(false)
        );
        assert_eq!(storage.balance_of(owner), Ok(U256::from(3)));
        assert_eq!(storage.balance_of(to), Ok(U256::zero()));
        assert_eq!(storage.allowance(owner, spender), Ok(U256::from(2)));
    }

    #[cfg(feature = "experimental-vft-account")]
    #[test]
    fn static_vft_account_storage_handles_inline_and_overflow_allowances() {
        let mut account_memory = vec![0u8; StaticVftAccountMap::<3>::bytes_len().unwrap()];
        let mut overflow_memory = vec![0u8; VftAllowances::<3>::bytes_len().unwrap()];
        let storage = unsafe {
            StaticVftAccountStorage::<3, 3>::new(
                account_memory.as_mut_ptr() as usize,
                overflow_memory.as_mut_ptr() as usize,
            )
            .unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender_a = ActorId::from(2u64);
        let spender_b = ActorId::from(3u64);
        let spender_c = ActorId::from(4u64);
        let to = ActorId::from(5u64);

        assert_eq!(storage.mint(owner, U256::from(20)), Ok(true));
        assert_eq!(storage.approve(owner, spender_a, U256::from(7)), Ok(true));
        assert_eq!(storage.approve(owner, spender_b, U256::from(8)), Ok(true));
        assert_eq!(storage.approve(owner, spender_c, U256::from(9)), Ok(true));
        assert_eq!(storage.allowance(owner, spender_a), Ok(U256::from(7)));
        assert_eq!(storage.allowance(owner, spender_b), Ok(U256::from(8)));
        assert_eq!(storage.allowance(owner, spender_c), Ok(U256::from(9)));

        assert_eq!(
            storage.transfer_from(spender_a, owner, to, U256::from(3)),
            Ok(true)
        );
        assert_eq!(storage.allowance(owner, spender_a), Ok(U256::from(4)));
        assert_eq!(
            storage.transfer_from(spender_c, owner, to, U256::from(4)),
            Ok(true)
        );
        assert_eq!(storage.allowance(owner, spender_c), Ok(U256::from(5)));
        assert_eq!(storage.balance_of(owner), Ok(U256::from(13)));
        assert_eq!(storage.balance_of(to), Ok(U256::from(7)));
    }

    #[cfg(feature = "experimental-vft-account")]
    #[test]
    fn static_vft_account_storage_reuses_cleared_inline_allowance_slot() {
        let mut account_memory = vec![0u8; StaticVftAccountMap::<2>::bytes_len().unwrap()];
        let mut overflow_memory = vec![0u8; VftAllowances::<2>::bytes_len().unwrap()];
        let storage = unsafe {
            StaticVftAccountStorage::<2, 2>::new(
                account_memory.as_mut_ptr() as usize,
                overflow_memory.as_mut_ptr() as usize,
            )
            .unwrap()
        };
        let owner = ActorId::from(1u64);
        let spender_a = ActorId::from(2u64);
        let spender_b = ActorId::from(3u64);
        let spender_c = ActorId::from(4u64);

        assert_eq!(storage.approve(owner, spender_a, U256::from(1)), Ok(true));
        assert_eq!(storage.approve(owner, spender_b, U256::from(2)), Ok(true));
        assert_eq!(storage.approve(owner, spender_a, U256::zero()), Ok(true));
        assert_eq!(storage.allowance(owner, spender_a), Ok(U256::zero()));
        assert_eq!(storage.approve(owner, spender_c, U256::from(3)), Ok(true));
        assert_eq!(storage.allowance(owner, spender_c), Ok(U256::from(3)));
        assert_eq!(
            storage
                .overflow_allowances()
                .get_allowance_u256(&owner, &spender_c),
            Ok(None)
        );
    }

    #[cfg(feature = "experimental-vft-account")]
    #[test]
    fn static_vft_account_storage_preserves_probe_chain_on_collisions() {
        let (first, second) = colliding_vft_account_pair::<1>();
        let mut account_memory = vec![0u8; StaticVftAccountMap::<1>::bytes_len().unwrap()];
        let mut overflow_memory = vec![0u8; VftAllowances::<1>::bytes_len().unwrap()];
        let storage = unsafe {
            StaticVftAccountStorage::<1, 1>::new(
                account_memory.as_mut_ptr() as usize,
                overflow_memory.as_mut_ptr() as usize,
            )
            .unwrap()
        };

        assert_eq!(storage.mint(first, U256::from(11)), Ok(true));
        assert_eq!(storage.mint(second, U256::from(13)), Ok(true));
        assert_eq!(storage.balance_of(first), Ok(U256::from(11)));
        assert_eq!(storage.balance_of(second), Ok(U256::from(13)));
    }

    #[test]
    fn static_actor_map_tombstone_preserves_probe_chain() {
        let (first, second) = colliding_actor_pair::<1>();
        let mut memory = vec![0u8; StaticActorIdU256Map::<1>::bytes_len().unwrap()];
        let table =
            unsafe { StaticActorIdU256Map::<1>::new(memory.as_mut_ptr() as usize).unwrap() };

        assert_eq!(table.insert_actor_u256(first, U256::from(1)), Ok(None));
        assert_eq!(table.insert_actor_u256(second, U256::from(2)), Ok(None));
        assert_eq!(table.remove_actor_u256(&first), Ok(Some(U256::from(1))));
        assert_eq!(table.get_actor_u256(&second), Ok(Some(U256::from(2))));
    }

    #[test]
    fn static_actor_map_rejects_zero_actor_mutation() {
        let mut memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let table =
            unsafe { StaticActorIdU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let actor = ActorId::zero();

        assert_eq!(
            table.insert_actor_u256(actor, U256::from(1)),
            Err(TableError::InvalidKey)
        );
        assert_eq!(table.remove_actor_u256(&actor), Err(TableError::InvalidKey));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_control_actor_map_insert_update_and_remove() {
        let (mut control, mut slots) = control_actor_memory::<2>();
        let table = unsafe {
            StaticControlActorIdU256Map::<2>::new(
                control.as_mut_ptr() as usize,
                slots.as_mut_ptr() as usize,
            )
            .unwrap()
        };
        let actor = ActorId::from(1u64);

        assert_eq!(table.get_actor_u256(&actor), Ok(None));
        assert_eq!(table.insert_actor_u256(actor, U256::from(10)), Ok(None));
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(10))));
        assert_eq!(
            table.insert_actor_u256(actor, U256::from(20)),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.remove_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_control_actor_map_zero_value_removes_and_reuses_deleted_slot() {
        let (first, second) = colliding_actor_pair::<1>();
        let (mut control, mut slots) = control_actor_memory::<1>();
        let table = unsafe {
            StaticControlActorIdU256Map::<1>::new(
                control.as_mut_ptr() as usize,
                slots.as_mut_ptr() as usize,
            )
            .unwrap()
        };

        assert_eq!(table.insert_actor_u256(first, U256::from(7)), Ok(None));
        assert_eq!(
            table.insert_actor_u256(first, U256::zero()),
            Ok(Some(U256::from(7)))
        );
        assert_eq!(table.get_actor_u256(&first), Ok(None));
        assert_eq!(table.insert_actor_u256(second, U256::from(8)), Ok(None));
        assert_eq!(table.get_actor_u256(&second), Ok(Some(U256::from(8))));
    }

    #[test]
    fn static_control_actor_map_deletion_preserves_probe_chain() {
        let (first, second) = colliding_actor_pair::<1>();
        let (mut control, mut slots) = control_actor_memory::<1>();
        let table = unsafe {
            StaticControlActorIdU256Map::<1>::new(
                control.as_mut_ptr() as usize,
                slots.as_mut_ptr() as usize,
            )
            .unwrap()
        };

        assert_eq!(table.insert_actor_u256(first, U256::from(1)), Ok(None));
        assert_eq!(table.insert_actor_u256(second, U256::from(2)), Ok(None));
        assert_eq!(table.remove_actor_u256(&first), Ok(Some(U256::from(1))));
        assert_eq!(table.get_actor_u256(&second), Ok(Some(U256::from(2))));
    }

    #[test]
    fn static_control_actor_map_reports_full_table() {
        let (mut control, mut slots) = control_actor_memory::<0>();
        let table = unsafe {
            StaticControlActorIdU256Map::<0>::new(
                control.as_mut_ptr() as usize,
                slots.as_mut_ptr() as usize,
            )
            .unwrap()
        };

        assert_eq!(
            table.insert_actor_u256(ActorId::from(1u64), U256::from(1)),
            Ok(None)
        );
        assert_eq!(
            table.insert_actor_u256(ActorId::from(2u64), U256::from(2)),
            Err(TableError::CapacityOverflow)
        );
    }

    #[test]
    fn static_control_actor_map_rejects_zero_actor_mutation() {
        let (mut control, mut slots) = control_actor_memory::<2>();
        let table = unsafe {
            StaticControlActorIdU256Map::<2>::new(
                control.as_mut_ptr() as usize,
                slots.as_mut_ptr() as usize,
            )
            .unwrap()
        };
        let actor = ActorId::zero();

        assert_eq!(
            table.insert_actor_u256(actor, U256::from(1)),
            Err(TableError::InvalidKey)
        );
        assert_eq!(table.remove_actor_u256(&actor), Err(TableError::InvalidKey));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_control_actor_map_clear_resets_control_and_data() {
        let (mut control, mut slots) = control_actor_memory::<2>();
        let table = unsafe {
            StaticControlActorIdU256Map::<2>::new(
                control.as_mut_ptr() as usize,
                slots.as_mut_ptr() as usize,
            )
            .unwrap()
        };

        table
            .insert_actor_u256(ActorId::from(1u64), U256::from(1))
            .unwrap();
        table.clear().unwrap();

        assert!(control.iter().all(|byte| *byte == 0));
        assert!(slots.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn static_page_local_actor_map_layout_fits_one_gear_page_tile() {
        assert_eq!(
            StaticPageLocalActorIdU256Map::<0>::tile_bytes(),
            PAGE_LOCAL_ACTOR_U256_TILE_BYTES
        );
        assert_eq!(StaticPageLocalActorIdU256Map::<0>::slots_per_tile(), 252);
        assert_eq!(StaticPageLocalActorIdU256Map::<0>::data_offset(), 256);
        assert_eq!(
            StaticPageLocalActorIdU256Map::<0>::data_offset()
                + StaticPageLocalActorIdU256Map::<0>::slots_per_tile()
                    * StaticPageLocalActorIdU256Map::<0>::slot_size(),
            StaticPageLocalActorIdU256Map::<0>::tile_bytes()
        );
        assert_eq!(
            StaticPageLocalActorIdU256Map::<1>::slots().unwrap(),
            2 * StaticPageLocalActorIdU256Map::<1>::slots_per_tile()
        );
        assert_eq!(
            StaticPageLocalActorIdU256Map::<1>::bytes_len().unwrap(),
            2 * PAGE_LOCAL_ACTOR_U256_TILE_BYTES
        );
    }

    #[test]
    fn static_page_local_actor_map_insert_update_and_remove() {
        let mut memory = page_local_actor_memory::<1>();
        let table = unsafe {
            StaticPageLocalActorIdU256Map::<1>::new(memory.as_mut_ptr() as usize).unwrap()
        };
        let actor = ActorId::from(1u64);

        assert_eq!(table.tile_count(), 2);
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
        assert_eq!(table.insert_actor_u256(actor, U256::from(10)), Ok(None));
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(10))));
        assert_eq!(
            table.insert_actor_u256(actor, U256::from(20)),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.remove_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_page_local_actor_map_zero_value_removes_and_reuses_deleted_slot() {
        let (first, second) = colliding_page_local_actor_pair::<0>();
        let mut memory = page_local_actor_memory::<0>();
        let table = unsafe {
            StaticPageLocalActorIdU256Map::<0>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        assert_eq!(table.insert_actor_u256(first, U256::from(7)), Ok(None));
        assert_eq!(
            table.insert_actor_u256(first, U256::zero()),
            Ok(Some(U256::from(7)))
        );
        assert_eq!(table.get_actor_u256(&first), Ok(None));
        assert_eq!(table.insert_actor_u256(second, U256::from(8)), Ok(None));
        assert_eq!(table.get_actor_u256(&second), Ok(Some(U256::from(8))));
    }

    #[test]
    fn static_page_local_actor_map_deletion_preserves_probe_chain() {
        let (first, second) = colliding_page_local_actor_pair::<0>();
        let mut memory = page_local_actor_memory::<0>();
        let table = unsafe {
            StaticPageLocalActorIdU256Map::<0>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        assert_eq!(table.insert_actor_u256(first, U256::from(1)), Ok(None));
        assert_eq!(table.insert_actor_u256(second, U256::from(2)), Ok(None));
        assert_eq!(table.remove_actor_u256(&first), Ok(Some(U256::from(1))));
        assert_eq!(table.get_actor_u256(&second), Ok(Some(U256::from(2))));
    }

    #[test]
    fn static_page_local_actor_map_reports_full_table() {
        let mut memory = page_local_actor_memory::<0>();
        let table = unsafe {
            StaticPageLocalActorIdU256Map::<0>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        for seed in 1..=StaticPageLocalActorIdU256Map::<0>::slots_per_tile() as u64 {
            assert_eq!(
                table.insert_actor_u256(ActorId::from(seed), U256::from(seed)),
                Ok(None)
            );
        }
        assert_eq!(
            table.insert_actor_u256(ActorId::from(10_000u64), U256::from(1)),
            Err(TableError::CapacityOverflow)
        );
    }

    #[test]
    fn static_page_local_actor_map_rejects_zero_actor_mutation() {
        let mut memory = page_local_actor_memory::<1>();
        let table = unsafe {
            StaticPageLocalActorIdU256Map::<1>::new(memory.as_mut_ptr() as usize).unwrap()
        };
        let actor = ActorId::zero();

        assert_eq!(
            table.insert_actor_u256(actor, U256::from(1)),
            Err(TableError::InvalidKey)
        );
        assert_eq!(table.remove_actor_u256(&actor), Err(TableError::InvalidKey));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_page_local_actor_map_clear_resets_tile_memory() {
        let mut memory = page_local_actor_memory::<1>();
        let table = unsafe {
            StaticPageLocalActorIdU256Map::<1>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        table
            .insert_actor_u256(ActorId::from(1u64), U256::from(1))
            .unwrap();
        table.clear().unwrap();

        assert!(memory.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn static_grouped_control_actor_map_layout_scales_by_group_pages() {
        assert_eq!(
            StaticGroupedControlActorIdU256Map::<2, 3>::groups().unwrap(),
            4
        );
        assert_eq!(
            StaticGroupedControlActorIdU256Map::<2, 3>::group_pages().unwrap(),
            8
        );
        assert_eq!(
            StaticGroupedControlActorIdU256Map::<2, 3>::slots_per_group().unwrap(),
            8 * StaticPageLocalActorIdU256Map::<0>::slots_per_tile()
        );
        assert_eq!(
            StaticGroupedControlActorIdU256Map::<2, 3>::data_offset().unwrap(),
            8 * StaticPageLocalActorIdU256Map::<0>::data_offset()
        );
        assert_eq!(
            StaticGroupedControlActorIdU256Map::<2, 3>::group_bytes().unwrap(),
            8 * PAGE_LOCAL_ACTOR_U256_TILE_BYTES
        );
        assert_eq!(
            StaticGroupedControlActorIdU256Map::<2, 3>::slots().unwrap(),
            4 * 8 * StaticPageLocalActorIdU256Map::<0>::slots_per_tile()
        );
        assert_eq!(
            StaticGroupedControlActorIdU256Map::<2, 3>::bytes_len().unwrap(),
            4 * 8 * PAGE_LOCAL_ACTOR_U256_TILE_BYTES
        );
    }

    #[test]
    fn static_grouped_control_actor_map_insert_update_and_remove() {
        let mut memory = grouped_control_actor_memory::<1, 1>();
        let table = unsafe {
            StaticGroupedControlActorIdU256Map::<1, 1>::new(memory.as_mut_ptr() as usize).unwrap()
        };
        let actor = ActorId::from(1u64);

        assert_eq!(table.group_count(), 2);
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
        assert_eq!(table.insert_actor_u256(actor, U256::from(10)), Ok(None));
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(10))));
        assert_eq!(
            table.insert_actor_u256(actor, U256::from(20)),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(table.get_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.remove_actor_u256(&actor), Ok(Some(U256::from(20))));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_grouped_control_actor_map_zero_value_removes_and_reuses_deleted_slot() {
        let (first, second) = colliding_grouped_control_actor_pair::<0, 1>();
        let mut memory = grouped_control_actor_memory::<0, 1>();
        let table = unsafe {
            StaticGroupedControlActorIdU256Map::<0, 1>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        assert_eq!(table.insert_actor_u256(first, U256::from(7)), Ok(None));
        assert_eq!(
            table.insert_actor_u256(first, U256::zero()),
            Ok(Some(U256::from(7)))
        );
        assert_eq!(table.get_actor_u256(&first), Ok(None));
        assert_eq!(table.insert_actor_u256(second, U256::from(8)), Ok(None));
        assert_eq!(table.get_actor_u256(&second), Ok(Some(U256::from(8))));
    }

    #[test]
    fn static_grouped_control_actor_map_deletion_preserves_probe_chain() {
        let (first, second) = colliding_grouped_control_actor_pair::<0, 1>();
        let mut memory = grouped_control_actor_memory::<0, 1>();
        let table = unsafe {
            StaticGroupedControlActorIdU256Map::<0, 1>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        assert_eq!(table.insert_actor_u256(first, U256::from(1)), Ok(None));
        assert_eq!(table.insert_actor_u256(second, U256::from(2)), Ok(None));
        assert_eq!(table.remove_actor_u256(&first), Ok(Some(U256::from(1))));
        assert_eq!(table.get_actor_u256(&second), Ok(Some(U256::from(2))));
    }

    #[test]
    fn static_grouped_control_actor_map_reports_full_table() {
        let mut memory = grouped_control_actor_memory::<0, 0>();
        let table = unsafe {
            StaticGroupedControlActorIdU256Map::<0, 0>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        for seed in
            1..=StaticGroupedControlActorIdU256Map::<0, 0>::slots_per_group().unwrap() as u64
        {
            assert_eq!(
                table.insert_actor_u256(ActorId::from(seed), U256::from(seed)),
                Ok(None)
            );
        }
        assert_eq!(
            table.insert_actor_u256(ActorId::from(10_000u64), U256::from(1)),
            Err(TableError::CapacityOverflow)
        );
    }

    #[test]
    fn static_grouped_control_actor_map_rejects_zero_actor_mutation() {
        let mut memory = grouped_control_actor_memory::<1, 1>();
        let table = unsafe {
            StaticGroupedControlActorIdU256Map::<1, 1>::new(memory.as_mut_ptr() as usize).unwrap()
        };
        let actor = ActorId::zero();

        assert_eq!(
            table.insert_actor_u256(actor, U256::from(1)),
            Err(TableError::InvalidKey)
        );
        assert_eq!(table.remove_actor_u256(&actor), Err(TableError::InvalidKey));
        assert_eq!(table.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn static_grouped_control_actor_map_clear_resets_group_memory() {
        let mut memory = grouped_control_actor_memory::<1, 1>();
        let table = unsafe {
            StaticGroupedControlActorIdU256Map::<1, 1>::new(memory.as_mut_ptr() as usize).unwrap()
        };

        table
            .insert_actor_u256(ActorId::from(1u64), U256::from(1))
            .unwrap();
        table.clear().unwrap();

        assert!(memory.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn static_allowance_map_insert_update_and_remove() {
        let mut memory = vec![0u8; StaticAllowanceU256Map::<2>::bytes_len().unwrap()];
        let table =
            unsafe { StaticAllowanceU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);

        assert_eq!(table.get_allowance_u256(&owner, &spender), Ok(None));
        assert_eq!(
            table.insert_allowance_u256(owner, spender, U256::from(10)),
            Ok(None)
        );
        assert_eq!(
            table.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(
            table.insert_allowance_u256(owner, spender, U256::zero()),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(table.get_allowance_u256(&owner, &spender), Ok(None));
    }

    #[test]
    fn static_allowance_map_decreases_allowance_in_one_operation() {
        let mut memory = vec![0u8; StaticAllowanceU256Map::<2>::bytes_len().unwrap()];
        let table =
            unsafe { StaticAllowanceU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let owner = ActorId::from(1u64);
        let spender = ActorId::from(2u64);

        assert_eq!(
            table.insert_allowance_u256(owner, spender, U256::from(10)),
            Ok(None)
        );
        assert_eq!(
            table.decrease_allowance_u256(owner, spender, U256::from(4)),
            Ok(Some(U256::from(6)))
        );
        assert_eq!(
            table.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(6)))
        );
        assert_eq!(
            table.decrease_allowance_u256(owner, spender, U256::from(7)),
            Ok(None)
        );
        assert_eq!(
            table.get_allowance_u256(&owner, &spender),
            Ok(Some(U256::from(6)))
        );
    }

    #[test]
    fn static_allowance_map_rejects_zero_actor_mutation() {
        let mut memory = vec![0u8; StaticAllowanceU256Map::<2>::bytes_len().unwrap()];
        let table =
            unsafe { StaticAllowanceU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let owner = ActorId::zero();
        let spender = ActorId::from(2u64);

        assert_eq!(
            table.insert_allowance_u256(owner, spender, U256::from(1)),
            Err(TableError::InvalidKey)
        );
        assert_eq!(
            table.remove_allowance_u256(&owner, &spender),
            Err(TableError::InvalidKey)
        );
        assert_eq!(table.get_allowance_u256(&owner, &spender), Ok(None));
    }

    fn control_actor_memory<const LOG2_SLOTS: u8>() -> (Vec<u8>, Vec<u8>) {
        (
            vec![0u8; StaticControlActorIdU256Map::<LOG2_SLOTS>::control_bytes_len().unwrap()],
            vec![0u8; StaticControlActorIdU256Map::<LOG2_SLOTS>::slots_bytes_len().unwrap()],
        )
    }

    fn page_local_actor_memory<const LOG2_TILES: u8>() -> Vec<u8> {
        vec![0u8; StaticPageLocalActorIdU256Map::<LOG2_TILES>::bytes_len().unwrap()]
    }

    fn grouped_control_actor_memory<const LOG2_GROUPS: u8, const LOG2_GROUP_PAGES: u8>() -> Vec<u8>
    {
        vec![
            0u8;
            StaticGroupedControlActorIdU256Map::<LOG2_GROUPS, LOG2_GROUP_PAGES>::bytes_len()
                .unwrap()
        ]
    }

    fn colliding_actor_pair<const LOG2_SLOTS: u8>() -> (ActorId, ActorId) {
        let mut seen = vec![None; 1usize << LOG2_SLOTS];
        for seed in 1u64..=4096 {
            let actor = ActorId::from(seed);
            let index = actor_index::<LOG2_SLOTS>(actor);
            if let Some(first) = seen[index] {
                return (first, actor);
            }
            seen[index] = Some(actor);
        }

        unreachable!("small actor key space must contain collisions")
    }

    fn colliding_page_local_actor_pair<const LOG2_TILES: u8>() -> (ActorId, ActorId) {
        let slots =
            (1usize << LOG2_TILES) * StaticPageLocalActorIdU256Map::<LOG2_TILES>::slots_per_tile();
        let mut seen = vec![None; slots];
        for seed in 1u64..=4096 {
            let actor = ActorId::from(seed);
            let index = page_local_actor_index::<LOG2_TILES>(actor);
            if let Some(first) = seen[index] {
                return (first, actor);
            }
            seen[index] = Some(actor);
        }

        unreachable!("small actor key space must contain page-local collisions")
    }

    fn colliding_grouped_control_actor_pair<const LOG2_GROUPS: u8, const LOG2_GROUP_PAGES: u8>()
    -> (ActorId, ActorId) {
        let slots =
            StaticGroupedControlActorIdU256Map::<LOG2_GROUPS, LOG2_GROUP_PAGES>::slots().unwrap();
        let mut seen = vec![None; slots];
        for seed in 1u64..=4096 {
            let actor = ActorId::from(seed);
            let index = grouped_control_actor_index::<LOG2_GROUPS, LOG2_GROUP_PAGES>(actor);
            if let Some(first) = seen[index] {
                return (first, actor);
            }
            seen[index] = Some(actor);
        }

        unreachable!("small actor key space must contain grouped-control collisions")
    }

    #[cfg(feature = "experimental-vft-account")]
    fn colliding_vft_account_pair<const LOG2_SLOTS: u8>() -> (ActorId, ActorId) {
        let mut seen = vec![None; 1usize << LOG2_SLOTS];
        for seed in 1u64..=4096 {
            let actor = ActorId::from(seed);
            let index = vft_account_index::<LOG2_SLOTS>(actor);
            if let Some(first) = seen[index] {
                return (first, actor);
            }
            seen[index] = Some(actor);
        }

        unreachable!("small actor key space must contain VFT account collisions")
    }

    fn actor_index<const LOG2_SLOTS: u8>(actor: ActorId) -> usize {
        let key = actor_key(actor);
        let hash = hash_words_for_test(&key);
        if LOG2_SLOTS == 0 {
            0
        } else {
            (hash >> (32 - u32::from(LOG2_SLOTS))) as usize
        }
    }

    fn page_local_actor_index<const LOG2_TILES: u8>(actor: ActorId) -> usize {
        let key = actor_key(actor);
        let hash = hash_words_for_test(&key);
        let tile = if LOG2_TILES == 0 {
            0
        } else {
            (hash >> (32 - u32::from(LOG2_TILES))) as usize
        };
        let tile_slot =
            (hash as usize) % StaticPageLocalActorIdU256Map::<LOG2_TILES>::slots_per_tile();
        tile * StaticPageLocalActorIdU256Map::<LOG2_TILES>::slots_per_tile() + tile_slot
    }

    fn grouped_control_actor_index<const LOG2_GROUPS: u8, const LOG2_GROUP_PAGES: u8>(
        actor: ActorId,
    ) -> usize {
        let key = actor_key(actor);
        let hash = hash_words_for_test(&key);
        let group = if LOG2_GROUPS == 0 {
            0
        } else {
            (hash >> (32 - u32::from(LOG2_GROUPS))) as usize
        };
        let group_slot = (hash as usize)
            % StaticGroupedControlActorIdU256Map::<LOG2_GROUPS, LOG2_GROUP_PAGES>::slots_per_group(
            )
            .unwrap();
        group
            * StaticGroupedControlActorIdU256Map::<LOG2_GROUPS, LOG2_GROUP_PAGES>::slots_per_group()
                .unwrap()
            + group_slot
    }

    #[cfg(feature = "experimental-vft-account")]
    fn vft_account_index<const LOG2_SLOTS: u8>(actor: ActorId) -> usize {
        let key = actor_key(actor);
        let mask = (1usize << LOG2_SLOTS) - 1;
        let tag = unsafe { ptr::read_unaligned(key.as_ptr().add(12).cast::<u64>()) };
        let hash = if tag != 0 {
            tag
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

    fn hash_words_for_test(bytes: &[u8; 32]) -> u32 {
        let mut hash = 0u32;
        let mut offset = 0;
        while offset < 32 {
            hash ^= u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;
        }

        hash.wrapping_mul(0x9E37_79B9)
    }
}
