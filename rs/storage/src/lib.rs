//! Allocator-light storage primitives for Sails programs.
//!
//! This crate provides bounded maps for program state that should not grow
//! through the allocator at runtime. It has two layers:
//!
//! - [`FixedOpenAddressMap`] stores a fixed-capacity map directly inside a
//!   Rust value. It is useful for small bounded state and tests.
//! - [`StaticOpenAddressTable`] and the actor-specific maps operate on a
//!   caller-reserved memory region. They are useful when a Sails program wants
//!   stable, lazy-page-friendly storage outside allocator-managed collections.
//!
//! The static tables are intentionally low level. Constructors are `unsafe`
//! because the caller must reserve a valid writable memory range and keep it
//! from overlapping other mutable state for the whole lifetime of the table.
//! In normal Sails builds, use `sails_rs::build::StaticMemoryLayout` from a
//! build script to reserve static memory and generate base/size constants.
//!
//! # Fixed map
//!
//! ```
//! use sails_storage::FixedOpenAddressMap;
//!
//! let mut map = FixedOpenAddressMap::<1, 1, 2>::new();
//! assert_eq!(map.insert([1], [10]), Ok(None));
//! assert_eq!(map.get(&[1]), Ok(Some([10])));
//! ```
//!
//! # Generic static table
//!
//! ```
//! use sails_storage::StaticOpenAddressTable;
//!
//! let mut memory = [0u8; StaticOpenAddressTable::<1, 1>::slot_size() * 2];
//! let table = unsafe {
//!     StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 2).unwrap()
//! };
//!
//! assert_eq!(table.insert(&[1], &[10]), Ok(None));
//! assert_eq!(table.get(&[1]), Ok(Some([10])));
//! ```
//!
//! # Actor maps
//!
//! [`StaticActorIdU256Map`] and [`StaticActorPairU256Map`] are specialized
//! static-memory layouts for common token-like state. They store `ActorId` and
//! `U256` values without per-slot state bytes: an all-zero actor key marks an
//! empty slot, and writing a zero `U256` removes the visible value for an
//! existing key. Because zero actor ids are reserved by the layout, mutation
//! methods reject them with [`TableError::InvalidKey`].
//!
//! ```
//! use gprimitives::{ActorId, U256};
//! use sails_storage::{ACTOR_ID_U256_SLOT_SIZE, StaticActorIdU256Map};
//!
//! let mut memory = [0u8; 4 * ACTOR_ID_U256_SLOT_SIZE];
//! let balances =
//!     unsafe { StaticActorIdU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
//! let account = ActorId::from(1u64);
//!
//! assert_eq!(balances.insert_actor_u256(account, U256::from(10)), Ok(None));
//! assert_eq!(balances.get_actor_u256(&account), Ok(Some(U256::from(10))));
//! ```

#![no_std]

use core::{fmt, marker::PhantomData, ptr};
use gprimitives::{ActorId, U256};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum SlotState {
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

/// Slot size used by the specialized actor-pair static map.
pub const ACTOR_PAIR_U256_SLOT_SIZE: usize = 96;

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
                self.slots[index].value = value;
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
                    self.write_value(index, value);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StaticLookup {
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

fn actor_key(actor: ActorId) -> [u8; 32] {
    let mut key = [0u8; 32];
    key.copy_from_slice(actor.as_ref());
    key
}

fn u256_value(value: U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    value.to_little_endian(&mut bytes);
    bytes
}

fn u256_from_value(bytes: [u8; 32]) -> U256 {
    U256::from_little_endian(&bytes)
}

fn reject_zero_key(bytes: &[u8; 32]) -> Result<(), TableError> {
    if is_zero_32(bytes) {
        Err(TableError::InvalidKey)
    } else {
        Ok(())
    }
}

fn is_zero_32(bytes: &[u8; 32]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

fn words_32(bytes: &[u8; 32]) -> [u64; 4] {
    [
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]),
        u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]),
        u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]),
        u64::from_le_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
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

const HASH_GOLDEN_RATIO: u32 = 0x9E37_79B9;
const HASH_PAIR_SPENDER: u32 = 0x85EB_CA6B;

fn hash_words(bytes: &[u8; 32]) -> u32 {
    fold_words(bytes).wrapping_mul(HASH_GOLDEN_RATIO)
}

fn hash_actor_key(key: &[u8; 32]) -> u32 {
    hash_words(key)
}

