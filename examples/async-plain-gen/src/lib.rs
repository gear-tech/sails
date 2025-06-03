#![no_std]

use sails_rs::{gstd, prelude::*};
use sails_rs::{
    gstd::{Syscall, services::Service},
    meta::{AnyServiceMeta, AnyServiceMetaFn, ProgramMeta},
};
use services::{
    AsyncMethodsServiceExposure, AsyncService, NoAsyncMethodsService, NoAsyncMethodsServiceExposure,
};

pub mod services {
    use sails_rs::prelude::{String, format};
    use sails_rs::{
        MessageId,
        gstd::{
            InvocationIo,
            services::{Exposure, ExposureCallScope, Service},
        },
        meta::{AnyServiceMetaFn, ServiceMeta},
    };

    fn sum_fibonacci(n: u32) -> u32 {
        fn fibonacci(n: u32) -> u32 {
            if n <= 1 {
                return n as u32;
            }
            fibonacci(n - 1) + fibonacci(n - 2)
        }

        (0..n).fold(0, |sum, i| sum + fibonacci(i))
    }

    pub struct NoAsyncMethodsService;

    impl NoAsyncMethodsService {
        pub fn some_method(&self) -> &'static str {
            sum_fibonacci(10);
            "Non async some method"
        }
    }

    pub struct NoAsyncMethodsServiceExposure<T: Service> {
        message_id: MessageId,
        route: &'static [u8],
        inner: T,
        base: T::BaseExposures,
    }

    impl<T: Service> Exposure for NoAsyncMethodsServiceExposure<T> {
        fn message_id(&self) -> MessageId {
            self.message_id
        }
        fn route(&self) -> &'static [u8] {
            self.route
        }
    }

    impl NoAsyncMethodsServiceExposure<NoAsyncMethodsService> {
        pub fn some_method(&self) -> &'static str {
            let _exposure_scope = ExposureCallScope::new(self);
            self.inner.some_method()
        }

        pub fn check_asyncness(&self, payload: impl AsRef<[u8]>) -> Option<bool> {
            let payload = payload.as_ref();
            if no_async_methods_service_meta::__SomeMethodParams::check_route(payload).is_ok() {
                return Some(no_async_methods_service_meta::__SomeMethodParams::ASYNC);
            }

            None
        }

        pub fn try_handle(&mut self, input: &[u8], result_handler: fn(&[u8], u128)) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::Exposure;
            if let Ok(_request) =
                no_async_methods_service_meta::__SomeMethodParams::decode_params(input)
            {
                let result = self.some_method();
                let value = 0u128;
                no_async_methods_service_meta::__SomeMethodParams::with_optimized_encode(
                    &result,
                    self.route().as_ref(),
                    |encoded_result| result_handler(encoded_result, value),
                );
                return Some(());
            }
            None
        }

        pub async fn try_handle_async(
            &mut self,
            _input: &[u8],
            _result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            None
        }
    }

    impl Service for NoAsyncMethodsService {
        type Exposure = NoAsyncMethodsServiceExposure<Self>;
        type BaseExposures = ();
        fn expose(self, message_id: MessageId, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                message_id,
                route,
                base: (),
                inner: self,
            }
        }
    }

    impl ServiceMeta for NoAsyncMethodsService {
        type CommandsMeta = no_async_methods_service_meta::CommandsMeta;
        type QueriesMeta = no_async_methods_service_meta::QueriesMeta;
        type EventsMeta = no_async_methods_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [AnyServiceMetaFn] = &[];
    }
    mod no_async_methods_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo, prelude::String};

        #[derive(Decode, TypeInfo)]
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __SomeMethodParams {}

        impl InvocationIo for __SomeMethodParams {
            const ROUTE: &'static [u8] = &[
                40u8, 83u8, 111u8, 109u8, 101u8, 77u8, 101u8, 116u8, 104u8, 111u8, 100u8,
            ];
            type Params = Self;
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {}

        #[derive(TypeInfo)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {
            SomeMethod(__SomeMethodParams, &'static str),
        }
        #[allow(
            non_upper_case_globals,
            deprecated,
            unused_attributes,
            unused_qualifications
        )]
        #[derive(TypeInfo)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum NoEvents {}
        #[allow(
            non_upper_case_globals,
            deprecated,
            unused_attributes,
            unused_qualifications
        )]

        pub type EventsMeta = NoEvents;
    }

    pub struct AsyncService;

    impl AsyncService {
        pub async fn some_async_method(&mut self) -> &'static str {
            sum_fibonacci(10);
            "Async some method"
        }
    }

    pub struct AsyncMethodsServiceExposure<T: Service> {
        message_id: MessageId,
        route: &'static [u8],
        inner: T,
        base: T::BaseExposures,
    }

    impl<T: Service> Exposure for AsyncMethodsServiceExposure<T> {
        fn message_id(&self) -> sails_rs::MessageId {
            self.message_id
        }
        fn route(&self) -> &'static [u8] {
            self.route
        }
    }

    impl AsyncMethodsServiceExposure<AsyncService> {
        pub async fn some_async_method(&mut self) -> &'static str {
            let _exposure_scope = ExposureCallScope::new(self);
            self.inner.some_async_method().await
        }

        pub fn check_asyncness(&self, payload: impl AsRef<[u8]>) -> Option<bool> {
            let payload = payload.as_ref();
            if async_methods_service_meta::__SomeAsyncMethodParams::check_route(payload).is_ok() {
                return Some(async_methods_service_meta::__SomeAsyncMethodParams::ASYNC);
            }
            None
        }

        pub fn try_handle(
            &mut self,
            _input: &[u8],
            _result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            None
        }

        pub async fn try_handle_async(
            &mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            if let Ok(_request) =
                async_methods_service_meta::__SomeAsyncMethodParams::decode_params(input)
            {
                let result = self.some_async_method().await;
                let value = 0u128;
                async_methods_service_meta::__SomeAsyncMethodParams::with_optimized_encode(
                    &result,
                    self.route().as_ref(),
                    |encoded_result| result_handler(encoded_result, value),
                );
                return Some(());
            }
            None
        }
    }

    impl Service for AsyncService {
        type Exposure = AsyncMethodsServiceExposure<Self>;
        type BaseExposures = ();
        fn expose(self, message_id: MessageId, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                message_id,
                route,
                base: (),
                inner: self,
            }
        }
    }

    impl ServiceMeta for AsyncService {
        type CommandsMeta = async_methods_service_meta::CommandsMeta;
        type QueriesMeta = async_methods_service_meta::QueriesMeta;
        type EventsMeta = async_methods_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
    }

    mod async_methods_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo, prelude::String};

        #[derive(Decode, TypeInfo)]
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __SomeAsyncMethodParams {}

        impl InvocationIo for __SomeAsyncMethodParams {
            const ROUTE: &'static [u8] = &[
                60u8, 83u8, 111u8, 109u8, 101u8, 65u8, 115u8, 121u8, 110u8, 99u8, 77u8, 101u8,
                116u8, 104u8, 111u8, 100u8,
            ];
            type Params = Self;
            const ASYNC: bool = true;
        }

        #[derive(TypeInfo)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            SomeAsyncMethod(__SomeAsyncMethodParams, &'static str),
        }

        #[derive(TypeInfo)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {}
        #[allow(
            non_upper_case_globals,
            deprecated,
            unused_attributes,
            unused_qualifications
        )]
        #[derive(TypeInfo)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum NoEvents {}
        pub type EventsMeta = NoEvents;
    }
}

