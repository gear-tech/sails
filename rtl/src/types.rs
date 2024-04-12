use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, TypeInfo)]
pub struct ActorId([u8; 32]);

impl From<[u8; 32]> for ActorId {
    fn from(arr: [u8; 32]) -> Self {
        Self(arr)
    }
}

// Can panic if slice is not 32 bytes long
impl From<&[u8]> for ActorId {
    fn from(slice: &[u8]) -> Self {
        let mut arr = [0; 32];
        arr.copy_from_slice(slice);
        Self(arr)
    }
}

impl From<u64> for ActorId {
    fn from(v: u64) -> Self {
        let mut arr = [0u8; 32];
        arr[0..8].copy_from_slice(&v.to_le_bytes()[..]);
        Self(arr)
    }
}

impl AsRef<[u8; 32]> for ActorId {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, TypeInfo)]
pub struct MessageId([u8; 32]);

impl From<[u8; 32]> for MessageId {
    fn from(arr: [u8; 32]) -> Self {
        Self(arr)
    }
}

// Can panic if slice is not 32 bytes long
impl From<&[u8]> for MessageId {
    fn from(slice: &[u8]) -> Self {
        let mut arr = [0; 32];
        arr.copy_from_slice(slice);
        Self(arr)
    }
}

impl From<u64> for MessageId {
    fn from(v: u64) -> Self {
        let mut arr = [0u8; 32];
        arr[0..8].copy_from_slice(&v.to_le_bytes()[..]);
        Self(arr)
    }
}

impl AsRef<[u8; 32]> for MessageId {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, TypeInfo)]
pub struct CodeId([u8; 32]);

impl From<[u8; 32]> for CodeId {
    fn from(arr: [u8; 32]) -> Self {
        Self(arr)
    }
}

// Can panic if slice is not 32 bytes long
impl From<&[u8]> for CodeId {
    fn from(slice: &[u8]) -> Self {
        let mut arr = [0; 32];
        arr.copy_from_slice(slice);
        Self(arr)
    }
}

impl From<u64> for CodeId {
    fn from(v: u64) -> Self {
        let mut arr = [0u8; 32];
        arr[0..8].copy_from_slice(&v.to_le_bytes()[..]);
        Self(arr)
    }
}

impl AsRef<[u8; 32]> for CodeId {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

pub type ValueUnit = u128;

pub type GasUnit = u64;