fn hash_actor_pair_key(left: &[u8; 32], right: &[u8; 32]) -> u32 {
    fold_words(left)
        .wrapping_mul(HASH_GOLDEN_RATIO)
        .wrapping_add(fold_words(right).wrapping_mul(HASH_PAIR_SPENDER))
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

/// A static-memory map keyed by `ActorId` and storing `U256`.
///
/// The layout uses `2^LOG2_SLOTS` slots. Each slot is exactly 64 bytes:
/// a 32-byte actor id followed by a 32-byte little-endian `U256` value.
pub struct StaticActorIdU256Map<const LOG2_SLOTS: u8> {
    base: usize,
    slots: usize,
    mask: usize,
    _marker: PhantomData<*mut u8>,
}

impl<const LOG2_SLOTS: u8> StaticActorIdU256Map<LOG2_SLOTS> {
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

    /// Returns the byte length required for this map.
    pub fn bytes_len() -> Result<usize, TableError> {
        Self::slots()?
            .checked_mul(Self::slot_size())
            .ok_or(TableError::InvalidLayout)
    }

    /// Creates a map over `2^LOG2_SLOTS * 64` bytes at `base`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the memory interval is valid for reads and writes
    /// for the whole lifetime of the map and does not overlap other mutable
    /// state.
    pub unsafe fn new(base: usize) -> Result<Self, TableError> {
        let slots = Self::slots()?;
        StaticRegion::new(base, Self::bytes_len()?)?;
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

    /// Returns the total byte length occupied by this map.
    pub fn bytes(&self) -> Result<usize, TableError> {
        Self::bytes_len()
    }

    /// Returns the visible value for `actor`.
    pub fn get_actor_u256(&self, actor: &ActorId) -> Result<Option<U256>, TableError> {
        let key = actor_key(*actor);
        if is_zero_32(&key) {
            return Ok(None);
        }

        let StaticLookup::Found(index) = self.lookup(&key) else {
            return Ok(None);
        };
        Ok(self.visible_value(index).map(u256_from_value))
    }

    /// Inserts or updates `actor`, returning the previous visible value.
    ///
    /// A zero value removes the visible value. Zero actor ids are rejected
    /// because zero key bytes mark empty slots in this layout.
    pub fn insert_actor_u256(
        &self,
        actor: ActorId,
        value: U256,
    ) -> Result<Option<U256>, TableError> {
        let key = actor_key(actor);
        reject_zero_key(&key)?;
        if value.is_zero() {
            return self
                .remove_key(&key)
                .map(|previous| previous.map(u256_from_value));
        }

        let value = u256_value(value);
        match self.lookup(&key) {
            StaticLookup::Found(index) => {
                let previous = self.visible_value(index);
                unsafe {
                    self.write_value(index, &value);
                }
                Ok(previous.map(u256_from_value))
            }
            StaticLookup::Vacant(index) => {
                unsafe {
                    self.write_key(index, &key);
                    self.write_value(index, &value);
                }
                Ok(None)
            }
            StaticLookup::Full => Err(TableError::CapacityOverflow),
        }
    }

    /// Removes the visible value for `actor`.
    pub fn remove_actor_u256(&self, actor: &ActorId) -> Result<Option<U256>, TableError> {
        let key = actor_key(*actor);
        reject_zero_key(&key)?;
        self.remove_key(&key)
            .map(|previous| previous.map(u256_from_value))
    }

    /// Clears every slot to the empty state.
    pub fn clear(&self) -> Result<(), TableError> {
        unsafe {
            ptr::write_bytes(self.base as *mut u8, 0, self.bytes()?);
        }
        Ok(())
    }

    fn remove_key(&self, key: &[u8; 32]) -> Result<Option<[u8; 32]>, TableError> {
        let StaticLookup::Found(index) = self.lookup(key) else {
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

    fn lookup(&self, key: &[u8; 32]) -> StaticLookup {
        let key_words = words_32(key);
        let mut index = static_map_index(hash_actor_key(key), LOG2_SLOTS);

        for _ in 0..self.slots {
            let stored_key = unsafe { read_words_32(self.key_ptr(index)) };
            if stored_key == key_words {
                return StaticLookup::Found(index);
            }
            if words_are_zero(stored_key) {
                return StaticLookup::Vacant(index);
            }

            index = (index + 1) & self.mask;
        }

        StaticLookup::Full
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
        unsafe { self.slot_ptr(slot).add(32) }
    }

    fn slot_ptr(&self, slot: usize) -> *mut u8 {
        (self.base + slot * Self::slot_size()) as *mut u8
    }
}

/// A static-memory map keyed by two `ActorId`s and storing `U256`.
///
/// The layout uses `2^LOG2_SLOTS` slots. Each slot is exactly 96 bytes:
/// the two 32-byte actor ids followed by a 32-byte little-endian `U256` value.
pub struct StaticActorPairU256Map<const LOG2_SLOTS: u8> {
    base: usize,
    slots: usize,
    mask: usize,
    _marker: PhantomData<*mut u8>,
}

impl<const LOG2_SLOTS: u8> StaticActorPairU256Map<LOG2_SLOTS> {
    /// Returns the byte length of one slot.
    pub const fn slot_size() -> usize {
        ACTOR_PAIR_U256_SLOT_SIZE
    }

    /// Returns the configured slot count.
    pub fn slots() -> Result<usize, TableError> {
        static_map_slots(LOG2_SLOTS)
    }

    /// Returns the mask used for power-of-two probing.
    pub fn mask() -> Result<usize, TableError> {
        Ok(Self::slots()? - 1)
    }

    /// Returns the byte length required for this map.
    pub fn bytes_len() -> Result<usize, TableError> {
        Self::slots()?
            .checked_mul(Self::slot_size())
            .ok_or(TableError::InvalidLayout)
    }

    /// Creates a map over `2^LOG2_SLOTS * 96` bytes at `base`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the memory interval is valid for reads and writes
    /// for the whole lifetime of the map and does not overlap other mutable
    /// state.
    pub unsafe fn new(base: usize) -> Result<Self, TableError> {
        let slots = Self::slots()?;
        StaticRegion::new(base, Self::bytes_len()?)?;
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

    /// Returns the total byte length occupied by this map.
    pub fn bytes(&self) -> Result<usize, TableError> {
        Self::bytes_len()
    }

    /// Returns the visible value for `(left, right)`.
    pub fn get_actor_pair_u256(
        &self,
        left: &ActorId,
        right: &ActorId,
    ) -> Result<Option<U256>, TableError> {
        let left = actor_key(*left);
        let right = actor_key(*right);
        if is_zero_32(&left) || is_zero_32(&right) {
            return Ok(None);
        }

        let StaticLookup::Found(index) = self.lookup(&left, &right) else {
            return Ok(None);
        };
        Ok(self.visible_value(index).map(u256_from_value))
    }

    /// Inserts or updates `(left, right)`, returning the previous visible value.
    ///
    /// A zero value removes the visible value. Zero actor ids are rejected
    /// because zero key bytes mark empty slots in this layout.
    pub fn insert_actor_pair_u256(
        &self,
        left: ActorId,
        right: ActorId,
        value: U256,
    ) -> Result<Option<U256>, TableError> {
        let left = actor_key(left);
        let right = actor_key(right);
        reject_zero_key(&left)?;
        reject_zero_key(&right)?;
        if value.is_zero() {
            return self
                .remove_key(&left, &right)
                .map(|previous| previous.map(u256_from_value));
        }

        let value = u256_value(value);
        match self.lookup(&left, &right) {
            StaticLookup::Found(index) => {
                let previous = self.visible_value(index);
                unsafe {
                    self.write_value(index, &value);
                }
                Ok(previous.map(u256_from_value))
            }
            StaticLookup::Vacant(index) => {
                unsafe {
                    self.write_left(index, &left);
                    self.write_right(index, &right);
                    self.write_value(index, &value);
                }
                Ok(None)
            }
            StaticLookup::Full => Err(TableError::CapacityOverflow),
        }
    }

    /// Removes the visible value for `(left, right)`.
    pub fn remove_actor_pair_u256(
        &self,
        left: &ActorId,
        right: &ActorId,
    ) -> Result<Option<U256>, TableError> {
        let left = actor_key(*left);
        let right = actor_key(*right);
        reject_zero_key(&left)?;
        reject_zero_key(&right)?;
        self.remove_key(&left, &right)
            .map(|previous| previous.map(u256_from_value))
    }

    /// Clears every slot to the empty state.
    pub fn clear(&self) -> Result<(), TableError> {
        unsafe {
            ptr::write_bytes(self.base as *mut u8, 0, self.bytes()?);
        }
        Ok(())
    }

    fn remove_key(
        &self,
        left: &[u8; 32],
        right: &[u8; 32],
    ) -> Result<Option<[u8; 32]>, TableError> {
        let StaticLookup::Found(index) = self.lookup(left, right) else {
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

    fn lookup(&self, left: &[u8; 32], right: &[u8; 32]) -> StaticLookup {
        let left_words = words_32(left);
        let right_words = words_32(right);
        let mut index = static_map_index(hash_actor_pair_key(left, right), LOG2_SLOTS);

        for _ in 0..self.slots {
            let stored_left = unsafe { read_words_32(self.left_ptr(index)) };
            if words_are_zero(stored_left) {
                return StaticLookup::Vacant(index);
            }
            if stored_left == left_words
                && unsafe { read_words_32(self.right_ptr(index)) } == right_words
            {
                return StaticLookup::Found(index);
            }

            index = (index + 1) & self.mask;
        }

        StaticLookup::Full
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

    unsafe fn write_left(&self, slot: usize, left: &[u8; 32]) {
        unsafe {
            ptr::copy_nonoverlapping(left.as_ptr(), self.left_ptr(slot), 32);
        }
    }

    unsafe fn write_right(&self, slot: usize, right: &[u8; 32]) {
        unsafe {
            ptr::copy_nonoverlapping(right.as_ptr(), self.right_ptr(slot), 32);
        }
    }

    unsafe fn write_value(&self, slot: usize, value: &[u8; 32]) {
        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), self.value_ptr(slot), 32);
        }
    }

    fn left_ptr(&self, slot: usize) -> *mut u8 {
        self.slot_ptr(slot)
    }

    fn right_ptr(&self, slot: usize) -> *mut u8 {
        unsafe { self.slot_ptr(slot).add(32) }
    }

    fn value_ptr(&self, slot: usize) -> *mut u8 {
        unsafe { self.slot_ptr(slot).add(64) }
    }

    fn slot_ptr(&self, slot: usize) -> *mut u8 {
        (self.base + slot * Self::slot_size()) as *mut u8
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
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
    fn actor_map_preserves_static_layout_and_tombstones_zero_values() {
        assert_eq!(StaticActorIdU256Map::<2>::slot_size(), 64);
        assert_eq!(StaticActorIdU256Map::<2>::slots(), Ok(4));
        assert_eq!(StaticActorIdU256Map::<2>::mask(), Ok(3));
        assert_eq!(StaticActorIdU256Map::<2>::bytes_len(), Ok(256));

        let mut memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let map = unsafe { StaticActorIdU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let actor = ActorId::from(1u64);

        assert_eq!(map.get_actor_u256(&actor), Ok(None));
        assert_eq!(map.insert_actor_u256(actor, U256::from(10)), Ok(None));
        assert_eq!(map.get_actor_u256(&actor), Ok(Some(U256::from(10))));
        assert_eq!(
            map.insert_actor_u256(actor, U256::zero()),
            Ok(Some(U256::from(10)))
        );
        assert_eq!(map.get_actor_u256(&actor), Ok(None));
    }

    #[test]
    fn actor_map_rejects_zero_actor_mutation() {
        let mut memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
        let map = unsafe { StaticActorIdU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };

        assert_eq!(
            map.insert_actor_u256(ActorId::zero(), U256::from(1)),
            Err(TableError::InvalidKey)
        );
        assert_eq!(
            map.remove_actor_u256(&ActorId::zero()),
            Err(TableError::InvalidKey)
        );
    }

    #[test]
    fn actor_pair_map_preserves_static_layout_and_basic_ops() {
        assert_eq!(StaticActorPairU256Map::<2>::slot_size(), 96);
        assert_eq!(StaticActorPairU256Map::<2>::slots(), Ok(4));
        assert_eq!(StaticActorPairU256Map::<2>::mask(), Ok(3));
        assert_eq!(StaticActorPairU256Map::<2>::bytes_len(), Ok(384));

        let mut memory = vec![0u8; StaticActorPairU256Map::<2>::bytes_len().unwrap()];
        let map =
            unsafe { StaticActorPairU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };
        let left = ActorId::from(1u64);
        let right = ActorId::from(2u64);

        assert_eq!(map.get_actor_pair_u256(&left, &right), Ok(None));
        assert_eq!(
            map.insert_actor_pair_u256(left, right, U256::from(20)),
            Ok(None)
        );
        assert_eq!(
            map.get_actor_pair_u256(&left, &right),
            Ok(Some(U256::from(20)))
        );
        assert_eq!(
            map.remove_actor_pair_u256(&left, &right),
            Ok(Some(U256::from(20)))
        );
        assert_eq!(map.get_actor_pair_u256(&left, &right), Ok(None));
    }

    #[test]
    fn actor_pair_map_rejects_zero_actor_mutation() {
        let mut memory = vec![0u8; StaticActorPairU256Map::<2>::bytes_len().unwrap()];
        let map =
            unsafe { StaticActorPairU256Map::<2>::new(memory.as_mut_ptr() as usize).unwrap() };

        assert_eq!(
            map.insert_actor_pair_u256(ActorId::zero(), ActorId::from(1u64), U256::from(1)),
            Err(TableError::InvalidKey)
        );
    }
}
