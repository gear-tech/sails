use crate::prelude::*;
use alloy_primitives::{Address, Selector};
use alloy_sol_types::{abi::TokenSeq, SolType, SolValue};

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
pub type MethodRoute = (&'static str, &'static [u8]);

pub trait ServiceSignature {
    const METHODS: &[MethodRoute];
}

pub trait ProgramSignature {
    const METHODS_LEN: usize;
    const SERVICES: &[(&'static str, &'static [u8], &[MethodRoute])];
    const CTORS: &[MethodRoute];
}

pub fn selector(s: impl AsRef<str>) -> Selector {
    alloy_primitives::keccak256(s.as_ref().as_bytes())[..4]
        .try_into()
        .unwrap()
}

pub const fn const_selector(name: &str) -> [u8; 4] {
    let hash: [u8; 32] = keccak_const::Keccak256::new()
        .update(name.as_bytes())
        .finalize();
    let mut output = [0u8; 4];
    let mut i = 0;
    while i < output.len() {
        output[i] = hash[i];
        i += 1;
    }
    output
}

macro_rules! const_selector {
    () => {
        [0u8; 4]
    };
    ($($s: expr),* $(,)?) => {{
        let mut keccak256 = crate::keccak_const::Keccak256::new();
        $(keccak256 = keccak256.update($s.as_bytes());)*
        let hash: [u8; 32] = keccak256.finalize();
        let mut output = [0u8; 4];
        let mut i = 0;
        while i < output.len() {
            output[i] = hash[i];
            i += 1;
        }
        output
    }};
}

pub struct ConstProgramMeta<T>(marker::PhantomData<T>);

impl<T> ConstProgramMeta<T>
where
    T: ProgramSignature,
{
    pub const fn ctor_sigs<const N: usize>() -> [[u8; 4]; N] {
        let mut sigs = [[0u8; 4]; N];
        let mut ctor_idx = 0;
        while ctor_idx < <T as ProgramSignature>::CTORS.len() {
            let (name, _) = <T as ProgramSignature>::CTORS[ctor_idx];
            sigs[ctor_idx] = const_selector!(name);
            ctor_idx += 1;
        }
        sigs
    }

    pub const fn method_sigs<const N: usize>() -> [[u8; 4]; N] {
        let mut sigs = [[0u8; 4]; N];
        let mut sigs_idx = 0;
        let mut svc_idx = 0;
        while svc_idx < <T as ProgramSignature>::SERVICES.len() {
            let (svc_name, _, methods) = <T as ProgramSignature>::SERVICES[svc_idx];
            let mut method_idx = 0;
            while method_idx < methods.len() {
                let (name, _) = methods[method_idx];
                sigs[sigs_idx] = const_selector!(svc_name, "_", name);
                method_idx += 1;
                sigs_idx += 1;
            }
            svc_idx += 1;
        }
        sigs
    }

    pub const fn method_routes<const N: usize>() -> [(&'static [u8], &'static [u8]); N] {
        let mut routes: [(&'static [u8], &'static [u8]); N] = [(b"", b""); N];
        let mut map_idx = 0;
        let mut svc_idx = 0;
        while svc_idx < <T as ProgramSignature>::SERVICES.len() {
            let (_, svc_route, methods) = <T as ProgramSignature>::SERVICES[svc_idx];
            let mut method_idx = 0;
            while method_idx < methods.len() {
                let (_, route) = methods[method_idx];
                routes[map_idx] = (svc_route, route);
                method_idx += 1;
                map_idx += 1;
            }
            svc_idx += 1;
        }
        routes
    }
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
    fn type_names() {
        let s = <(u32,) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32)", s);

        let s = <(u32, String) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32,string)", s);

        let s = <(u32, String, ActorId2) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32,string,address)", s);
    }

    struct Prg;
    struct Svc;

    impl ServiceSignature for Svc {
        const METHODS: &[MethodRoute] = &[
            (
                concatcp!(
                    "do_this",
                    <<(u32, String) as SolValue>::SolType as SolType>::SOL_NAME,
                ),
                &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            ),
            (
                concatcp!(
                    "this",
                    <<(bool,) as SolValue>::SolType as SolType>::SOL_NAME
                ),
                &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            ),
        ];
    }

    impl solidity::ProgramSignature for Prg {
        const METHODS_LEN: usize = <Svc as solidity::ServiceSignature>::METHODS.len()
            + <Svc as solidity::ServiceSignature>::METHODS.len();

        const CTORS: &[MethodRoute] = &[(
            concatcp!("default", <<() as SolValue>::SolType as SolType>::SOL_NAME,),
            &[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8] as &[u8],
        )];

        const SERVICES: &[(&'static str, &'static [u8], &[MethodRoute])] = &[
            (
                "svc1",
                &[16u8, 83u8, 118u8, 99u8, 49u8] as &[u8],
                <Svc as solidity::ServiceSignature>::METHODS,
            ),
            (
                "svc2",
                &[16u8, 83u8, 118u8, 99u8, 49u8] as &[u8],
                <Svc as solidity::ServiceSignature>::METHODS,
            ),
        ];
    }

    #[test]
    fn program_signature() {
        const S1: [u8; 4] = [16, 223, 169, 238];
        const S2: [u8; 4] = [173, 172, 115, 149];
        let sigs = solidity::ConstProgramMeta::<Prg>::method_sigs::<
            { <Prg as solidity::ProgramSignature>::METHODS_LEN },
        >();
        assert_eq!(4, sigs.len());

        let sig1 = selector("svc1_do_this(uint32,string)");
        assert_eq!(S1, sig1.as_slice());
        assert_eq!(S1, sigs[0]);

        let sig2 = selector("svc1_this(bool)");
        assert_eq!(S2, sig2.as_slice());
        assert_eq!(S2, sigs[1]);

        assert_eq!(Some(0), sigs.iter().position(|s| s == &S1));
        assert_eq!(Some(1), sigs.iter().position(|s| s == &S2));

        let sig3 = selector("svc2_do_this(uint32,string)");
        assert_eq!(Some(2), sigs.iter().position(|s| s == sig3.as_slice()));

        let sig4 = selector("svc2_this(bool)");
        assert_eq!(Some(3), sigs.iter().position(|s| s == sig4.as_slice()));
    }
}
