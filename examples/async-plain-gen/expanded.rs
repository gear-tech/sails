#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use core::prelude::rust_2024::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
pub struct AsyncMethodsService;
impl AsyncMethodsService {
    pub async fn async_method(&self) -> u32 {
        42
    }
}
pub struct AsyncMethodsServiceExposure<T: sails_rs::gstd::services::Service> {
    message_id: sails_rs::MessageId,
    route: &'static [u8],
    #[cfg(target_arch = "wasm32")]
    inner: T,
    base: T::BaseExposures,
}
impl<T: sails_rs::gstd::services::Service> sails_rs::gstd::services::Exposure
for AsyncMethodsServiceExposure<T> {
    fn message_id(&self) -> sails_rs::MessageId {
        self.message_id
    }
    fn route(&self) -> &'static [u8] {
        self.route
    }
}
impl AsyncMethodsServiceExposure<AsyncMethodsService> {
    pub async fn async_method(&self) -> u32 {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new(self);
        self.inner.async_method().await
    }
    pub async fn try_handle(
        &mut self,
        input: &[u8],
        result_handler: fn(&[u8], u128),
    ) -> Option<()> {
        use sails_rs::gstd::InvocationIo;
        use sails_rs::gstd::services::Exposure;
        if let Ok(request) = async_methods_service_meta::__AsyncMethodParams::decode_params(
            input,
        ) {
            let result = self.async_method().await;
            let value = 0u128;
            async_methods_service_meta::__AsyncMethodParams::with_optimized_encode(
                &result,
                self.route().as_ref(),
                |encoded_result| result_handler(encoded_result, value),
            );
            return Some(());
        }
        None
    }
}
impl sails_rs::gstd::services::Service for AsyncMethodsService {
    type Exposure = AsyncMethodsServiceExposure<Self>;
    type BaseExposures = ();
    fn expose(
        self,
        message_id: sails_rs::MessageId,
        route: &'static [u8],
    ) -> Self::Exposure {
        #[cfg(target_arch = "wasm32")]
        let inner = &self;
        Self::Exposure {
            message_id,
            route,
            base: (),
            #[cfg(target_arch = "wasm32")]
            inner: self,
        }
    }
}
impl sails_rs::meta::ServiceMeta for AsyncMethodsService {
    type CommandsMeta = async_methods_service_meta::CommandsMeta;
    type QueriesMeta = async_methods_service_meta::QueriesMeta;
    type EventsMeta = async_methods_service_meta::EventsMeta;
    const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
}
mod async_methods_service_meta {
    use super::*;
    use sails_rs::{Decode, TypeInfo};
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __AsyncMethodParams {}
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for __AsyncMethodParams {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(__AsyncMethodParams {})
            }
        }
    };
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for __AsyncMethodParams {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "__AsyncMethodParams",
                            "async_plain_gen::async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(sails_rs::scale_info::build::Fields::named())
            }
        }
    };
    impl sails_rs::gstd::InvocationIo for __AsyncMethodParams {
        const ROUTE: &'static [u8] = &[
            44u8, 65u8, 115u8, 121u8, 110u8, 99u8, 77u8, 101u8, 116u8, 104u8, 111u8,
            100u8,
        ];
        type Params = Self;
    }
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum CommandsMeta {}
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for CommandsMeta {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "CommandsMeta",
                            "async_plain_gen::async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(sails_rs::scale_info::build::Variants::new())
            }
        }
    };
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum QueriesMeta {
        AsyncMethod(__AsyncMethodParams, u32),
    }
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for QueriesMeta {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "QueriesMeta",
                            "async_plain_gen::async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant(
                                "AsyncMethod",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f
                                                        .ty::<__AsyncMethodParams>()
                                                        .type_name("__AsyncMethodParams")
                                                })
                                                .field(|f| f.ty::<u32>().type_name("u32")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum NoEvents {}
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for NoEvents {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "NoEvents",
                            "async_plain_gen::async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(sails_rs::scale_info::build::Variants::new())
            }
        }
    };
    pub type EventsMeta = NoEvents;
}
pub struct NoAsyncMethodsService;
impl NoAsyncMethodsService {
    pub fn sync_method(&self) -> u32 {
        24
    }
}
pub struct NoAsyncMethodsServiceExposure<T: sails_rs::gstd::services::Service> {
    message_id: sails_rs::MessageId,
    route: &'static [u8],
    #[cfg(target_arch = "wasm32")]
    inner: T,
    base: T::BaseExposures,
}
impl<T: sails_rs::gstd::services::Service> sails_rs::gstd::services::Exposure
for NoAsyncMethodsServiceExposure<T> {
    fn message_id(&self) -> sails_rs::MessageId {
        self.message_id
    }
    fn route(&self) -> &'static [u8] {
        self.route
    }
}
impl NoAsyncMethodsServiceExposure<NoAsyncMethodsService> {
    pub fn sync_method(&self) -> u32 {
        let exposure_scope = sails_rs::gstd::services::ExposureCallScope::new(self);
        self.inner.sync_method()
    }
    pub async fn try_handle(
        &mut self,
        input: &[u8],
        result_handler: fn(&[u8], u128),
    ) -> Option<()> {
        use sails_rs::gstd::InvocationIo;
        use sails_rs::gstd::services::Exposure;
        if let Ok(request) = no_async_methods_service_meta::__SyncMethodParams::decode_params(
            input,
        ) {
            let result = self.sync_method();
            let value = 0u128;
            no_async_methods_service_meta::__SyncMethodParams::with_optimized_encode(
                &result,
                self.route().as_ref(),
                |encoded_result| result_handler(encoded_result, value),
            );
            return Some(());
        }
        None
    }
}
impl sails_rs::gstd::services::Service for NoAsyncMethodsService {
    type Exposure = NoAsyncMethodsServiceExposure<Self>;
    type BaseExposures = ();
    fn expose(
        self,
        message_id: sails_rs::MessageId,
        route: &'static [u8],
    ) -> Self::Exposure {
        #[cfg(target_arch = "wasm32")]
        let inner = &self;
        Self::Exposure {
            message_id,
            route,
            base: (),
            #[cfg(target_arch = "wasm32")]
            inner: self,
        }
    }
}
impl sails_rs::meta::ServiceMeta for NoAsyncMethodsService {
    type CommandsMeta = no_async_methods_service_meta::CommandsMeta;
    type QueriesMeta = no_async_methods_service_meta::QueriesMeta;
    type EventsMeta = no_async_methods_service_meta::EventsMeta;
    const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
}
mod no_async_methods_service_meta {
    use super::*;
    use sails_rs::{Decode, TypeInfo};
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __SyncMethodParams {}
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for __SyncMethodParams {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(__SyncMethodParams {})
            }
        }
    };
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for __SyncMethodParams {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "__SyncMethodParams",
                            "async_plain_gen::no_async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(sails_rs::scale_info::build::Fields::named())
            }
        }
    };
    impl sails_rs::gstd::InvocationIo for __SyncMethodParams {
        const ROUTE: &'static [u8] = &[
            40u8, 83u8, 121u8, 110u8, 99u8, 77u8, 101u8, 116u8, 104u8, 111u8, 100u8,
        ];
        type Params = Self;
    }
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum CommandsMeta {}
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for CommandsMeta {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "CommandsMeta",
                            "async_plain_gen::no_async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(sails_rs::scale_info::build::Variants::new())
            }
        }
    };
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum QueriesMeta {
        SyncMethod(__SyncMethodParams, u32),
    }
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for QueriesMeta {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "QueriesMeta",
                            "async_plain_gen::no_async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant(
                                "SyncMethod",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f.ty::<__SyncMethodParams>().type_name("__SyncMethodParams")
                                                })
                                                .field(|f| f.ty::<u32>().type_name("u32")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum NoEvents {}
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for NoEvents {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "NoEvents",
                            "async_plain_gen::no_async_methods_service_meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(sails_rs::scale_info::build::Variants::new())
            }
        }
    };
    pub type EventsMeta = NoEvents;
}
pub struct MyProgram;
impl MyProgram {
    pub fn new() -> Self {
        MyProgram
    }
    fn __async_service(&self) -> AsyncMethodsService {
        AsyncMethodsService
    }
    fn __no_async_service(&self) -> NoAsyncMethodsService {
        NoAsyncMethodsService
    }
    pub fn async_service(
        &self,
    ) -> <AsyncMethodsService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__async_service();
        let exposure = <AsyncMethodsService as sails_rs::gstd::services::Service>::expose(
            service,
            sails_rs::gstd::Syscall::message_id(),
            __ROUTE_ASYNCSERVICE.as_ref(),
        );
        exposure
    }
    pub fn no_async_service(
        &self,
    ) -> <NoAsyncMethodsService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__no_async_service();
        let exposure = <NoAsyncMethodsService as sails_rs::gstd::services::Service>::expose(
            service,
            sails_rs::gstd::Syscall::message_id(),
            __ROUTE_NOASYNCSERVICE.as_ref(),
        );
        exposure
    }
}
const __ROUTE_ASYNCSERVICE: [u8; 13usize] = [
    48u8, 65u8, 115u8, 121u8, 110u8, 99u8, 83u8, 101u8, 114u8, 118u8, 105u8, 99u8, 101u8,
];
const __ROUTE_NOASYNCSERVICE: [u8; 15usize] = [
    56u8, 78u8, 111u8, 65u8, 115u8, 121u8, 110u8, 99u8, 83u8, 101u8, 114u8, 118u8, 105u8,
    99u8, 101u8,
];
impl sails_rs::meta::ProgramMeta for MyProgram {
    type ConstructorsMeta = meta_in_program::ConstructorsMeta;
    const SERVICES: &'static [(&'static str, sails_rs::meta::AnyServiceMetaFn)] = &[
        ("AsyncService", sails_rs::meta::AnyServiceMeta::new::<AsyncMethodsService>),
        ("NoAsyncService", sails_rs::meta::AnyServiceMeta::new::<NoAsyncMethodsService>),
    ];
}
mod meta_in_program {
    use super::*;
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __NewParams {}
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for __NewParams {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(__NewParams {})
            }
        }
    };
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for __NewParams {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "__NewParams",
                            "async_plain_gen::meta_in_program",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(sails_rs::scale_info::build::Fields::named())
            }
        }
    };
    impl sails_rs::gstd::InvocationIo for __NewParams {
        const ROUTE: &'static [u8] = &[12u8, 78u8, 101u8, 119u8];
        type Params = Self;
    }
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum ConstructorsMeta {
        New(__NewParams),
    }
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for ConstructorsMeta {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "ConstructorsMeta",
                            "async_plain_gen::meta_in_program",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant(
                                "New",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<__NewParams>().type_name("__NewParams")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
}
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use sails_rs::{gstd, hex, prelude::*};
    static mut PROGRAM: Option<MyProgram> = None;
    #[unsafe(no_mangle)]
    extern "C" fn init() {
        gstd::message_loop(async {
            use gstd::InvocationIo;
            let mut input: &[u8] = &gstd::msg::load_bytes()
                .expect("Failed to read input");
            let (program, invocation_route) = if let Ok(request) = meta_in_program::__NewParams::decode_params(
                input,
            ) {
                let program = MyProgram::new();
                (program, meta_in_program::__NewParams::ROUTE)
            } else {
                gstd::unknown_input_panic("Unexpected ctor", input)
            };
            unsafe {
                PROGRAM = Some(program);
            }
            gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
        });
    }
    fn __main_safe() {
        gstd::message_loop(async {
            let mut input: &[u8] = &gstd::msg::load_bytes()
                .expect("Failed to read input");
            let program_ref = unsafe { PROGRAM.as_mut() }
                .expect("Program not initialized");
            if input.starts_with(&__ROUTE_ASYNCSERVICE) {
                let mut service = program_ref.async_service();
                service
                    .try_handle(
                        &input[__ROUTE_ASYNCSERVICE.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .await
                    .unwrap_or_else(|| {
                        gstd::unknown_input_panic("Unknown request", input)
                    });
            } else if input.starts_with(&__ROUTE_NOASYNCSERVICE) {
                let mut service = program_ref.no_async_service();
                service
                    .try_handle(
                        &input[__ROUTE_NOASYNCSERVICE.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .await
                    .unwrap_or_else(|| {
                        gstd::unknown_input_panic("Unknown request", input)
                    });
            } else {
                gstd::unknown_input_panic("Unexpected service", input)
            };
        });
    }
    #[unsafe(no_mangle)]
    extern "C" fn handle() {
        __main_safe();
    }
    #[unsafe(no_mangle)]
    extern "C" fn handle_reply() {
        gstd::handle_reply_with_hook();
        ();
    }
    #[unsafe(no_mangle)]
    extern "C" fn handle_signal() {
        gstd::handle_signal();
        ();
    }
}