const __ROUTE_ASYNCSERVICE: [u8; 13] = [48, 65, 115, 121, 110, 99, 83, 101, 114, 118, 105, 99, 101];
const __ROUTE_NOASYNCSERVICE: [u8; 15] = [
    56, 78, 111, 65, 115, 121, 110, 99, 83, 101, 114, 118, 105, 99, 101,
];

pub struct MyProgram;

impl MyProgram {
    pub fn new() -> Self {
        MyProgram
    }

    pub fn async_service(&self) -> AsyncMethodsServiceExposure<AsyncService> {
        let service = AsyncService;
        let exposure = AsyncService::expose(
            service,
            Syscall::message_id(),
            __ROUTE_ASYNCSERVICE.as_ref(),
        );

        exposure
    }

    pub fn no_async_service(&self) -> NoAsyncMethodsServiceExposure<NoAsyncMethodsService> {
        let service = NoAsyncMethodsService;
        let exposure = NoAsyncMethodsService::expose(
            service,
            Syscall::message_id(),
            __ROUTE_NOASYNCSERVICE.as_ref(),
        );

        exposure
    }
}

impl ProgramMeta for MyProgram {
    type ConstructorsMeta = meta_in_program::ConstructorsMeta;
    const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
        ("AsyncService", AnyServiceMeta::new::<AsyncService>),
        (
            "NoAsyncService",
            AnyServiceMeta::new::<NoAsyncMethodsService>,
        ),
    ];
}

