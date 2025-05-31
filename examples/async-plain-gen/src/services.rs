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
            return Some(no_async_methods_service_meta::__SomeMethodParams::ASYNCNESS);
        }

        None
    }

    pub fn try_handle(&mut self, input: &[u8], result_handler: fn(&[u8], u128)) -> Option<()> {
        use sails_rs::gstd::InvocationIo;
        use sails_rs::gstd::services::Exposure;
        if let Ok(_request) = no_async_methods_service_meta::__SomeMethodParams::decode_params(input)
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
        const ASYNCNESS: bool = false;
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
            return Some(async_methods_service_meta::__SomeAsyncMethodParams::ASYNCNESS);
        }
        None
    }

    pub fn try_handle(&mut self, _input: &[u8], _result_handler: fn(&[u8], u128)) -> Option<()> {
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
            60u8, 83u8, 111u8, 109u8, 101u8, 65u8, 115u8, 121u8, 110u8, 99u8, 77u8, 101u8, 116u8,
            104u8, 111u8, 100u8,
        ];
        type Params = Self;
        const ASYNCNESS: bool = true;
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
