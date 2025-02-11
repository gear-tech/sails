#![no_std]
use sails_rs::alloy_sol_types::SolType;
use sails_rs::alloy_sol_types::SolValue;
use sails_rs::gstd::debug;
use sails_rs::prelude::*;

pub struct MyProgram;

static __ROUTE_SVC1: [u8; 5usize] = [16u8, 83u8, 118u8, 99u8, 49u8];

impl MyProgram {
    pub fn default() -> Self {
        Self
    }

    pub fn svc1(&self) -> <SomeService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__svc1();
        let exposure = <SomeService as sails_rs::gstd::services::Service>::expose(
            service,
            sails_rs::gstd::msg::id().into(),
            __ROUTE_SVC1.as_ref(),
        );
        exposure
    }
    fn __svc1(&self) -> SomeService {
        SomeService
    }
}
const _: () = {
    impl sails_rs::meta::ProgramMeta for MyProgram {
        fn constructors() -> sails_rs::scale_info::MetaType {
            sails_rs::scale_info::MetaType::new::<meta_in_program::ConstructorsMeta>()
        }
        fn services() -> impl Iterator<Item = (&'static str, sails_rs::meta::AnyServiceMeta)> {
            [("Svc1", sails_rs::meta::AnyServiceMeta::new::<SomeService>())].into_iter()
        }
    }

    impl solidity::ProgramSignatures for MyProgram {
        fn constructors() -> impl Iterator<Item = (String, &'static [u8])> {
            [(
                format!("default()"),
                &[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8] as &[u8],
            )]
            .into_iter()
        }

        fn methods() -> impl Iterator<Item = (String, &'static [u8], &'static [u8])> {
            <SomeService as solidity::ServiceSignatures>::methods("svc1")
                .into_iter()
                .map(|(sig, route)| (sig, &__ROUTE_SVC1 as &[u8], route))
        }
    }
};

use sails_rs::Decode as __ProgramDecode;
use sails_rs::TypeInfo as __ProgramTypeInfo;
#[derive(__ProgramDecode, __ProgramTypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
#[allow(dead_code)]
struct __DefaultParams {}

mod meta_in_program {
    use super::*;
    #[derive(__ProgramTypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum ConstructorsMeta {
        Default(__DefaultParams),
    }
}
// #[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use sails_rs::{
        alloy_primitives::Selector,
        collections::btree_map::BTreeMap,
        gstd::{self, services},
        hex,
        meta::ProgramMeta,
        prelude::*,
    };
    static mut PROGRAM: Option<MyProgram> = None;
    static mut __SOL_SIGNATURES: BTreeMap<Selector, (&'static [u8], &'static [u8])> =
        BTreeMap::new();

    #[gstd::async_init]
    async fn init() {
        // sails_rs::gstd::events::__enable_events();
        let input: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");

        let ctors = <MyProgram as solidity::ProgramSignatures>::constructors_map();
        unsafe {
            __SOL_SIGNATURES = <MyProgram as solidity::ProgramSignatures>::methods_map();
        }

        if let Ok(sig) = TryInto::<[u8; 4]>::try_into(&input[..4]) {
            if let Some(&ctor) = ctors.get(&sig) {
                unsafe {
                    PROGRAM = match_ctor_solidity(ctor, &input[4..]).await;
                }
                gstd::msg::reply_bytes(&[], 0).expect("Failed to send output");
            }
        }
        if unsafe { PROGRAM.is_none() } {
            let (program, invocation_route) =
                if input.starts_with(&[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8]) {
                    static INVOCATION_ROUTE: [u8; 8usize] =
                        [28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8];
                    let request = __DefaultParams::decode(&mut &input[8usize..])
                        .expect("Failed to decode request");
                    let program = MyProgram::default();
                    (program, INVOCATION_ROUTE.as_ref())
                } else {
                    sails_rs::gstd::unknown_input_panic("Unexpected ctor", input)
                };

            unsafe {
                PROGRAM = Some(program);
            }
            gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
        }
    }

    async fn match_ctor_solidity(ctor: &[u8], input: &[u8]) -> Option<MyProgram> {
        match ctor {
            &[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8] => {
                // let _request = <()>::decode(input);
                let program = MyProgram::default();
                Some(program)
            }
            _ => None,
        }
    }

    #[gstd::async_main]
    async fn main() {
        let input: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
        let program_ref = unsafe { PROGRAM.as_ref() }.expect("Program not initialized");

        if let Ok(sig) = TryInto::<[u8; 4]>::try_into(&input[..4]) {
            if let Some(&(route, method)) = unsafe { __SOL_SIGNATURES.get(&sig) } {
                if route == &__ROUTE_SVC1 {
                    let mut service = program_ref.svc1();
                    let (output, value) = service
                        .try_handle_solidity(method, &input[4..])
                        .await
                        .unwrap_or_else(|| {
                            sails_rs::gstd::unknown_input_panic("Unknown request", input)
                        });
                    gstd::msg::reply_bytes(output, value).expect("Failed to send output");
                    return;
                }
            }
        }

        let (output, value): (Vec<u8>, ValueUnit) = if input.starts_with(&__ROUTE_SVC1) {
            let mut service = program_ref.svc1();
            let (output, value) = service
                .try_handle(&input[__ROUTE_SVC1.len()..])
                .await
                .unwrap_or_else(|| sails_rs::gstd::unknown_input_panic("Unknown request", input));
            ([__ROUTE_SVC1.as_ref(), &output].concat(), value)
        } else {
            sails_rs::gstd::unknown_input_panic("Unexpected service", input)
        };
        gstd::msg::reply_bytes(output, value).expect("Failed to send output");
    }
}