mod meta_in_program {
    use super::*;
    use sails_rs::{Decode, TypeInfo};

    #[derive(Decode, TypeInfo)]
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __NewParams {}

    impl sails_rs::gstd::InvocationIo for __NewParams {
        const ROUTE: &'static [u8] = &[12u8, 78u8, 101u8, 119u8];
        type Params = Self;
        const ASYNC: bool = false;
    }

    #[derive(TypeInfo)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum ConstructorsMeta {
        New(__NewParams),
    }
}

pub mod wasm {
    use super::*;

    static mut PROGRAM: Option<MyProgram> = None;

    // #[gstd::async_init]
    #[unsafe(no_mangle)]
    extern "C" fn init() {
        use gstd::InvocationIo;
        use sails_rs::gstd;
        let input: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
        let (program, invocation_route) =
            if let Ok(_request) = meta_in_program::__NewParams::decode_params(input) {
                let program = MyProgram::new();
                (program, meta_in_program::__NewParams::ROUTE)
            } else {
                gstd::unknown_input_panic("Unexpected ctor", input)
            };
        unsafe {
            PROGRAM = Some(program);
        }
        gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
    }

    #[unsafe(no_mangle)]
    extern "C" fn handle() {
        let input1: Vec<u8> = gstd::msg::load_bytes().expect("Failed to read input");
        #[allow(static_mut_refs)]
        let program_ref = unsafe { PROGRAM.as_mut() }.expect("Program not initialized");

        if input1.starts_with(&__ROUTE_ASYNCSERVICE) {
            let mut service = program_ref.async_service();
            let Some(is_async) = service.check_asyncness(&input1[__ROUTE_ASYNCSERVICE.len()..])
            else {
                gstd::unknown_input_panic("Unknown call", &input1[__ROUTE_ASYNCSERVICE.len()..])
            };

            if is_async {
                gstd::message_loop(async move {
                    let input1 = input1.clone();
                    service
                        .try_handle_async(
                            &input1[__ROUTE_ASYNCSERVICE.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| gstd::unknown_input_panic("Unknown request", &input1));
                });
            } else {
                service
                    .try_handle(
                        &input1[__ROUTE_ASYNCSERVICE.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .unwrap_or_else(|| gstd::unknown_input_panic("Unknown request", &input1));
            }
        } else if input1.starts_with(&__ROUTE_NOASYNCSERVICE) {
            let mut service = program_ref.no_async_service();
            service
                .try_handle(
                    &input1[__ROUTE_NOASYNCSERVICE.len()..],
                    |encoded_result, value| {
                        gstd::msg::reply_bytes(encoded_result, value)
                            .expect("Failed to send output");
                    },
                )
                .unwrap_or_else(|| gstd::unknown_input_panic("Unknown request", &input1));
        } else {
            gstd::unknown_input_panic("Unexpected service", &input1)
        };
    }
}
