#![no_std]
#![allow(unused_imports, dead_code, unused_variables)]
use sails_rs::gstd::services::*;
use sails_rs::prelude::*;

struct MyProgram;

const __ROUTE_SERVICE2: [u8; 9usize] = [32u8, 83u8, 101u8, 114u8, 118u8, 105u8, 99u8, 101u8, 50u8];
const __ROUTE_SVC1: [u8; 5usize] = [16u8, 83u8, 118u8, 99u8, 49u8];
impl MyProgram {
    #[route("svc1")]
    pub fn service1(&self) -> <SomeService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__service1();
        let exposure = <SomeService as sails_rs::gstd::services::Service>::expose(
            service,
            sails_rs::gstd::msg::id().into(),
            __ROUTE_SVC1.as_ref(),
        );
        exposure
    }
    pub fn service2(&self) -> <SomeService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__service2();
        let exposure = <SomeService as sails_rs::gstd::services::Service>::expose(
            service,
            sails_rs::gstd::msg::id().into(),
            __ROUTE_SERVICE2.as_ref(),
        );
        exposure
    }
    fn __service2(&self) -> SomeService {
        SomeService
    }
    fn __service1(&self) -> SomeService {
        SomeService
    }
    pub fn default() -> Self {
        Self
    }
}
impl sails_rs::meta::ProgramMeta for MyProgram {
    fn constructors() -> sails_rs::scale_info::MetaType {
        sails_rs::scale_info::MetaType::new::<meta_in_program::ConstructorsMeta>()
    }
    fn services() -> impl Iterator<Item = (&'static str, sails_rs::meta::AnyServiceMeta)> {
        [
            (
                "Service2",
                sails_rs::meta::AnyServiceMeta::new::<SomeService>(),
            ),
            ("Svc1", sails_rs::meta::AnyServiceMeta::new::<SomeService>()),
        ]
        .into_iter()
    }
}
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
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use sails_rs::{gstd, hex, prelude::*};
    static mut PROGRAM: Option<MyProgram> = None;
    #[gstd::async_init]
    async fn init() {
        sails_rs::gstd::events::__enable_events();
        let mut input: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
        let (program, invocation_route) =
            if input.starts_with(&[28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8]) {
                static INVOCATION_ROUTE: [u8; 8usize] =
                    [28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8];
                let request = __DefaultParams::decode(&mut &input[8usize..])
                    .expect("Failed to decode request");
                let program = MyProgram::default();
                (program, INVOCATION_ROUTE.as_ref())
            } else {
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
                panic!("Unexpected ctor: {}", input)
            };
        unsafe {
            PROGRAM = Some(program);
        }
        gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
    }
    #[gstd::async_main]
    async fn main() {
        let mut input: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
        let (output, value): (Vec<u8>, ValueUnit) = if input.starts_with(&__ROUTE_SERVICE2) {
            let program_ref = unsafe { PROGRAM.as_ref() }.expect("Program not initialized");
            let mut service = program_ref.service2();
            let (output, value) = service.handle(&input[__ROUTE_SERVICE2.len()..]).await;
            ([__ROUTE_SERVICE2.as_ref(), &output].concat(), value)
        } else if input.starts_with(&__ROUTE_SVC1) {
            let program_ref = unsafe { PROGRAM.as_ref() }.expect("Program not initialized");
            let mut service = program_ref.service1();
            let (output, value) = service.handle(&input[__ROUTE_SVC1.len()..]).await;
            ([__ROUTE_SVC1.as_ref(), &output].concat(), value)
        } else {
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
            panic!("Unexpected service: {}", input)
        };
        gstd::msg::reply_bytes(output, value).expect("Failed to send output");
    }
}

pub struct SomeService;
impl SomeService {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
        p1
    }
    pub fn this(&self, p1: bool) -> bool {
        p1
    }
}

trait __SomeServiceImplTrait {
    async fn do_this(&mut self, p1: u32, p2: String) -> u32;
    fn this(&self, p1: bool) -> bool;
}

