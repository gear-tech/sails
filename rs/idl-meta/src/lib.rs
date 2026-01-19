#![no_std]

extern crate alloc;

#[cfg(feature = "ast")]
mod ast;
#[cfg(feature = "ast")]
mod hash;

use alloc::{
    format,
    string::{String, ToString as _},
};
#[cfg(feature = "ast")]
pub use ast::*;
use parity_scale_codec::{Decode, Encode, Error};
use scale_info::{MetaType, StaticTypeInfo, prelude::vec::Vec};

mod header;
pub use header::*;

pub type AnyServiceMetaFn = fn() -> AnyServiceMeta;

/// A trait for types that have a static Interface ID.
pub trait Identifiable {
    const INTERFACE_ID: InterfaceId;
}

/// A trait for types that represent a service method, providing its Entry ID.
pub trait MethodMeta: Identifiable {
    const ENTRY_ID: u16;
}

/// Unique identifier for a service (or "interface" in terms of sails binary protocol).
///
/// For more information about interface IDs, see the interface ID spec.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InterfaceId(pub [u8; 8]);

impl InterfaceId {
    /// Create a zeroed interface ID.
    pub const fn zero() -> Self {
        Self([0u8; 8])
    }

    /// Create interface ID from bytes.
    pub const fn from_bytes_32(bytes: [u8; 32]) -> Self {
        let inner = [
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ];

        Self(inner)
    }

    /// Create interface ID from bytes.
    pub const fn from_bytes_8(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Create interface ID from u64.
    pub const fn from_u64(int: u64) -> Self {
        Self(int.to_be_bytes())
    }

    /// Get interface ID as a u64.
    pub const fn as_u64(&self) -> u64 {
        u64::from_be_bytes(self.0)
    }

    /// Get interface ID as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Deserialize from bytes, advancing the slice
    pub fn try_read_bytes(bytes: &mut &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 8 {
            return Err("Insufficient bytes for interface ID");
        }

        let mut id = [0u8; 8];
        id.copy_from_slice(&bytes[0..8]);
        *bytes = &bytes[8..];
        Ok(Self(id))
    }

    /// Deserialize from bytes without mutating the input
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut slice = bytes;
        Self::try_read_bytes(&mut slice)
    }
}

impl core::fmt::Debug for InterfaceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

impl core::fmt::Display for InterfaceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("0x")?;
        for byte in self.as_bytes() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl core::str::FromStr for InterfaceId {
    type Err = String;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        // Strip optional 0x / 0X prefix
        if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            s = rest;
        }

        if s.len() != 16 {
            return Err(format!("expected 16 hex digits (8 bytes), got {}", s.len()));
        }

        let mut bytes = [0u8; 8];
        for (i, chunk) in s.as_bytes().chunks_exact(2).enumerate() {
            let hex = core::str::from_utf8(chunk).map_err(|_| "invalid UTF-8".to_string())?;

            bytes[i] =
                u8::from_str_radix(hex, 16).map_err(|_| format!("invalid hex byte: {hex}"))?;
        }

        Ok(InterfaceId(bytes))
    }
}

impl Encode for InterfaceId {
    fn encode_to<O: parity_scale_codec::Output + ?Sized>(&self, dest: &mut O) {
        dest.write(self.as_bytes());
    }
}

impl Decode for InterfaceId {
    fn decode<I: parity_scale_codec::Input>(input: &mut I) -> Result<Self, Error> {
        let mut bytes = [0u8; 8];
        input.read(&mut bytes)?;
        let mut slice = bytes.as_slice();
        Self::try_read_bytes(&mut slice).map_err(Error::from)
    }
}

#[derive(Debug, Clone)]
pub struct BaseServiceMeta {
    pub name: &'static str,
    pub interface_id: InterfaceId,
    pub meta: AnyServiceMetaFn,
    pub base: &'static [BaseServiceMeta],
}

impl BaseServiceMeta {
    pub const fn new<S: ServiceMeta>(name: &'static str) -> Self {
        Self {
            name,
            interface_id: S::INTERFACE_ID,
            meta: AnyServiceMeta::new::<S>,
            base: S::BASE_SERVICES,
        }
    }
}

pub trait ServiceMeta: Identifiable {
    type CommandsMeta: StaticTypeInfo;
    type QueriesMeta: StaticTypeInfo;
    type EventsMeta: StaticTypeInfo;
    /// The order of base services here is lexicographical by their names
    const BASE_SERVICES: &'static [BaseServiceMeta];
    /// The order of base services here is lexicographical by their names
    // const BASE_SERVICES_IDS: &'static [AnyServiceIds];
    const ASYNC: bool;

    fn commands() -> MetaType {
        MetaType::new::<Self::CommandsMeta>()
    }

    fn queries() -> MetaType {
        MetaType::new::<Self::QueriesMeta>()
    }

    fn events() -> MetaType {
        MetaType::new::<Self::EventsMeta>()
    }

    fn base_services() -> &'static [BaseServiceMeta] {
        Self::BASE_SERVICES
    }
}

