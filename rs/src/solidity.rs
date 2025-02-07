use crate::collections::BTreeMap;
use crate::prelude::*;
use alloy_primitives::{Address, Selector};
use alloy_sol_types::{SolCall, SolType, SolValue};

pub trait SolFunction {
    /// The function's ABI signature.
    const SIGNATURE: &'static str;

    /// The function selector: `keccak256(SIGNATURE)[0..4]`
    const SELECTOR: &'static [u8];
}

impl<T> SolFunction for T
where
    T: SolCall,
{
    const SIGNATURE: &'static str = T::SIGNATURE;
    const SELECTOR: &'static [u8] = &T::SELECTOR;
}

pub trait SolSignature {
    /// The Solidity type that this type corresponds to.
    type SolType: SolType;
    /// The corresponding Rust type.
    type RustType;
    /// The type signature.
    const SIGNATURE: &'static str;

    fn decode(data: &[u8]) -> Self::RustType;
    fn encode(value: &Self::RustType) -> Vec<u8>;
}

pub struct SolTypeMarker<T> {
    _t: marker::PhantomData<T>,
}

impl<T> SolSignature for SolTypeMarker<T>
where
    T: SolValue,
    T: From<<<T as SolValue>::SolType as SolType>::RustType>,
{
    type SolType = T::SolType;
    type RustType = T;
    const SIGNATURE: &'static str = T::SolType::SOL_NAME;

    fn decode(data: &[u8]) -> Self::RustType {
        <T as SolValue>::abi_decode(data, false).expect("Failed to decode request")
    }

    fn encode(value: &Self::RustType) -> Vec<u8> {
        <T as SolValue>::abi_encode(value)
    }
}

impl SolTypeMarker<ActorId> {
    pub const SIGNATURE: &'static str = "address";

    pub fn decode(data: &[u8]) -> ActorId {
        let value = alloy_sol_types::sol_data::Address::abi_decode(data, false)
            .expect("Failed to decode request");
        let bytes: [u8; 32] = value.into_word().into();
        ActorId::from(bytes)
    }

    pub fn encode(value: ActorId) -> Vec<u8> {
        let bytes: [u8; 32] = value.into_bytes();
        Address::from_word(bytes.into()).abi_encode()
    }
}

pub trait ServiceSignatures {
    fn methods(route: &str) -> impl Iterator<Item = (String, &'static [u8])>;
}

pub trait ProgramSignatures {
    fn constructors() -> impl Iterator<Item = (String, &'static [u8])>;
    fn methods() -> impl Iterator<Item = (String, &'static [u8], &'static [u8])>;

    fn constructors_map() -> BTreeMap<Selector, &'static [u8]> {
        let mut map = BTreeMap::new();
        Self::constructors().into_iter().for_each(|(name, route)| {
            map.insert(selector(&name), route);
        });
        map
    }

    fn methods_map() -> BTreeMap<Selector, (&'static [u8], &'static [u8])> {
        let mut map = BTreeMap::new();
        Self::methods().for_each(|(name, service, method)| {
            map.insert(selector(&name), (service, method));
        });
        map
    }
}

pub fn selector(s: impl AsRef<str>) -> Selector {
    alloy_primitives::keccak256(s.as_ref().as_bytes())[..4]
        .try_into()
        .unwrap()
}
