use crate::prelude::*;
use alloy_primitives::Selector;
use sails_idl_meta::{InterfaceId, ServiceMeta};

#[cfg(any(feature = "gtest", all(feature = "gstd", target_arch = "wasm32")))]
pub(crate) const ETH_EVENT_ADDR: gstd::ActorId = gstd::ActorId::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
]);

pub type MethodExpo = (
    InterfaceId,  // Service interface id
    u16,          // Method entry id
    &'static str, // Method name
    &'static str, // Method parameters types
    &'static str, // Method callback parameters types
);

pub type ServiceExpo = (
    &'static str,          // Service expo name
    u8,                    // Service route idx
    &'static [MethodExpo], // Method routes
);

pub trait ServiceSignature: ServiceMeta {
    const METHODS: &'static [MethodExpo];
}

pub trait ProgramSignature {
    const CTORS: &'static [MethodExpo];
    const SERVICES: &'static [ServiceExpo];
    const METHODS_LEN: usize;
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

#[macro_export]
macro_rules! const_concat_slices {
    (<$T:ty>, $($A:expr),+ $(,)?) => {{
        use core::mem::MaybeUninit;

        const LEN: usize = $( $A.len() + )* 0;
        const fn combined() -> [$T; LEN] {
            let mut output: [MaybeUninit<$T>; LEN] = [const { MaybeUninit::uninit() }; LEN];
            let offset = 0;
            $(let offset = copy_slice(&mut output, $A, offset);)*
            assert!(offset == LEN);
            unsafe { core::mem::transmute::<_, [$T; LEN]>(output) }
        }
        const fn copy_slice(output: &mut [MaybeUninit<$T>], input: &[$T], offset: usize) -> usize {
            let mut index = 0;
            while index < input.len() {
                output[offset + index].write(input[index]);
                index += 1;
            }
            offset + index
        }
        const RESULT: &[$T] = &combined();
        RESULT
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
            let (_, _, name, params, _) = <T as ProgramSignature>::CTORS[ctor_idx];
            let selector = const_selector!(name, params);
            sigs[ctor_idx] = selector;
            ctor_idx += 1;
        }
        sigs
    }

    pub const fn ctor_callback_sigs<const N: usize>() -> [[u8; 4]; N] {
        let mut sigs = [[0u8; 4]; N];
        let mut ctor_idx = 0;
        while ctor_idx < <T as ProgramSignature>::CTORS.len() {
            let (_, _, name, _, callback) = <T as ProgramSignature>::CTORS[ctor_idx];
            sigs[ctor_idx] = const_selector!("replyOn_", name, callback);
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
                let (_, _, name, params, _) = methods[method_idx];
                let selector = const_selector!(svc_name, name, params);
                sigs[sigs_idx] = selector;
                method_idx += 1;
                sigs_idx += 1;
            }
            svc_idx += 1;
        }
        sigs
    }

    pub const fn method_routes<const N: usize>() -> [(InterfaceId, u16, u8); N] {
        let mut routes: [(InterfaceId, u16, u8); N] = [(InterfaceId::zero(), 0, 0); N];
        let mut map_idx = 0;
        let mut svc_idx = 0;
        while svc_idx < <T as ProgramSignature>::SERVICES.len() {
            let (_, route_idx, methods) = <T as ProgramSignature>::SERVICES[svc_idx];
            let mut method_idx = 0;
            while method_idx < methods.len() {
                let (interface_id, entry_id, ..) = methods[method_idx];
                routes[map_idx] = (interface_id, entry_id, route_idx);
                method_idx += 1;
                map_idx += 1;
            }
            svc_idx += 1;
        }
        routes
    }

    pub const fn callback_sigs<const N: usize>() -> [[u8; 4]; N] {
        let mut sigs = [[0u8; 4]; N];
        let mut sigs_idx = 0;
        let mut svc_idx = 0;
        while svc_idx < <T as ProgramSignature>::SERVICES.len() {
            let (svc_name, _, methods) = <T as ProgramSignature>::SERVICES[svc_idx];
            let mut method_idx = 0;
            while method_idx < methods.len() {
                let (_, _, name, _, callback) = methods[method_idx];
                sigs[sigs_idx] = const_selector!("replyOn_", svc_name, name, callback);
                method_idx += 1;
                sigs_idx += 1;
            }
            svc_idx += 1;
        }
        sigs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;
    use alloy_sol_types::{SolType, SolValue};

    #[test]
    fn type_names() {
        let s = <() as SolValue>::SolType::SOL_NAME;
        assert_eq!("()", s);

        let s = <(u32,) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32)", s);

        let s = <(u32, String) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(uint32,string)", s);

        let s = <(Vec<u8>, String) as SolValue>::SolType::SOL_NAME;
        assert_eq!("(bytes,string)", s);

        // let s = <(u32, String, ActorId) as SolValue>::SolType::SOL_NAME;
        // assert_eq!("(uint32,string,address)", s);
    }

    struct Prg;
    struct Svc;
    struct ExtendedSvc;
    #[derive(crate::TypeInfo)]
    enum Empty {}

    impl ServiceMeta for Svc {
        type CommandsMeta = Empty;
        type QueriesMeta = Empty;
        type EventsMeta = Empty;
        const BASE_SERVICES: &'static [sails_idl_meta::BaseServiceMeta] = &[];
        const ASYNC: bool = false;
        const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(1);
    }

    impl ServiceSignature for Svc {
        const METHODS: &[MethodExpo] = &[
            (
                InterfaceId::from_u64(1),
                0,
                "DoThis",
                <<(u32, String, u128) as SolValue>::SolType as SolType>::SOL_NAME,
                <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
            ),
            (
                InterfaceId::from_u64(1),
                1,
                "This",
                <<(bool, u128) as SolValue>::SolType as SolType>::SOL_NAME,
                <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
            ),
        ];
    }