pub struct AnyServiceMeta {
    commands: MetaType,
    queries: MetaType,
    events: MetaType,
    base_services: Vec<BaseServiceMeta>,
    interface_id: InterfaceId,
}

impl AnyServiceMeta {
    pub fn new<S: ServiceMeta>() -> Self {
        Self {
            commands: S::commands(),
            queries: S::queries(),
            events: S::events(),
            base_services: S::BASE_SERVICES.to_vec(),
            interface_id: S::INTERFACE_ID,
        }
    }

    pub fn commands(&self) -> &MetaType {
        &self.commands
    }

    pub fn queries(&self) -> &MetaType {
        &self.queries
    }

    pub fn events(&self) -> &MetaType {
        &self.events
    }

    pub fn base_services(&self) -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        self.base_services
            .iter()
            .map(|base| (base.name, (base.meta)()))
    }

    pub fn interface_id(&self) -> InterfaceId {
        self.interface_id
    }
}

pub trait ProgramMeta {
    type ConstructorsMeta: StaticTypeInfo;
    const SERVICES: &'static [(&'static str, AnyServiceMetaFn)];
    const ASYNC: bool;

    fn constructors() -> MetaType {
        MetaType::new::<Self::ConstructorsMeta>()
    }

    fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        Self::SERVICES.iter().map(|&(s, f)| (s, f()))
    }
}

pub const fn count_base_services<S: ServiceMeta>() -> usize {
    let mut counter = 0;

    let direct_base_services = S::BASE_SERVICES;
    let mut idx = 0;
    while idx != direct_base_services.len() {
        let base = &direct_base_services[idx];
        count_base_services_recursive(&mut counter, base);
        idx += 1;
    }

    counter
}

const fn count_base_services_recursive(counter: &mut usize, base: &BaseServiceMeta) {
    *counter += 1;

    let base_services = base.base;
    let mut idx = 0;
    while idx != base_services.len() {
        count_base_services_recursive(counter, &base_services[idx]);
        idx += 1;
    }
}

/// Generate interface IDs array from exposed services
pub const fn interface_ids<const N: usize>(
    exposed_services: &'static [BaseServiceMeta],
) -> [(InterfaceId, u8); N] {
    let mut output = [(InterfaceId([0u8; 8]), 0u8); N];

    let mut exposed_svc_idx = 0;
    let mut output_offset = 0;
    let mut route_id = 1;
    while exposed_svc_idx != exposed_services.len() {
        let service = &exposed_services[exposed_svc_idx];
        fill_interface_ids_recursive(&mut output, &mut output_offset, service, route_id);
        exposed_svc_idx += 1;
        route_id += 1;
    }

    assert!(output_offset == N, "Mismatched interface IDs count");

    output
}

const fn fill_interface_ids_recursive(
    arr: &mut [(InterfaceId, u8)],
    offset: &mut usize,
    service: &BaseServiceMeta,
    route_id: u8,
) {
    arr[*offset] = (service.interface_id, route_id);
    *offset += 1;
    let base_services = service.base;
    let mut idx = 0;
    while idx != base_services.len() {
        fill_interface_ids_recursive(arr, offset, &base_services[idx], route_id);
        idx += 1;
    }
}

pub const fn service_has_interface_id(
    service: &BaseServiceMeta,
    interface_id: InterfaceId,
) -> bool {
    if service.interface_id.as_u64() == interface_id.as_u64() {
        true
    } else {
        let mut idx = 0;
        while idx != service.base.len() {
            if service_has_interface_id(&service.base[idx], interface_id) {
                return true;
            }
            idx += 1;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_id_codec() {
        let inner = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let id = InterfaceId(inner);

        let encoded = id.encode();
        assert_eq!(inner.encode(), encoded);

        let decoded = Decode::decode(&mut &encoded[..]).unwrap();
        assert_eq!(id, decoded);
    }

    #[test]
    fn interface_id_serde() {
        let inner = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut slice = inner.as_slice();
        let id = InterfaceId::try_read_bytes(&mut slice).unwrap();
        assert_eq!(inner, id.0);
        assert_eq!(slice.len(), 0);
        assert_eq!(id.as_bytes(), inner);
    }

    #[test]
    fn interface_id_try_read_bytes() {
        // Read from a slice with extra data
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut slice = data.as_slice();

        let id = InterfaceId::try_read_bytes(&mut slice).unwrap();
        assert_eq!(id.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(slice, &[9, 10]);

        // Read from a slice with insufficient data
        let data = [1u8, 2, 3, 4, 5, 6, 7];
        let mut slice = data.as_slice();
        let result = InterfaceId::try_read_bytes(&mut slice);
        assert_eq!(result, Err("Insufficient bytes for interface ID"));
    }

    #[test]
    fn interface_id_try_from_bytes() {
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = data.as_slice();

        let id = InterfaceId::try_from_bytes(slice).unwrap();
        assert_eq!(id.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(slice.len(), data.len()); // Original slice should remain unchanged
    }
}
