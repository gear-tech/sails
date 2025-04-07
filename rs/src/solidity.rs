use crate::prelude::*;
use alloy_primitives::Selector;

#[cfg(any(feature = "gtest", all(feature = "gstd", target_arch = "wasm32")))]
pub(crate) const ETH_EVENT_ADDR: gstd::ActorId = gstd::ActorId::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
]);

pub type MethodExpo = (
    &'static [u8], // Method route
    &'static str,  // Method name
    &'static str,  // Method parameters types
    &'static str,  // Method callback parameters types
);

pub type ServiceExpo = (
    &'static str,          // Service expo name
    &'static [u8],         // Service route
    &'static [MethodExpo], // Method routes
);

pub trait ServiceSignature {
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
            let (_, name, params, _) = <T as ProgramSignature>::CTORS[ctor_idx];
            let selector = const_selector!(name, params);
            Self::assert_selector_not_equals_ctor_routes(selector);
            sigs[ctor_idx] = selector;
            ctor_idx += 1;
        }
        sigs
    }

    pub const fn ctor_callback_sigs<const N: usize>() -> [[u8; 4]; N] {
        let mut sigs = [[0u8; 4]; N];
        let mut ctor_idx = 0;
        while ctor_idx < <T as ProgramSignature>::CTORS.len() {
            let (_, name, _, callback) = <T as ProgramSignature>::CTORS[ctor_idx];
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
                let (_, name, params, _) = methods[method_idx];
                let selector = const_selector!(svc_name, name, params);
                Self::assert_selector_not_equals_method_routes(selector);
                sigs[sigs_idx] = selector;
                method_idx += 1;
                sigs_idx += 1;
            }
            svc_idx += 1;
        }
        sigs
    }

    pub const fn method_routes<const N: usize>() -> [(&'static [u8], &'static [u8]); N] {
        let mut routes: [(&'static [u8], &'static [u8]); N] = [(&[], &[]); N];
        let mut map_idx = 0;
        let mut svc_idx = 0;
        while svc_idx < <T as ProgramSignature>::SERVICES.len() {
            let (_, svc_route, methods) = <T as ProgramSignature>::SERVICES[svc_idx];
            let mut method_idx = 0;
            while method_idx < methods.len() {
                let (route, ..) = methods[method_idx];
                routes[map_idx] = (svc_route, route);
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
                let (_, name, _, callback) = methods[method_idx];
                sigs[sigs_idx] = const_selector!("replyOn_", svc_name, name, callback);
                method_idx += 1;
                sigs_idx += 1;
            }
            svc_idx += 1;
        }
        sigs
    }

    const fn assert_selector_not_equals_method_routes(selector: [u8; 4]) {
        let mut svc_idx = 0;
        while svc_idx < <T as ProgramSignature>::SERVICES.len() {
            let (_, svc_route, methods) = <T as ProgramSignature>::SERVICES[svc_idx];
            let mut method_idx = 0;
            while method_idx < methods.len() {
                let (route, ..) = methods[method_idx];
                assert!(!selector_equals(selector, svc_route, route));
                method_idx += 1;
            }
            svc_idx += 1;
        }
    }

    const fn assert_selector_not_equals_ctor_routes(selector: [u8; 4]) {
        let mut ctor_idx = 0;
        while ctor_idx < <T as ProgramSignature>::CTORS.len() {
            let (route, ..) = <T as ProgramSignature>::CTORS[ctor_idx];
            assert!(!selector_equals(selector, route, &[]));
            ctor_idx += 1;
        }
    }
}

/// Compares a 4-byte selector array against the concatenation of two input byte slices.
///
/// This function checks whether the **first 4 bytes** of the concatenated `first` and `second` slices
/// exactly match the `selector`. It does not require `first` or `second` to be of any specific length,
/// but the **combined length must be at least 4** for the function to return `true`.
///
/// Any mismatch in the first 4 bytes results in `false`.
const fn selector_equals(selector: [u8; 4], first: &[u8], second: &[u8]) -> bool {
    let mut i = 0;
    while i < first.len() && i < selector.len() {
        if selector[i] != first[i] {
            return false;
        }
        i += 1;
    }
    let mut j = 0;
    while j < second.len() && i < selector.len() {
        if selector[i] != second[j] {
            return false;
        }
        i += 1;
        j += 1;
    }
    // False if we didn't match full len
    i == selector.len()
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

    impl ServiceSignature for Svc {
        const METHODS: &[MethodExpo] = &[
            (
                &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
                "DoThis",
                <<(u32, String, u128) as SolValue>::SolType as SolType>::SOL_NAME,
                <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
            ),
            (
                &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
                "This",
                <<(bool, u128) as SolValue>::SolType as SolType>::SOL_NAME,
                <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
            ),
        ];
    }

    impl ServiceSignature for ExtendedSvc {
        const METHODS: &[MethodExpo] = const_concat_slices!(
            <MethodExpo>,
            &[
                (
                    &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
                    "DoThis",
                    <<(u32, String, u128,) as SolValue>::SolType as SolType>::SOL_NAME,
                    <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
                ),
                (
                    &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
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
            &[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8] as &[u8],
            "create",
            <<(u128,) as SolValue>::SolType as SolType>::SOL_NAME,
            <<(B256,) as SolValue>::SolType as SolType>::SOL_NAME,
        )];

        const SERVICES: &[ServiceExpo] = &[
            (
                "svc1",
                &[16u8, 83u8, 118u8, 99u8, 49u8] as &[u8],
                <Svc as ServiceSignature>::METHODS,
            ),
            (
                "svc2",
                &[16u8, 83u8, 118u8, 99u8, 49u8] as &[u8],
                <ExtendedSvc as ServiceSignature>::METHODS,
            ),
        ];
    }

    #[test]
    fn service_signature_extended() {
        assert_eq!(4, ExtendedSvc::METHODS.len());

        let do_this = (
            &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            "DoThis",
            "(uint32,string,uint128)",
            "(bytes32,uint32)",
        );
        let this = (
            &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            "This",
            <<(bool, u128) as SolValue>::SolType as SolType>::SOL_NAME,
            <<(B256, u32) as SolValue>::SolType as SolType>::SOL_NAME,
        );
        assert_eq!(do_this, ExtendedSvc::METHODS[0]);
        assert_eq!(this, ExtendedSvc::METHODS[1]);
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
    fn selector_equals_test_exact_match() {
        let sel = *b"ABCD";
        assert!(selector_equals(sel, b"AB", b"CD"));
    }

    #[test]
    fn selector_equals_test_longer_second() {
        let sel = *b"ABCD";
        assert!(selector_equals(sel, b"AB", b"CDEF"));
    }

    #[test]
    fn selector_equals_test_longer_first() {
        let sel = *b"ABCD";
        assert!(selector_equals(sel, b"ABCDX", b""));
    }

    #[test]
    fn selector_equals_test_short_inputs() {
        let sel = *b"ABCD";
        assert!(selector_equals(sel, b"A", b"BCD"));
        assert!(selector_equals(sel, b"", b"ABCD"));
    }

    #[test]
    fn selector_equals_test_mismatch_in_first() {
        let sel = *b"XBCD";
        assert!(!selector_equals(sel, b"AB", b"CD"));
    }

    #[test]
    fn selector_equals_test_mismatch_in_second() {
        let sel = *b"ABXD";
        assert!(!selector_equals(sel, b"AB", b"CD"));
    }

    #[test]
    fn selector_equals_test_mismatch_due_to_short_concat() {
        let sel = *b"ABCD";
        assert!(!selector_equals(sel, b"A", b"B")); // only "AB" < 4 bytes
    }
}
