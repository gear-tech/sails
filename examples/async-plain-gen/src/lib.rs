#![no_std]

use sails_rs::{gstd, prelude::*};
use sails_rs::{
    gstd::{
        Syscall,
        services::Service,
    },
    meta::{AnyServiceMeta, AnyServiceMetaFn, ProgramMeta},
};
use services::{
    AsyncMethodsServiceExposure, AsyncService, NoAsyncMethodsService, NoAsyncMethodsServiceExposure,
};

pub mod services;

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
        const ASYNCNESS: bool = false;
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
        // let input = &input1[..];
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
                    let input_clone = input1.clone();
                    let input_clone_ref = &input_clone[..];
                    service
                        .try_handle_async(
                            &input_clone_ref[__ROUTE_ASYNCSERVICE.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| gstd::unknown_input_panic("Unknown request", input_clone_ref));
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
