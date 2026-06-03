//! Allocator-light storage primitives for Sails programs.
//!
//! This crate provides bounded maps for program state that should not grow
//! through the allocator at runtime. It has two layers:
//!
//! - [`FixedOpenAddressMap`] stores a fixed-capacity map directly inside a
//!   Rust value. It is useful for small bounded state and tests.
//! - [`StaticOpenAddressTable`] operates on a caller-reserved memory region.
//!   It is useful when a Sails program wants stable, lazy-page-friendly storage
//!   outside allocator-managed collections.
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

#![no_std]

use core::{fmt, marker::PhantomData, ptr};

/// Errors returned by fixed and static storage tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableError {
    /// The table has no reusable slot for a new key.
    CapacityOverflow,
    /// The provided memory layout overflows or does not fit the requested region.
    InvalidLayout,
    /// A static-memory slot state byte is not one of the supported values.
    InvalidSlotState,
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityOverflow => f.write_str("capacity overflow"),
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
    hash: u32,
    mut classify: impl FnMut(usize) -> Result<SlotMatch, TableError>,
) -> Result<Lookup, TableError> {
    if slots == 0 {
        return Ok(Lookup::Full);
    }

    let mut first_deleted = None;
    let mut index = start_index(slots, hash);

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

fn start_index(slots: usize, hash: u32) -> usize {
    let hash = hash as usize;
    if slots.is_power_of_two() {
        hash & (slots - 1)
    } else {
        hash % slots
    }
}

fn hash_bytes(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5u32;

    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }

    hash
}

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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::{vec, vec::Vec};

    type Fixed = FixedOpenAddressMap<1, 1, 2>;

    fn colliding_keys(slots: usize) -> ([u8; 1], [u8; 1]) {
        for left in 0u8..=u8::MAX {
            for right in left.wrapping_add(1)..=u8::MAX {
                if start_index(slots, hash_bytes(&[left]))
                    == start_index(slots, hash_bytes(&[right]))
                {
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
    fn hash_bytes_is_arch_stable() {
        assert_eq!(hash_bytes(&[]), 0x811c_9dc5);
        assert_eq!(hash_bytes(&[0]), 0x050c_5d1f);
        assert_eq!(hash_bytes(&[1, 2, 3, 4]), 0x5734_a87d);
    }
}
