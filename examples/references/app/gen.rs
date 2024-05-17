#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
use core::ptr::addr_of;
use gstd::prelude::*;
use sails_rtl::gstd::gservice;
static mut COUNTER: Counter = Counter { count: 0 };
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct Counter {
    count: u32,
}
#[automatically_derived]
impl ::core::fmt::Debug for Counter {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "Counter",
            "count",
            &&self.count,
        )
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl sails_rtl::scale_codec::Encode for Counter {
        fn size_hint(&self) -> usize {
            sails_rtl::scale_codec::Encode::size_hint(&&self.count)
        }
        fn encode_to<
            __CodecOutputEdqy: sails_rtl::scale_codec::Output + ?::core::marker::Sized,
        >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
            sails_rtl::scale_codec::Encode::encode_to(&&self.count, __codec_dest_edqy)
        }
        fn encode(
            &self,
        ) -> sails_rtl::scale_codec::alloc::vec::Vec<::core::primitive::u8> {
            sails_rtl::scale_codec::Encode::encode(&&self.count)
        }
        fn using_encoded<
            __CodecOutputReturn,
            __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                    &[::core::primitive::u8],
                ) -> __CodecOutputReturn,
        >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
            sails_rtl::scale_codec::Encode::using_encoded(&&self.count, f)
        }
    }
    #[automatically_derived]
    impl sails_rtl::scale_codec::EncodeLike for Counter {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl sails_rtl::scale_codec::Decode for Counter {
        fn decode<__CodecInputEdqy: sails_rtl::scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, sails_rtl::scale_codec::Error> {
            ::core::result::Result::Ok(Counter {
                count: {
                    let __codec_res_edqy = <u32 as sails_rtl::scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Counter::count`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
            })
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl sails_rtl::scale_info::TypeInfo for Counter {
        type Identity = Self;
        fn type_info() -> sails_rtl::scale_info::Type {
            sails_rtl::scale_info::Type::builder()
                .path(
                    sails_rtl::scale_info::Path::new_with_replace(
                        "Counter",
                        "references_app",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .composite(
                    sails_rtl::scale_info::build::Fields::named()
                        .field(|f| f.ty::<u32>().name("count").type_name("u32")),
                )
        }
    }
};
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct NamedCounter<'a> {
    name: &'a str,
    count: u32,
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for NamedCounter<'a> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "NamedCounter",
            "name",
            &self.name,
            "count",
            &&self.count,
        )
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl<'a> sails_rtl::scale_codec::Encode for NamedCounter<'a> {
        fn size_hint(&self) -> usize {
            0_usize
                .saturating_add(sails_rtl::scale_codec::Encode::size_hint(&self.name))
                .saturating_add(sails_rtl::scale_codec::Encode::size_hint(&self.count))
        }
        fn encode_to<
            __CodecOutputEdqy: sails_rtl::scale_codec::Output + ?::core::marker::Sized,
        >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
            sails_rtl::scale_codec::Encode::encode_to(&self.name, __codec_dest_edqy);
            sails_rtl::scale_codec::Encode::encode_to(&self.count, __codec_dest_edqy);
        }
    }
    #[automatically_derived]
    impl<'a> sails_rtl::scale_codec::EncodeLike for NamedCounter<'a> {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl<'a> sails_rtl::scale_codec::Decode for NamedCounter<'a> {
        fn decode<__CodecInputEdqy: sails_rtl::scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, sails_rtl::scale_codec::Error> {
            ::core::result::Result::Ok(NamedCounter::<'a> {
                name: {
                    let __codec_res_edqy = <&'a str as sails_rtl::scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `NamedCounter::name`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
                count: {
                    let __codec_res_edqy = <u32 as sails_rtl::scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `NamedCounter::count`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
            })
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl<'a> sails_rtl::scale_info::TypeInfo for NamedCounter<'a>
    where
        'a: 'static,
    {
        type Identity = Self;
        fn type_info() -> sails_rtl::scale_info::Type {
            sails_rtl::scale_info::Type::builder()
                .path(
                    sails_rtl::scale_info::Path::new_with_replace(
                        "NamedCounter",
                        "references_app",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .composite(
                    sails_rtl::scale_info::build::Fields::named()
                        .field(|f| {
                            f.ty::<&'static str>().name("name").type_name("&'static str")
                        })
                        .field(|f| f.ty::<u32>().name("count").type_name("u32")),
                )
        }
    }
};
pub struct Service<'a> {
    name: &'a str,
}
#[automatically_derived]
impl<'a> ::core::default::Default for Service<'a> {
    #[inline]
    fn default() -> Service<'a> {
        Service {
            name: ::core::default::Default::default(),
        }
    }
}
impl<'a> Service<'a> {
    pub const fn new(name: &'a str) -> Self {
        Self { name }
    }
    pub fn name(&self) -> &'a str {
        &self.name
    }
    pub fn baked(&self) -> &'static str {
        "Static str!"
    }
    pub fn incr(&mut self) -> &'a Counter {
        unsafe {
            COUNTER.count += 1;
            &*&raw const COUNTER
        }
    }
    pub fn add(&mut self, x: i32) -> Result<&Counter, &'static str> {
        if x < 0 {
            return Err("Can't add negative numbers");
        }
        unsafe {
            COUNTER.count += x as u32;
            Ok(&*&raw const COUNTER)
        }
    }
    pub fn crazy(
        &self,
    ) -> Result<Result<&'a Counter, Option<NamedCounter<'a>>>, &'a str> {
        Ok(
            Err(
                Some(NamedCounter {
                    name: "Named counter",
                    count: 42,
                }),
            ),
        )
    }
}
pub struct Exposure<T> {
    message_id: sails_rtl::MessageId,
    route: &'static [u8],
    inner: T,
}
impl<'a> Exposure<Service<'a>> {
    pub fn add(&mut self, x: i32) -> Result<&Counter, &'static str> {
        let exposure_scope = sails_rtl::gstd::services::ExposureCallScope::new(self);
        self.inner.add(x)
    }
    pub fn baked(&self) -> &'static str {
        let exposure_scope = sails_rtl::gstd::services::ExposureCallScope::new(self);
        self.inner.baked()
    }
    pub fn crazy(
        &self,
    ) -> Result<Result<&'a Counter, Option<NamedCounter<'a>>>, &'a str> {
        let exposure_scope = sails_rtl::gstd::services::ExposureCallScope::new(self);
        self.inner.crazy()
    }
    pub fn incr(&mut self) -> &'a Counter {
        let exposure_scope = sails_rtl::gstd::services::ExposureCallScope::new(self);
        self.inner.incr()
    }
    pub fn name(&self) -> &'a str {
        let exposure_scope = sails_rtl::gstd::services::ExposureCallScope::new(self);
        self.inner.name()
    }
    pub async fn handle(&mut self, mut input: &[u8]) -> Vec<u8> {
        if input.starts_with(&[12u8, 65u8, 100u8, 100u8]) {
            let output = self.__add(&input[4usize..]).await;
            static INVOCATION_ROUTE: [u8; 4usize] = [12u8, 65u8, 100u8, 100u8];
            return [INVOCATION_ROUTE.as_ref(), &output].concat();
        }
        if input.starts_with(&[20u8, 66u8, 97u8, 107u8, 101u8, 100u8]) {
            let output = self.__baked(&input[6usize..]).await;
            static INVOCATION_ROUTE: [u8; 6usize] = [
                20u8,
                66u8,
                97u8,
                107u8,
                101u8,
                100u8,
            ];
            return [INVOCATION_ROUTE.as_ref(), &output].concat();
        }
        if input.starts_with(&[20u8, 67u8, 114u8, 97u8, 122u8, 121u8]) {
            let output = self.__crazy(&input[6usize..]).await;
            static INVOCATION_ROUTE: [u8; 6usize] = [
                20u8,
                67u8,
                114u8,
                97u8,
                122u8,
                121u8,
            ];
            return [INVOCATION_ROUTE.as_ref(), &output].concat();
        }
        if input.starts_with(&[16u8, 73u8, 110u8, 99u8, 114u8]) {
            let output = self.__incr(&input[5usize..]).await;
            static INVOCATION_ROUTE: [u8; 5usize] = [16u8, 73u8, 110u8, 99u8, 114u8];
            return [INVOCATION_ROUTE.as_ref(), &output].concat();
        }
        if input.starts_with(&[16u8, 78u8, 97u8, 109u8, 101u8]) {
            let output = self.__name(&input[5usize..]).await;
            static INVOCATION_ROUTE: [u8; 5usize] = [16u8, 78u8, 97u8, 109u8, 101u8];
            return [INVOCATION_ROUTE.as_ref(), &output].concat();
        }
        let invocation_path = String::decode(&mut input)
            .expect("Failed to decode invocation path");
        {
            ::core::panicking::panic_fmt(
                format_args!("Unknown request: {0}", invocation_path),
            );
        };
    }
    async fn __add(&mut self, mut input: &[u8]) -> Vec<u8> {
        let request = __AddParams::decode(&mut input).expect("Failed to decode request");
        let result = self.add(request.x);
        return result.encode();
    }
    async fn __baked(&self, mut input: &[u8]) -> Vec<u8> {
        let request = __BakedParams::decode(&mut input)
            .expect("Failed to decode request");
        let result = self.baked();
        return result.encode();
    }
    async fn __crazy(&self, mut input: &[u8]) -> Vec<u8> {
        let request = __CrazyParams::decode(&mut input)
            .expect("Failed to decode request");
        let result = self.crazy();
        return result.encode();
    }
    async fn __incr(&mut self, mut input: &[u8]) -> Vec<u8> {
        let request = __IncrParams::decode(&mut input)
            .expect("Failed to decode request");
        let result = self.incr();
        return result.encode();
    }
    async fn __name(&self, mut input: &[u8]) -> Vec<u8> {
        let request = __NameParams::decode(&mut input)
            .expect("Failed to decode request");
        let result = self.name();
        return result.encode();
    }
}
impl<'a> sails_rtl::gstd::services::Exposure for Exposure<Service<'a>> {
    fn message_id(&self) -> sails_rtl::MessageId {
        self.message_id
    }
    fn route(&self) -> &'static [u8] {
        self.route
    }
}
impl<'a> sails_rtl::gstd::services::Service for Service<'a> {
    type Exposure = Exposure<Service<'a>>;
    fn expose(
        self,
        message_id: sails_rtl::MessageId,
        route: &'static [u8],
    ) -> Self::Exposure {
        Self::Exposure {
            message_id,
            route,
            inner: self,
        }
    }
}
impl<'a> sails_rtl::meta::ServiceMeta for Service<'a> {
    fn commands() -> sails_rtl::scale_info::MetaType {
        sails_rtl::scale_info::MetaType::new::<meta::CommandsMeta>()
    }
    fn queries() -> sails_rtl::scale_info::MetaType {
        sails_rtl::scale_info::MetaType::new::<meta::QueriesMeta>()
    }
    fn events() -> sails_rtl::scale_info::MetaType {
        sails_rtl::scale_info::MetaType::new::<meta::EventsMeta>()
    }
}
use sails_rtl::Decode as __ServiceDecode;
use sails_rtl::Encode as __ServiceEncode;
use sails_rtl::TypeInfo as __ServiceTypeInfo;
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct __AddParams {
    x: i32,
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl sails_rtl::scale_codec::Decode for __AddParams {
        fn decode<__CodecInputEdqy: sails_rtl::scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, sails_rtl::scale_codec::Error> {
            ::core::result::Result::Ok(__AddParams {
                x: {
                    let __codec_res_edqy = <i32 as sails_rtl::scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `__AddParams::x`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
            })
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl sails_rtl::scale_info::TypeInfo for __AddParams {
        type Identity = Self;
        fn type_info() -> sails_rtl::scale_info::Type {
            sails_rtl::scale_info::Type::builder()
                .path(
                    sails_rtl::scale_info::Path::new_with_replace(
                        "__AddParams",
                        "references_app",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .composite(
                    sails_rtl::scale_info::build::Fields::named()
                        .field(|f| f.ty::<i32>().name("x").type_name("i32")),
                )
        }
    }
};
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct __BakedParams {}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl sails_rtl::scale_codec::Decode for __BakedParams {
        fn decode<__CodecInputEdqy: sails_rtl::scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, sails_rtl::scale_codec::Error> {
            ::core::result::Result::Ok(__BakedParams {})
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl sails_rtl::scale_info::TypeInfo for __BakedParams {
        type Identity = Self;
        fn type_info() -> sails_rtl::scale_info::Type {
            sails_rtl::scale_info::Type::builder()
                .path(
                    sails_rtl::scale_info::Path::new_with_replace(
                        "__BakedParams",
                        "references_app",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .composite(sails_rtl::scale_info::build::Fields::named())
        }
    }
};
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct __CrazyParams {}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl sails_rtl::scale_codec::Decode for __CrazyParams {
        fn decode<__CodecInputEdqy: sails_rtl::scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, sails_rtl::scale_codec::Error> {
            ::core::result::Result::Ok(__CrazyParams {})
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl sails_rtl::scale_info::TypeInfo for __CrazyParams {
        type Identity = Self;
        fn type_info() -> sails_rtl::scale_info::Type {
            sails_rtl::scale_info::Type::builder()
                .path(
                    sails_rtl::scale_info::Path::new_with_replace(
                        "__CrazyParams",
                        "references_app",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .composite(sails_rtl::scale_info::build::Fields::named())
        }
    }
};
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct __IncrParams {}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl sails_rtl::scale_codec::Decode for __IncrParams {
        fn decode<__CodecInputEdqy: sails_rtl::scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, sails_rtl::scale_codec::Error> {
            ::core::result::Result::Ok(__IncrParams {})
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl sails_rtl::scale_info::TypeInfo for __IncrParams {
        type Identity = Self;
        fn type_info() -> sails_rtl::scale_info::Type {
            sails_rtl::scale_info::Type::builder()
                .path(
                    sails_rtl::scale_info::Path::new_with_replace(
                        "__IncrParams",
                        "references_app",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .composite(sails_rtl::scale_info::build::Fields::named())
        }
    }
};
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
pub struct __NameParams {}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl sails_rtl::scale_codec::Decode for __NameParams {
        fn decode<__CodecInputEdqy: sails_rtl::scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, sails_rtl::scale_codec::Error> {
            ::core::result::Result::Ok(__NameParams {})
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl sails_rtl::scale_info::TypeInfo for __NameParams {
        type Identity = Self;
        fn type_info() -> sails_rtl::scale_info::Type {
            sails_rtl::scale_info::Type::builder()
                .path(
                    sails_rtl::scale_info::Path::new_with_replace(
                        "__NameParams",
                        "references_app",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .composite(sails_rtl::scale_info::build::Fields::named())
        }
    }
};
mod meta {
    use super::*;
    #[scale_info(crate = sails_rtl::scale_info)]
    pub enum CommandsMeta {
        Add(__AddParams, Result<Counter, String>),
        Incr(__IncrParams, Counter),
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl sails_rtl::scale_info::TypeInfo for CommandsMeta {
            type Identity = Self;
            fn type_info() -> sails_rtl::scale_info::Type {
                sails_rtl::scale_info::Type::builder()
                    .path(
                        sails_rtl::scale_info::Path::new_with_replace(
                            "CommandsMeta",
                            "references_app::meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rtl::scale_info::build::Variants::new()
                            .variant(
                                "Add",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rtl::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<__AddParams>().type_name("__AddParams"))
                                                .field(|f| {
                                                    f
                                                        .ty::<Result<Counter, String>>()
                                                        .type_name("Result<Counter, String>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "Incr",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rtl::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<__IncrParams>().type_name("__IncrParams"))
                                                .field(|f| f.ty::<Counter>().type_name("Counter")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    #[scale_info(crate = sails_rtl::scale_info)]
    pub enum QueriesMeta {
        Baked(__BakedParams, String),
        Crazy(__CrazyParams, Result<Result<Counter, Option<NamedCounter<'a>>>, String>),
        Name(__NameParams, String),
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl sails_rtl::scale_info::TypeInfo for QueriesMeta {
            type Identity = Self;
            fn type_info() -> sails_rtl::scale_info::Type {
                sails_rtl::scale_info::Type::builder()
                    .path(
                        sails_rtl::scale_info::Path::new_with_replace(
                            "QueriesMeta",
                            "references_app::meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rtl::scale_info::build::Variants::new()
                            .variant(
                                "Baked",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rtl::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f.ty::<__BakedParams>().type_name("__BakedParams")
                                                })
                                                .field(|f| f.ty::<String>().type_name("String")),
                                        )
                                },
                            )
                            .variant(
                                "Crazy",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rtl::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f.ty::<__CrazyParams>().type_name("__CrazyParams")
                                                })
                                                .field(|f| {
                                                    f
                                                        .ty::<
                                                            Result<
                                                                Result<Counter, Option<NamedCounter<'static>>>,
                                                                String,
                                                            >,
                                                        >()
                                                        .type_name(
                                                            "Result<Result<Counter, Option<NamedCounter<'static>>>, String>",
                                                        )
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "Name",
                                |v| {
                                    v
                                        .index(2usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rtl::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<__NameParams>().type_name("__NameParams"))
                                                .field(|f| f.ty::<String>().type_name("String")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    #[scale_info(crate = sails_rtl::scale_info)]
    pub enum NoEvents {}
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl sails_rtl::scale_info::TypeInfo for NoEvents {
            type Identity = Self;
            fn type_info() -> sails_rtl::scale_info::Type {
                sails_rtl::scale_info::Type::builder()
                    .path(
                        sails_rtl::scale_info::Path::new_with_replace(
                            "NoEvents",
                            "references_app::meta",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(sails_rtl::scale_info::build::Variants::new())
            }
        }
    };
    pub type EventsMeta = NoEvents;
}
