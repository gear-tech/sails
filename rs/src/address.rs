use crate::{
    alloy_sol_types::{SolValue, Word, abi::token::WordToken, private::SolTypeValue, sol_data},
    prelude::*,
};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Default, TypeInfo,
)]
#[codec(crate = crate::scale_codec)]
#[annotate(sol_type = "address")]
pub struct Address(pub H160);

impl Address {
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Address(H160(bytes))
    }
}

impl SolValue for Address {
    type SolType = sol_data::Address;
}

impl SolTypeValue<sol_data::Address> for Address {
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        let mut word = [0u8; 32];
        word[12..].copy_from_slice(self.0.as_fixed_bytes());
        WordToken(Word::from(word))
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.0.as_fixed_bytes());
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> Word {
        <Address as SolTypeValue<sol_data::Address>>::stv_to_tokens(self).0
    }
}

impl SolTypeValue<sol_data::Address> for &Address {
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        <Address as SolTypeValue<sol_data::Address>>::stv_to_tokens(self)
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        <Address as SolTypeValue<sol_data::Address>>::stv_abi_encode_packed_to(self, out)
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> Word {
        <Address as SolTypeValue<sol_data::Address>>::stv_eip712_data_word(self)
    }
}

impl From<ActorId> for Address {
    fn from(value: ActorId) -> Self {
        Address(value.to_address_lossy())
    }
}

impl From<Address> for ActorId {
    fn from(value: Address) -> Self {
        ActorId::from(value.0)
    }
}

impl From<H160> for Address {
    fn from(value: H160) -> Self {
        Address(value)
    }
}

impl From<Address> for H160 {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Address(H160::from(bytes))
    }
}

impl From<u64> for Address {
    fn from(value: u64) -> Self {
        Address(ActorId::from(value).to_address_lossy())
    }
}

impl From<alloy_primitives::Address> for Address {
    fn from(value: alloy_primitives::Address) -> Self {
        Address(H160::from(value.0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_info_has_soltype_address_annotation() {
        use sails_type_registry::Registry;
        let mut registry = Registry::new();
        let ty = Address::type_def(&mut registry).expect("Address type is not registered");
        assert_eq!(ty.name, "Address");
        assert!(
            ty.annotations
                .iter()
                .any(|a| a.0 == "sol_type" && a.1.as_deref() == Some("address")),
            "expected @sol_type: address annotation on Address type"
        );
    }

    fn word_bytes(w: Word) -> [u8; 32] {
        w.0
    }

    fn addr_from(bytes20: [u8; 20]) -> Address {
        Address::from(bytes20)
    }

    #[test]
    fn stv_to_tokens_left_pad_12_zeros_and_copy_last_20_bytes() {
        let b20 = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x10, 0x20, 0x30, 0x40, 0x50,
        ];
        let a = addr_from(b20);

        let w = <Address as SolTypeValue<sol_data::Address>>::stv_eip712_data_word(&a);
        let bytes32 = word_bytes(w);

        assert_eq!(&bytes32[0..12], &[0u8; 12]);
        assert_eq!(&bytes32[12..32], &b20);
    }

    #[test]
    fn stv_abi_encode_packed_writes_exact_20_bytes_and_appends() {
        let b20 = [0xAB; 20];
        let a = addr_from(b20);

        let mut out = Vec::from([0x01, 0x02, 0x03]);
        <Address as SolTypeValue<sol_data::Address>>::stv_abi_encode_packed_to(&a, &mut out);

        assert_eq!(out.len(), 3 + 20);
        assert_eq!(&out[0..3], &[0x01, 0x02, 0x03]);
        assert_eq!(&out[3..], &b20);
    }

    #[test]
    fn stv_eip712_word_equals_to_tokens_word() {
        let b20 = [0x01; 20];
        let a = addr_from(b20);

        let w1 = <Address as SolTypeValue<sol_data::Address>>::stv_eip712_data_word(&a);
        let w2 = <Address as SolTypeValue<sol_data::Address>>::stv_to_tokens(&a).0;

        assert_eq!(word_bytes(w1), word_bytes(w2));
    }

    #[test]
    fn soltypevalue_for_ref_delegates_to_owned_impl() {
        let b20 = [0x7F; 20];
        let a = addr_from(b20);
        let aref = &a;

        let w_owned = <Address as SolTypeValue<sol_data::Address>>::stv_eip712_data_word(&a);
        let w_ref = <&Address as SolTypeValue<sol_data::Address>>::stv_eip712_data_word(&aref);
        assert_eq!(word_bytes(w_owned), word_bytes(w_ref));

        let mut o1 = Vec::new();
        let mut o2 = Vec::new();
        <Address as SolTypeValue<sol_data::Address>>::stv_abi_encode_packed_to(&a, &mut o1);
        <&Address as SolTypeValue<sol_data::Address>>::stv_abi_encode_packed_to(&aref, &mut o2);
        assert_eq!(o1, o2);

        let t_owned = <Address as SolTypeValue<sol_data::Address>>::stv_to_tokens(&a);
        let t_ref = <&Address as SolTypeValue<sol_data::Address>>::stv_to_tokens(&aref);
        assert_eq!(word_bytes(t_owned.0), word_bytes(t_ref.0));
    }

    #[test]
    fn from_address_to_actorid_and_back_is_identity_for_address() {
        let b20 = [0x42; 20];
        let a1 = addr_from(b20);

        let actor: ActorId = a1.into();
        let a2: Address = actor.into();

        assert_eq!(a2.0.as_fixed_bytes(), a1.0.as_fixed_bytes());
    }

    #[test]
    fn from_h160_and_into_h160_roundtrip() {
        let b20 = [0x13; 20];
        let h = H160::from(b20);

        let a: Address = h.into();
        let h2: H160 = a.into();

        assert_eq!(h2, H160::from(b20));
    }

    #[test]
    fn from_bytes20_constructs_expected_h160() {
        let b20 = [0x99; 20];
        let a = Address::from(b20);
        assert_eq!(a.0, H160::from(b20));
    }

    #[test]
    fn from_u64_matches_actorid_to_address_lossy() {
        let x: u64 = 123456789;
        let a1: Address = x.into();
        let a2 = Address(ActorId::from(x).to_address_lossy());
        assert_eq!(a1.0.as_fixed_bytes(), a2.0.as_fixed_bytes());
    }

    #[test]
    fn from_alloy_primitives_address_copies_bytes() {
        let b20 = [0xA5; 20];

        let alloy_addr = alloy_primitives::Address::from(b20);
        let a: Address = alloy_addr.into();

        assert_eq!(a.0.as_fixed_bytes(), &b20);
    }
}