// pub struct SomeServiceExposure<T> {
//     message_id: sails_rs::MessageId,
//     route: &'static [u8],
//     #[cfg(not(target_arch = "wasm32"))]
//     inner: Box<T>,
//     #[cfg(not(target_arch = "wasm32"))]
//     inner_ptr: *const T,
//     #[cfg(target_arch = "wasm32")]
//     inner: T,
// }
// impl ServiceExposure<SomeService> {
//     pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
//         self.call_scoped(|s| s.do_this(p1, p2)).await
//         // let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new(self);
//         // self.inner.do_this(p1, p2).await
//     }
//     pub fn this(&self, p1: bool) -> bool {
//         let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new(self);
//         self.inner.this(p1)
//     }
//     pub async fn handle(&mut self, input: &[u8]) -> (Vec<u8>, u128) {
//         self.try_handle(input).await.unwrap_or_else(|| {
//             let mut __input = input;
//             let input: String = sails_rs::Decode::decode(&mut __input).unwrap_or_else(|_| {
//                 if input.len() <= 8 {
//                     format!("0x{}", sails_rs::hex::encode(input))
//                 } else {
//                     format!(
//                         "0x{}..{}",
//                         sails_rs::hex::encode(&input[..4]),
//                         sails_rs::hex::encode(&input[input.len() - 4..])
//                     )
//                 }
//             });
//             panic!("Unknown request: {}", input)
//         })
//     }
//     pub async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)> {
//         if input.starts_with(&[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8]) {
//             let (output, value) = self.__do_this(&input[7usize..]).await;
//             static INVOCATION_ROUTE: [u8; 7usize] = [24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8];
//             return Some(([INVOCATION_ROUTE.as_ref(), &output].concat(), value));
//         }
//         if input.starts_with(&[16u8, 84u8, 104u8, 105u8, 115u8]) {
//             let (output, value) = self.__this(&input[5usize..]).await;
//             static INVOCATION_ROUTE: [u8; 5usize] = [16u8, 84u8, 104u8, 105u8, 115u8];
//             return Some(([INVOCATION_ROUTE.as_ref(), &output].concat(), value));
//         }
//         None
//     }
//     async fn __do_this(&mut self, mut input: &[u8]) -> (Vec<u8>, u128) {
//         let request: some_service_meta::__DoThisParams =
//             sails_rs::Decode::decode(&mut input).expect("Failed to decode request");
//         let result = self.do_this(request.p1, request.p2).await;
//         let value = 0u128;
//         return (sails_rs::Encode::encode(&result), value);
//     }
//     async fn __this(&self, mut input: &[u8]) -> (Vec<u8>, u128) {
//         let request: some_service_meta::__ThisParams =
//             sails_rs::Decode::decode(&mut input).expect("Failed to decode request");
//         let result = self.this(request.p1);
//         let value = 0u128;
//         return (sails_rs::Encode::encode(&result), value);
//     }
// }
// impl sails_rs::gstd::services::Exposure for SomeServiceExposure<SomeService> {
//     type Service = SomeService;

//     fn message_id(&self) -> sails_rs::MessageId {
//         self.message_id
//     }
//     fn route(&self) -> &'static [u8] {
//         self.route
//     }
// }

#[allow(unused_parens)]
impl __SomeServiceImplTrait
    for ServiceExposure<SomeService, (ServiceExposure<ExtendedService1, ()>)>
{
    async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new2(self);
        self.inner.do_this(p1, p2).await
    }
    fn this(&self, p1: bool) -> bool {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new2(self);
        self.inner.this(p1)
    }
}

#[allow(unused_parens)]
impl sails_rs::gstd::services::Service for SomeService {
    type Exposure = ServiceExposure<SomeService, (ServiceExposure<ExtendedService1, ()>)>;
    type Extend = (ServiceExposure<ExtendedService1, ()>);

    fn expose(self, message_id: sails_rs::MessageId, route: &'static [u8]) -> Self::Exposure {
        let extend = <ExtendedService1 as Clone>::clone(AsRef::<ExtendedService1>::as_ref(&self))
            .expose(message_id, route);
        Self::Exposure::new(message_id, route, self, extend)
    }
}

impl sails_rs::gstd::services::ServiceHandle for SomeService {
    async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)> {
        let mut __input = input;
        let route: String = sails_rs::Decode::decode(&mut __input).ok()?;
        match route.as_str() {
            "DoThis" => {
                let request: some_service_meta::__DoThisParams =
                    sails_rs::Decode::decode(&mut __input).expect("Failed to decode request");
                let result = self.do_this(request.p1, request.p2).await;
                let value = 0u128;
                Some((sails_rs::Encode::encode(&("DoThis", &result)), value))
            }
            "This" => {
                let request: some_service_meta::__ThisParams =
                    sails_rs::Decode::decode(&mut __input).expect("Failed to decode request");
                let result = self.this(request.p1);
                let value = 0u128;
                Some((sails_rs::Encode::encode(&("This", &result)), value))
            }
            _ => None,
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
}

#[derive(Debug, Clone)]
pub struct ExtendedService1;
impl ExtendedService1 {
    pub async fn none() {}
}
impl Service for ExtendedService1 {
    type Exposure = ServiceExposure<ExtendedService1, ()>;
    type Extend = ();

    fn expose(self, message_id: MessageId, route: &'static [u8]) -> Self::Exposure {
        Self::Exposure::new(message_id, route, self, ())
    }
}

impl AsRef<ExtendedService1> for SomeService {
    fn as_ref(&self) -> &ExtendedService1 {
        todo!()
    }
}

pub struct ReferenceService<'a> {
    data: Option<ReferenceData<'a>>,
}
struct ReferenceData<'a> {
    num: &'a mut u8,
    message: &'a str,
}

