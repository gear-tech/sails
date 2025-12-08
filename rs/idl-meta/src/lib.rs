#![no_std]

extern crate alloc;

#[cfg(feature = "ast")]
mod ast;

#[cfg(feature = "ast")]
pub use ast::*;
use parity_scale_codec::{Decode, Encode, Error};
use scale_info::{MetaType, StaticTypeInfo, prelude::vec::Vec};

pub type AnyServiceMetaFn = fn() -> AnyServiceMeta;

/// Unique identifier for a service (or "interface" in terms of sails binary protocol).
///
/// For more information about interface IDs, see the interface ID spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterfaceId(pub [u8; 8]);

impl InterfaceId {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 8] {
        self.0
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

impl Encode for InterfaceId {
    fn encode_to<O: parity_scale_codec::Output + ?Sized>(&self, dest: &mut O) {
        dest.write(&self.to_bytes());
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

pub trait ServiceMeta {
    type CommandsMeta: StaticTypeInfo;
    type QueriesMeta: StaticTypeInfo;
    type EventsMeta: StaticTypeInfo;
    const BASE_SERVICES: &'static [AnyServiceMetaFn];
    const ASYNC: bool;
    const INTERFACE_ID: InterfaceId;

    fn commands() -> MetaType {
        MetaType::new::<Self::CommandsMeta>()
    }

    fn queries() -> MetaType {
        MetaType::new::<Self::QueriesMeta>()
    }

    fn events() -> MetaType {
        MetaType::new::<Self::EventsMeta>()
    }

    fn base_services() -> impl Iterator<Item = AnyServiceMeta> {
        Self::BASE_SERVICES.iter().map(|f| f())
    }
}

pub struct AnyServiceMeta {
    commands: MetaType,
    queries: MetaType,
    events: MetaType,
    base_services: Vec<AnyServiceMeta>,
}

impl AnyServiceMeta {
    pub fn new<S: ServiceMeta>() -> Self {
        Self {
            commands: S::commands(),
            queries: S::queries(),
            events: S::events(),
            base_services: S::base_services().collect(),
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

    pub fn base_services(&self) -> impl Iterator<Item = &AnyServiceMeta> {
        self.base_services.iter()
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
        Self::SERVICES.iter().map(|(s, f)| (*s, f()))
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
        assert_eq!(id.to_bytes(), inner);
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