struct SomeService;
impl SomeService {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
        p1
    }
    pub fn this(&self, p1: bool) -> bool {
        p1
    }
}
pub struct SomeServiceExposure<T> {
    message_id: sails_rs::MessageId,
    route: &'static [u8],
    #[cfg(not(target_arch = "wasm32"))]
    inner: Box<T>,
    #[cfg(not(target_arch = "wasm32"))]
    inner_ptr: *const T,
    #[cfg(target_arch = "wasm32")]
    inner: T,
}
impl SomeServiceExposure<SomeService> {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new(self);
        self.inner.do_this(p1, p2).await
    }
    pub fn this(&self, p1: bool) -> bool {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new(self);
        self.inner.this(p1)
    }
    pub async fn handle(&mut self, input: &[u8]) -> (Vec<u8>, u128) {
        self.try_handle(input).await.unwrap_or_else(|| {
            let mut __input = input;
            let input: String = sails_rs::Decode::decode(&mut __input).unwrap_or_else(|_| {
                if input.len() <= 8 {
                    format!("0x{}", sails_rs::hex::encode(input))
                } else {
                    format!(
                        "0x{}..{}",
                        sails_rs::hex::encode(&input[..4]),
                        sails_rs::hex::encode(&input[input.len() - 4..])
                    )
                }
            });
            panic!("Unknown request: {}", input)
        })
    }
    pub async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)> {
        if input.starts_with(&[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8]) {
            let (output, value) = self.__do_this(&input[7usize..]).await;
            static INVOCATION_ROUTE: [u8; 7usize] = [24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8];
            return Some(([INVOCATION_ROUTE.as_ref(), &output].concat(), value));
        }
        if input.starts_with(&[16u8, 84u8, 104u8, 105u8, 115u8]) {
            let (output, value) = self.__this(&input[5usize..]).await;
            static INVOCATION_ROUTE: [u8; 5usize] = [16u8, 84u8, 104u8, 105u8, 115u8];
            return Some(([INVOCATION_ROUTE.as_ref(), &output].concat(), value));
        }
        // if input.starts_with(&some_service_meta::__DoThisParams::SELECTOR) {
        //     let mut input = &input[4usize..];
        //     let request: some_service_meta::__DoThisParams =
        //         sails_rs::Decode::decode(&mut input).expect("Failed to decode request");
        //     let result: u32 = self.do_this(request.p1, request.p2).await;
        //     let value = 0u128;
        //     return Some((sails_rs::Encode::encode(&result), value));
        // }
        None
    }

    pub async fn try_handle_solidity(
        &mut self,
        method: &[u8],
        input: &[u8],
    ) -> Option<(Vec<u8>, u128)> {
        if method == &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] {
            let (p1, p2): (u32, String) = <(u32, String)>::abi_decode_params(input, false).unwrap();
            let result: u32 = self.do_this(p1, p2).await;
            let value = 0u128;
            debug!("{}", result);
            return Some((<(u32,)>::abi_encode_params(&(result,)), value));
        }
        if method == &[16u8, 84u8, 104u8, 105u8, 115u8] {
            let (p1,): (bool,) = <(bool,)>::abi_decode_params(input, false).unwrap();
            let result = self.this(p1);
            let value = 0u128;
            return Some((<(bool,)>::abi_encode_params(&(result,)), value));
        }
        None
    }

    async fn __do_this(&mut self, mut input: &[u8]) -> (Vec<u8>, u128) {
        let request: some_service_meta::__DoThisParams =
            sails_rs::Decode::decode(&mut input).expect("Failed to decode request");
        let result: u32 = self.do_this(request.p1, request.p2).await;
        let value = 0u128;
        return (sails_rs::Encode::encode(&result), value);
    }
    async fn __this(&self, mut input: &[u8]) -> (Vec<u8>, u128) {
        let request: some_service_meta::__ThisParams =
            sails_rs::Decode::decode(&mut input).expect("Failed to decode request");
        let result = self.this(request.p1);
        let value = 0u128;
        return (sails_rs::Encode::encode(&result), value);
    }
}
impl sails_rs::gstd::services::Exposure for SomeServiceExposure<SomeService> {
    fn message_id(&self) -> sails_rs::MessageId {
        self.message_id
    }
    fn route(&self) -> &'static [u8] {
        self.route
    }
}
impl sails_rs::gstd::services::Service for SomeService {
    type Exposure = SomeServiceExposure<SomeService>;
    fn expose(self, message_id: sails_rs::MessageId, route: &'static [u8]) -> Self::Exposure {
        #[cfg(not(target_arch = "wasm32"))]
        let inner_box = Box::new(self);
        #[cfg(not(target_arch = "wasm32"))]
        let inner = inner_box.as_ref();
        #[cfg(target_arch = "wasm32")]
        let inner = &self;
        Self::Exposure {
            message_id,
            route,
            #[cfg(not(target_arch = "wasm32"))]
            inner_ptr: inner_box.as_ref() as *const Self,
            #[cfg(not(target_arch = "wasm32"))]
            inner: inner_box,
            #[cfg(target_arch = "wasm32")]
            inner: self,
        }
    }
}
impl sails_rs::meta::ServiceMeta for SomeService {
    fn commands() -> sails_rs::scale_info::MetaType {
        sails_rs::scale_info::MetaType::new::<some_service_meta::CommandsMeta>()
    }
    fn queries() -> sails_rs::scale_info::MetaType {
        sails_rs::scale_info::MetaType::new::<some_service_meta::QueriesMeta>()
    }
    fn events() -> sails_rs::scale_info::MetaType {
        sails_rs::scale_info::MetaType::new::<some_service_meta::EventsMeta>()
    }
    fn base_services() -> impl Iterator<Item = sails_rs::meta::AnyServiceMeta> {
        [].into_iter()
    }
}
mod some_service_meta {
    use super::*;
    use sails_rs::{Decode, TypeInfo};
    #[derive(Decode, TypeInfo)]
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __DoThisParams {
        pub(super) p1: u32,
        pub(super) p2: String,
    }
    #[derive(Decode, TypeInfo)]
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __ThisParams {
        pub(super) p1: bool,
    }
    #[derive(TypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum CommandsMeta {
        DoThis(__DoThisParams, u32),
    }
    #[derive(TypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum QueriesMeta {
        This(__ThisParams, bool),
    }
    #[derive(TypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum NoEvents {}
    pub type EventsMeta = NoEvents;

    // impl SolFunction for __DoThisParams {
    //     const SIGNATURE: &'static str = "do_this(uint32,string)";
    //     const SELECTOR: &[u8] = &[72, 187, 45, 101];
    // }
}

impl solidity::ServiceSignatures for SomeService {
    fn methods(route: &str) -> impl Iterator<Item = (String, &'static [u8])> {
        [
            (
                format!(
                    "{}_do_this({},{})",
                    route,
                    <<u32 as SolValue>::SolType as SolType>::SOL_NAME,
                    <<String as SolValue>::SolType as SolType>::SOL_NAME,
                ),
                &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            ),
            (
                format!(
                    "{}_this({})",
                    route,
                    <<bool as SolValue>::SolType as SolType>::SOL_NAME
                ),
                &[16u8, 84u8, 104u8, 105u8, 115u8] as &[u8],
            ),
        ]
        .into_iter()
    }
}
