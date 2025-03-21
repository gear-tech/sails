use crate::prelude::*;
use alloy_primitives::Selector;

pub type MethodRoute = (&'static str, &'static [u8]);

pub trait ServiceSignature {
    const METHODS: &'static [MethodRoute];
}

pub trait ProgramSignature {
    const CTORS: &'static [MethodRoute];
    const SERVICES: &'static [(&'static str, &'static [u8], &'static [MethodRoute])];
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
            let _offset = 0;
            $(let _offset = copy_slice(&mut output, $A, _offset);)*
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
        let mut routes: [(&'static [u8], &'static [u8]); N] = [(&[], &[]); N];
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

#[cfg(test)]
mod tests {
    use super::*;
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
        const METHODS: &[MethodRoute] = &[
            (
                concatcp!(
                    "do_this",
                    <<(u32, String, u128,) as SolValue>::SolType as SolType>::SOL_NAME,
                ),
                &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            ),
            (
                concatcp!(
                    "this",
                    <<(bool, u128,) as SolValue>::SolType as SolType>::SOL_NAME
                ),
                &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            ),
        ];
    }

    impl ServiceSignature for ExtendedSvc {
        const METHODS: &[MethodRoute] = const_concat_slices!(
            <MethodRoute>,
            &[
                (
                    concatcp!(
                        "do_this",
                        <<(u32, String, u128,) as SolValue>::SolType as SolType>::SOL_NAME,
                    ),
                    &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
                ),
                (
                    concatcp!(
                        "this",
                        <<(bool, u128,) as SolValue>::SolType as SolType>::SOL_NAME
                    ),
                    &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
                ),
            ],
            <Svc as ServiceSignature>::METHODS
        );
    }

    impl ProgramSignature for Prg {
        const METHODS_LEN: usize = <Svc as ServiceSignature>::METHODS.len()
            + <ExtendedSvc as ServiceSignature>::METHODS.len();

        const CTORS: &[MethodRoute] = &[(
            concatcp!(
                "default",
                <<(u128,) as SolValue>::SolType as SolType>::SOL_NAME,
            ),
            &[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8] as &[u8],
        )];

        const SERVICES: &[(&'static str, &'static [u8], &[MethodRoute])] = &[
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
            "do_this(uint32,string,uint128)",
            &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
        );
        let this = (
            concatcp!(
                "this",
                <<(bool, u128,) as SolValue>::SolType as SolType>::SOL_NAME
            ),
            &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
        );
        assert_eq!(do_this, ExtendedSvc::METHODS[0]);
        assert_eq!(this, ExtendedSvc::METHODS[1]);
        assert_eq!(do_this, ExtendedSvc::METHODS[2]);
        assert_eq!(this, ExtendedSvc::METHODS[3]);
    }

    #[test]
    fn program_signature() {
        const S1: [u8; 4] = [107, 214, 203, 248];
        const S2: [u8; 4] = [141, 22, 87, 153];
        const SIGS: [[u8; 4]; <Prg as solidity::ProgramSignature>::METHODS_LEN] =
            solidity::ConstProgramMeta::<Prg>::method_sigs();
        assert_eq!(6, SIGS.len());

        let sig1 = selector("svc1_do_this(uint32,string,uint128)");
        assert_eq!(S1, sig1.as_slice());
        assert_eq!(S1, SIGS[0]);

        let sig2 = selector("svc1_this(bool,uint128)");
        assert_eq!(S2, sig2.as_slice());
        assert_eq!(S2, SIGS[1]);

        assert_eq!(Some(0), SIGS.iter().position(|s| s == &S1));
        assert_eq!(Some(1), SIGS.iter().position(|s| s == &S2));

        let sig3 = selector("svc2_do_this(uint32,string,uint128)");
        assert_eq!(Some(2), SIGS.iter().position(|s| s == sig3.as_slice()));

        let sig4 = selector("svc2_this(bool,uint128)");
        assert_eq!(Some(3), SIGS.iter().position(|s| s == sig4.as_slice()));
    }

    #[test]
    fn program_ctor_sigs() {
        const CTOR_SIGS: [[u8; 4]; <Prg as solidity::ProgramSignature>::CTORS.len()] =
            solidity::ConstProgramMeta::<Prg>::ctor_sigs();
        let sig_ctor = selector("default(uint128)");
        assert_eq!(CTOR_SIGS[0], sig_ctor.as_slice());
    }
}