impl<'t> ReferenceService<'t> {
    pub async fn guess_num(&mut self, number: u8) -> Result<&'t str, &'static str> {
        if number > 42 {
            Err("Number is too large")
        } else if let Some(data) = &self.data.as_ref() {
            if *data.num == number {
                Ok(data.message)
            } else {
                Err("Try again")
            }
        } else {
            Err("Data is not set")
        }
    }
    pub async fn message(&self) -> Option<&'t str> {
        self.data.as_ref().map(|d| d.message)
    }
}
trait __ReferenceServiceImplTrait<'t> {
    async fn guess_num(&mut self, number: u8) -> Result<&'t str, &'static str>;
    async fn message(&self) -> Option<&'t str>;
}
impl<'t> __ReferenceServiceImplTrait<'t>
    for sails_rs::gstd::services::ServiceExposure<ReferenceService<'t>, ()>
{
    async fn guess_num(&mut self, number: u8) -> Result<&'t str, &'static str> {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new2(self);
        self.inner.guess_num(number).await
    }
    async fn message(&self) -> Option<&'t str> {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new2(self);
        self.inner.message().await
    }
}
impl<'t> sails_rs::gstd::services::ServiceHandle for ReferenceService<'t> {
    async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)> {
        let mut __input = input;
        let route: String = sails_rs::Decode::decode(&mut __input).ok()?;
        match route.as_str() {
            "GuessNum" => {
                let request: reference_service_meta::__GuessNumParams =
                    sails_rs::Decode::decode(&mut __input).expect("Failed to decode request");
                let result = self.guess_num(request.number).await;
                let value = 0u128;
                Some((sails_rs::Encode::encode(&("GuessNum", &result)), value))
            }
            "Message" => {
                let request: reference_service_meta::__MessageParams =
                    sails_rs::Decode::decode(&mut __input).expect("Failed to decode request");
                let result = self.message().await;
                let value = 0u128;
                Some((sails_rs::Encode::encode(&("Message", &result)), value))
            }
            _ => None,
        }
    }
}
impl<'t> sails_rs::gstd::services::Service for ReferenceService<'t> {
    type Exposure = sails_rs::gstd::services::ServiceExposure<ReferenceService<'t>, ()>;
    type Extend = ();
    fn expose(self, message_id: sails_rs::MessageId, route: &'static [u8]) -> Self::Exposure {
        let extend = ();
        Self::Exposure::new(message_id, route, self, extend)
    }
}
impl<'t> sails_rs::meta::ServiceMeta for ReferenceService<'t> {
    fn commands() -> sails_rs::scale_info::MetaType {
        sails_rs::scale_info::MetaType::new::<reference_service_meta::CommandsMeta>()
    }
    fn queries() -> sails_rs::scale_info::MetaType {
        sails_rs::scale_info::MetaType::new::<reference_service_meta::QueriesMeta>()
    }
    fn events() -> sails_rs::scale_info::MetaType {
        sails_rs::scale_info::MetaType::new::<reference_service_meta::EventsMeta>()
    }
    fn base_services() -> impl Iterator<Item = sails_rs::meta::AnyServiceMeta> {
        [].into_iter()
    }
}
mod reference_service_meta {
    use super::*;
    use sails_rs::{Decode, TypeInfo};
    #[derive(Decode, TypeInfo)]
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __GuessNumParams {
        pub(super) number: u8,
    }
    #[derive(Decode, TypeInfo)]
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __MessageParams {}
    #[derive(TypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum CommandsMeta {
        GuessNum(__GuessNumParams, Result<&'static str, &'static str>),
    }
    #[derive(TypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum QueriesMeta {
        Message(__MessageParams, Option<&'static str>),
    }
    #[derive(TypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum NoEvents {}
    pub type EventsMeta = NoEvents;
}
