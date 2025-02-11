use crate::collections::BTreeMap;
use crate::prelude::*;
use alloy_primitives::{Address, Selector};
use alloy_sol_types::{abi::TokenSeq, SolCall, SolType, SolValue};

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

    fn decode<'a>(data: &'a [u8]) -> Self::RustType
    where
        <Self::SolType as SolType>::Token<'a>: TokenSeq<'a>;

    fn encode(value: &Self::RustType) -> Vec<u8>
    where
        for<'a> <Self::SolType as SolType>::Token<'a>: TokenSeq<'a>;
}

pub struct SolTypeMarker<T, V> {
    _t: marker::PhantomData<T>,
    _v: marker::PhantomData<V>,
}

impl<T> SolSignature for SolTypeMarker<T, T>
where
    T: SolValue,
    T: From<<<T as SolValue>::SolType as SolType>::RustType>,
{
    type SolType = T::SolType;
    type RustType = T;
    const SIGNATURE: &'static str = T::SolType::SOL_NAME;

    fn decode<'a>(data: &'a [u8]) -> T
    where
        <Self::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        Self::SolType::abi_decode_params(data, false)
            .expect("Failed to decode request")
            .into()
    }

    fn encode(value: &T) -> Vec<u8>
    where
        for<'a> <Self::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        Self::SolType::abi_encode_params(value)
    }
}

impl SolSignature for SolTypeMarker<ActorId, Address> {
    type SolType = <Address as SolValue>::SolType;
    type RustType = ActorId;
    const SIGNATURE: &'static str = <Address as SolValue>::SolType::SOL_NAME;

    fn decode<'a>(data: &'a [u8]) -> ActorId
    where
        <Self::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        let value =
            Self::SolType::abi_decode_params(data, false).expect("Failed to decode request");
        let bytes: [u8; 32] = value.into_word().into();
        ActorId::from(bytes)
    }

    fn encode(value: &ActorId) -> Vec<u8>
    where
        for<'a> <Self::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        let bytes: [u8; 32] = value.into_bytes();
        Address::from_word(bytes.into()).abi_encode()
        // Self::SolType::abi_encode_params(value)
    }
}

// impl SolTypeMarker<ActorId> {
//     pub const SIGNATURE: &'static str = "address";

//     pub fn decode(data: &[u8]) -> ActorId {
//         let value = alloy_sol_types::sol_data::Address::abi_decode(data, false)
//             .expect("Failed to decode request");
//         let bytes: [u8; 32] = value.into_word().into();
//         ActorId::from(bytes)
//     }

//     pub fn encode(value: ActorId) -> Vec<u8> {
//         let bytes: [u8; 32] = value.into_bytes();
//         Address::from_word(bytes.into()).abi_encode()
//     }
// }

pub trait ServiceSignatures {
    const METHODS: &[(&'static str, &'static [u8])];
}

pub trait ProgramSignatures {
    const METHODS: &[(
        &'static str,
        &'static [u8],
        &[(&'static str, &'static [u8])],
    )];
    const CONSTRUCTORS: &[(&'static str, &'static [u8])];

    fn constructors_map() -> BTreeMap<Selector, &'static [u8]> {
        let mut map = BTreeMap::new();
        Self::CONSTRUCTORS.into_iter().for_each(|(name, route)| {
            map.insert(selector(name), *route);
        });
        map
    }

    fn methods_map() -> BTreeMap<Selector, (&'static [u8], &'static [u8])> {
        let mut map = BTreeMap::new();
        Self::METHODS
            .into_iter()
            .for_each(|(svc_name, svc_route, methods)| {
                methods.into_iter().for_each(|(name, route)| {
                    map.insert(
                        selector(format!("{}_{}", svc_name, name)),
                        (*svc_route, *route),
                    );
                });
            });
        map
    }
}

pub fn selector(s: impl AsRef<str>) -> Selector {
    alloy_primitives::keccak256(s.as_ref().as_bytes())[..4]
        .try_into()
        .unwrap()
}

pub struct ActorId2(pub ActorId);

impl From<Address> for ActorId2 {
    fn from(value: Address) -> Self {
        let bytes: [u8; 32] = value.into_word().into();
        ActorId2(ActorId::from(bytes))
    }
}

impl SolValue for ActorId2 {
    type SolType = <alloy_primitives::Address as SolValue>::SolType;
}

impl ::alloy_sol_types::private::SolTypeValue<alloy_sol_types::sol_data::Address> for ActorId2 {
    #[inline]
    fn stv_to_tokens(&self) -> alloy_sol_types::abi::token::WordToken {
        let bytes = self.0.into_bytes();
        ::alloy_sol_types::abi::token::WordToken(::alloy_sol_types::Word::from(bytes))
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        let bytes: &[u8] = &self.0.into_bytes()[12..];
        out.extend_from_slice(bytes);
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
        ::alloy_sol_types::private::SolTypeValue::<alloy_sol_types::sol_data::Address>::stv_to_tokens(self).0
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U160;

    use super::*;

    #[test]
    fn actor_encode_decode() {
        let address = Address::from(U160::from(42));
        let address_encoded = address.abi_encode();

        let bytes: [u8; 32] = address.into_word().into();
        let id = ActorId::from(bytes);
        let actor_encoded = ActorId2(id).abi_encode();

        assert_eq!(address_encoded.as_slice(), actor_encoded.as_slice());

        let actor2 = ActorId2::abi_decode(actor_encoded.as_slice(), false);
        assert_eq!(id, actor2.unwrap().0);

        let address2 = Address::abi_decode(actor_encoded.as_slice(), false);
        assert_eq!(address, address2.unwrap());
    }

    #[test]
    fn sig() {
        let s = <(u32,) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32)", s);

        let s = <(u32, String) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32,string)", s);

        let s = <(u32, String, ActorId2) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32,string,address)", s);
    }
}