    impl ServiceMeta for ExtendedSvc {
        type CommandsMeta = Empty;
        type QueriesMeta = Empty;
        type EventsMeta = Empty;
        const BASE_SERVICES: &'static [sails_idl_meta::BaseServiceMeta] = &[];
        const ASYNC: bool = false;
        const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(2);
    }

    impl ServiceSignature for ExtendedSvc {
        const METHODS: &[MethodExpo] = const_concat_slices!(
            <MethodExpo>,
            &[
                (
                    InterfaceId::from_u64(2),
                    0,
                    "DoThis",
                    <<(u32, String, u128,) as SolValue>::SolType as SolType>::SOL_NAME,
                    <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
                ),
                (
                    InterfaceId::from_u64(2),
                    1,
                    "This",
                    <<(bool, u128,) as SolValue>::SolType as SolType>::SOL_NAME,
                    <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
                ),
            ],
            <Svc as ServiceSignature>::METHODS
        );
    }

    impl ProgramSignature for Prg {
        const METHODS_LEN: usize = <Svc as ServiceSignature>::METHODS.len()
            + <ExtendedSvc as ServiceSignature>::METHODS.len();

        const CTORS: &[MethodExpo] = &[(
            InterfaceId::zero(),
            0,
            "create",
            <<(u128,) as SolValue>::SolType as SolType>::SOL_NAME,
            <<(B256,) as SolValue>::SolType as SolType>::SOL_NAME,
        )];

        const SERVICES: &[ServiceExpo] = &[
            ("svc1", 1, <Svc as ServiceSignature>::METHODS),
            ("svc2", 2, <ExtendedSvc as ServiceSignature>::METHODS),
        ];
    }

    #[test]
    fn service_signature_extended() {
        assert_eq!(4, ExtendedSvc::METHODS.len());

        let do_this_ext = (
            InterfaceId::from_u64(2),
            0,
            "DoThis",
            "(uint32,string,uint128)",
            "(bytes32,uint32)",
        );
        let this_ext = (
            InterfaceId::from_u64(2),
            1,
            "This",
            <<(bool, u128) as SolValue>::SolType as SolType>::SOL_NAME,
            <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
        );
        let do_this = (
            InterfaceId::from_u64(1),
            0,
            "DoThis",
            "(uint32,string,uint128)",
            "(bytes32,uint32)",
        );
        let this = (
            InterfaceId::from_u64(1),
            1,
            "This",
            <<(bool, u128) as SolValue>::SolType as SolType>::SOL_NAME,
            <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
        );
        assert_eq!(do_this_ext, ExtendedSvc::METHODS[0]);
        assert_eq!(this_ext, ExtendedSvc::METHODS[1]);
        assert_eq!(do_this, ExtendedSvc::METHODS[2]);
        assert_eq!(this, ExtendedSvc::METHODS[3]);
    }

    #[test]
    fn program_signature() {
        const S1: [u8; 4] = [236, 140, 92, 145];
        const S2: [u8; 4] = [27, 178, 77, 160];
        const SIGS: [[u8; 4]; <Prg as solidity::ProgramSignature>::METHODS_LEN] =
            solidity::ConstProgramMeta::<Prg>::method_sigs();
        assert_eq!(6, SIGS.len());

        let sig1 = selector("svc1DoThis(uint32,string,uint128)");
        assert_eq!(S1, sig1.as_slice());
        assert_eq!(S1, SIGS[0]);

        let sig2 = selector("svc1This(bool,uint128)");
        assert_eq!(S2, sig2.as_slice());
        assert_eq!(S2, SIGS[1]);

        assert_eq!(Some(0), SIGS.iter().position(|s| s == &S1));
        assert_eq!(Some(1), SIGS.iter().position(|s| s == &S2));

        let sig3 = selector("svc2DoThis(uint32,string,uint128)");
        assert_eq!(Some(2), SIGS.iter().position(|s| s == sig3.as_slice()));

        let sig4 = selector("svc2This(bool,uint128)");
        assert_eq!(Some(3), SIGS.iter().position(|s| s == sig4.as_slice()));
    }

    #[test]
    fn program_ctor_sigs() {
        const CTOR_SIGS: [[u8; 4]; <Prg as solidity::ProgramSignature>::CTORS.len()] =
            solidity::ConstProgramMeta::<Prg>::ctor_sigs();
        let sig_ctor = selector("create(uint128)");
        assert_eq!(CTOR_SIGS[0], sig_ctor.as_slice());
    }

    #[test]
    fn program_ctor_callback_sigs() {
        const CTOR_CALLBACK_SIGS: [[u8; 4]; <Prg as solidity::ProgramSignature>::CTORS.len()] =
            solidity::ConstProgramMeta::<Prg>::ctor_callback_sigs();
        let sig_ctor = selector("replyOn_create(bytes32)");
        assert_eq!(CTOR_CALLBACK_SIGS[0], sig_ctor.as_slice());
    }

    #[test]
    fn encode_decode_sol_types() {
        let original = (false, ActorId::zero(), [1u8, 2, 3, 4]);
        let input = original.clone().abi_encode_sequence();

        type ActorType = <<ActorId as SolValue>::SolType as SolType>::RustType;
        type ArrayType = <<[u8; 4] as SolValue>::SolType as SolType>::RustType;
        let decoded: (bool, ActorType, ArrayType) =
            SolValue::abi_decode_sequence(&input).expect("decode failed");

        let result: (bool, ActorId, [u8; 4]) = (decoded.0, decoded.1.into(), decoded.2.into());
        assert_eq!(original, result);
    }
}
