
#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use core::prelude::rust_2024::*;
#[macro_use]
extern crate core;
use demo_walker as walker;
use sails_rs::{cell::RefCell, prelude::*};
mod counter {
    use sails_rs::{cell::RefCell, prelude::*};
    pub struct CounterData {
        counter: u32,
    }
    impl CounterData {
        pub const fn new(counter: u32) -> Self {
            Self { counter }
        }
    }
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum CounterEvents {
        /// Emitted when a new value is added to the counter
        Added(u32),
        /// Emitted when a value is subtracted from the counter
        Subtracted(u32),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for CounterEvents {
        #[inline]
        fn clone(&self) -> CounterEvents {
            match self {
                CounterEvents::Added(__self_0) => {
                    CounterEvents::Added(::core::clone::Clone::clone(__self_0))
                }
                CounterEvents::Subtracted(__self_0) => {
                    CounterEvents::Subtracted(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for CounterEvents {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                CounterEvents::Added(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Added",
                        &__self_0,
                    )
                }
                CounterEvents::Subtracted(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Subtracted",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for CounterEvents {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for CounterEvents {
        #[inline]
        fn eq(&self, other: &CounterEvents) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (CounterEvents::Added(__self_0), CounterEvents::Added(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    (
                        CounterEvents::Subtracted(__self_0),
                        CounterEvents::Subtracted(__arg1_0),
                    ) => __self_0 == __arg1_0,
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Encode for CounterEvents {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        CounterEvents::Added(ref aa) => {
                            0_usize
                                .saturating_add(
                                    sails_rs::scale_codec::Encode::size_hint(aa),
                                )
                        }
                        CounterEvents::Subtracted(ref aa) => {
                            0_usize
                                .saturating_add(
                                    sails_rs::scale_codec::Encode::size_hint(aa),
                                )
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: sails_rs::scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                #[automatically_derived]
                const _: () = {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    const indices: [(usize, &'static str); 2usize] = [
                        ((0usize) as ::core::primitive::usize, "Added"),
                        ((1usize) as ::core::primitive::usize, "Subtracted"),
                    ];
                    const fn search_for_invalid_index(
                        array: &[(usize, &'static str); 2usize],
                    ) -> (bool, usize) {
                        let mut i = 0;
                        while i < 2usize {
                            if array[i].0 > 255 {
                                return (true, i);
                            }
                            i += 1;
                        }
                        (false, 0)
                    }
                    const INVALID_INDEX: (bool, usize) = search_for_invalid_index(
                        &indices,
                    );
                    if INVALID_INDEX.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper("Found variant `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].1,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` with invalid index: `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].0,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Max supported index is 255.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                    const fn duplicate_info(
                        array: &[(usize, &'static str); 2usize],
                    ) -> (bool, usize, usize) {
                        let len = 2usize;
                        let mut i = 0usize;
                        while i < len {
                            let mut j = i + 1;
                            while j < len {
                                if array[i].0 == array[j].0 {
                                    return (true, i, j);
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        (false, 0, 0)
                    }
                    const DUP_INFO: (bool, usize, usize) = duplicate_info(&indices);
                    if DUP_INFO.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "Found variants that have duplicate indexes. Both `",
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` and `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.2].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` have the index `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].0)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Use different indexes for each variant.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                };
                match *self {
                    CounterEvents::Added(ref aa) => {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_dest_edqy.push_byte((0usize) as ::core::primitive::u8);
                        sails_rs::scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    CounterEvents::Subtracted(ref aa) => {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_dest_edqy.push_byte((1usize) as ::core::primitive::u8);
                        sails_rs::scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl sails_rs::scale_codec::EncodeLike for CounterEvents {}
    };
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for CounterEvents {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "CounterEvents",
                            "demo::counter",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant(
                                "Added",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<u32>().type_name("u32")),
                                        )
                                        .docs(&["Emitted when a new value is added to the counter"])
                                },
                            )
                            .variant(
                                "Subtracted",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<u32>().type_name("u32")),
                                        )
                                        .docs(
                                            &["Emitted when a value is subtracted from the counter"],
                                        )
                                },
                            ),
                    )
            }
        }
    };
    impl sails_rs::SailsEvent for CounterEvents {
        fn encoded_event_name(&self) -> &'static [u8] {
            match self {
                CounterEvents::Added(..) => &[20u8, 65u8, 100u8, 100u8, 101u8, 100u8],
                CounterEvents::Subtracted(..) => {
                    &[
                        40u8, 83u8, 117u8, 98u8, 116u8, 114u8, 97u8, 99u8, 116u8, 101u8,
                        100u8,
                    ]
                }
            }
        }
        fn skip_bytes() -> usize {
            1
        }
    }
    pub struct CounterService<'a> {
        data: &'a RefCell<CounterData>,
    }
    impl<'a> CounterService<'a> {
        pub fn new(data: &'a RefCell<CounterData>) -> Self {
            Self { data }
        }
    }
    pub struct CounterServiceExposure<T> {
        route: &'static [u8],
        inner: T,
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::Exposure
    for CounterServiceExposure<T> {
        fn route(&self) -> &'static [u8] {
            self.route
        }
        fn check_asyncness(input: &[u8]) -> Option<bool> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if !T::ASYNC {
                return Some(false);
            }
            if let Ok(is_async) = counter_service_meta::__AddParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = counter_service_meta::__SubParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = counter_service_meta::__ValueParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            None
        }
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::ExposureWithEvents
    for CounterServiceExposure<T> {
        type Events = CounterEvents;
    }
    impl<T> core::ops::Deref for CounterServiceExposure<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T> core::ops::DerefMut for CounterServiceExposure<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl CounterServiceExposure<CounterService<'_>> {
        /// Add a value to the counter
        pub fn add(&mut self, value: u32) -> u32 {
            let mut data_mut = self.data.borrow_mut();
            data_mut.counter += value;
            self.emit_event(CounterEvents::Added(value)).unwrap();
            data_mut.counter
        }
        /// Substract a value from the counter
        pub fn sub(&mut self, value: u32) -> u32 {
            let mut data_mut = self.data.borrow_mut();
            data_mut.counter -= value;
            self.emit_event(CounterEvents::Subtracted(value)).unwrap();
            data_mut.counter
        }
        /// Get the current value
        pub fn value(&self) -> u32 {
            self.data.borrow().counter
        }
        pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
            <Self as sails_rs::gstd::services::Exposure>::check_asyncness(input)
        }
        pub fn try_handle(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = counter_service_meta::__AddParams::decode_params(
                input,
            ) {
                let result = self.add(request.value);
                let value = 0u128;
                if !counter_service_meta::__AddParams::is_empty_tuple::<u32>() {
                    counter_service_meta::__AddParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = counter_service_meta::__SubParams::decode_params(
                input,
            ) {
                let result = self.sub(request.value);
                let value = 0u128;
                if !counter_service_meta::__SubParams::is_empty_tuple::<u32>() {
                    counter_service_meta::__SubParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = counter_service_meta::__ValueParams::decode_params(
                input,
            ) {
                let result = self.value();
                let value = 0u128;
                if !counter_service_meta::__ValueParams::is_empty_tuple::<u32>() {
                    counter_service_meta::__ValueParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
        pub async fn try_handle_async(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            None
        }
        pub fn emit_event(&self, event: CounterEvents) -> sails_rs::errors::Result<()> {
            use sails_rs::gstd::services::ExposureWithEvents;
            self.emitter().emit_event(event)
        }
    }
    impl sails_rs::gstd::services::Service for CounterService<'_> {
        type Exposure = CounterServiceExposure<Self>;
        fn expose(self, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                route,
                inner: self,
            }
        }
    }
    impl sails_rs::meta::ServiceMeta for CounterService<'_> {
        type CommandsMeta = counter_service_meta::CommandsMeta;
        type QueriesMeta = counter_service_meta::QueriesMeta;
        type EventsMeta = counter_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
        const ASYNC: bool = false;
    }
    mod counter_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo};
        use sails_rs::gstd::InvocationIo;
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __AddParams {
            pub(super) value: u32,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __AddParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__AddParams {
                        value: {
                            let __codec_res_edqy = <u32 as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__AddParams::value`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __AddParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__AddParams",
                                "demo::counter::counter_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| f.ty::<u32>().name("value").type_name("u32")),
                        )
                }
            }
        };
        impl InvocationIo for __AddParams {
            const ROUTE: &'static [u8] = &[12u8, 65u8, 100u8, 100u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __SubParams {
            pub(super) value: u32,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __SubParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__SubParams {
                        value: {
                            let __codec_res_edqy = <u32 as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__SubParams::value`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __SubParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__SubParams",
                                "demo::counter::counter_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| f.ty::<u32>().name("value").type_name("u32")),
                        )
                }
            }
        };
        impl InvocationIo for __SubParams {
            const ROUTE: &'static [u8] = &[12u8, 83u8, 117u8, 98u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __ValueParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __ValueParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__ValueParams {})
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
            impl sails_rs::scale_info::TypeInfo for __ValueParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__ValueParams",
                                "demo::counter::counter_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __ValueParams {
            const ROUTE: &'static [u8] = &[20u8, 86u8, 97u8, 108u8, 117u8, 101u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            /// Add a value to the counter
            Add(__AddParams, u32),
            /// Substract a value from the counter
            Sub(__SubParams, u32),
        }
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
                                "demo::counter::counter_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "Add",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__AddParams>().type_name("__AddParams"))
                                                    .field(|f| f.ty::<u32>().type_name("u32")),
                                            )
                                            .docs(&["Add a value to the counter"])
                                    },
                                )
                                .variant(
                                    "Sub",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__SubParams>().type_name("__SubParams"))
                                                    .field(|f| f.ty::<u32>().type_name("u32")),
                                            )
                                            .docs(&["Substract a value from the counter"])
                                    },
                                ),
                        )
                }
            }
        };
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {
            /// Get the current value
            Value(__ValueParams, u32),
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
                                "demo::counter::counter_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "Value",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__ValueParams>().type_name("__ValueParams")
                                                    })
                                                    .field(|f| f.ty::<u32>().type_name("u32")),
                                            )
                                            .docs(&["Get the current value"])
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
                                "demo::counter::counter_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(sails_rs::scale_info::build::Variants::new())
                }
            }
        };
        pub type EventsMeta = CounterEvents;
    }
}
mod dog {
    use crate::mammal::MammalService;
    use demo_walker::WalkerService;
    use sails_rs::prelude::*;
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum DogEvents {
        Barked,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DogEvents {
        #[inline]
        fn clone(&self) -> DogEvents {
            DogEvents::Barked
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DogEvents {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "Barked")
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for DogEvents {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for DogEvents {
        #[inline]
        fn eq(&self, other: &DogEvents) -> bool {
            true
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Encode for DogEvents {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        DogEvents::Barked => 0_usize,
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: sails_rs::scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                #[automatically_derived]
                const _: () = {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    const indices: [(usize, &'static str); 1usize] = [
                        ((0usize) as ::core::primitive::usize, "Barked"),
                    ];
                    const fn search_for_invalid_index(
                        array: &[(usize, &'static str); 1usize],
                    ) -> (bool, usize) {
                        let mut i = 0;
                        while i < 1usize {
                            if array[i].0 > 255 {
                                return (true, i);
                            }
                            i += 1;
                        }
                        (false, 0)
                    }
                    const INVALID_INDEX: (bool, usize) = search_for_invalid_index(
                        &indices,
                    );
                    if INVALID_INDEX.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper("Found variant `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].1,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` with invalid index: `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].0,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Max supported index is 255.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                    const fn duplicate_info(
                        array: &[(usize, &'static str); 1usize],
                    ) -> (bool, usize, usize) {
                        let len = 1usize;
                        let mut i = 0usize;
                        while i < len {
                            let mut j = i + 1;
                            while j < len {
                                if array[i].0 == array[j].0 {
                                    return (true, i, j);
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        (false, 0, 0)
                    }
                    const DUP_INFO: (bool, usize, usize) = duplicate_info(&indices);
                    if DUP_INFO.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "Found variants that have duplicate indexes. Both `",
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` and `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.2].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` have the index `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].0)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Use different indexes for each variant.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                };
                match *self {
                    DogEvents::Barked => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((0usize) as ::core::primitive::u8);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl sails_rs::scale_codec::EncodeLike for DogEvents {}
    };
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for DogEvents {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "DogEvents",
                            "demo::dog",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant(
                                "Barked",
                                |v| v.index(0usize as ::core::primitive::u8),
                            ),
                    )
            }
        }
    };
    impl sails_rs::SailsEvent for DogEvents {
        fn encoded_event_name(&self) -> &'static [u8] {
            match self {
                DogEvents::Barked => &[24u8, 66u8, 97u8, 114u8, 107u8, 101u8, 100u8],
            }
        }
        fn skip_bytes() -> usize {
            1
        }
    }
    pub struct DogService {
        walker: WalkerService,
        mammal: MammalService,
    }
    impl DogService {
        pub fn new(walker: WalkerService) -> Self {
            Self {
                walker,
                mammal: MammalService::new(42),
            }
        }
    }
    impl From<DogService> for (MammalService, WalkerService) {
        fn from(value: DogService) -> Self {
            (value.mammal, value.walker)
        }
    }
    pub struct DogServiceExposure<T> {
        route: &'static [u8],
        inner: T,
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::Exposure
    for DogServiceExposure<T> {
        fn route(&self) -> &'static [u8] {
            self.route
        }
        fn check_asyncness(input: &[u8]) -> Option<bool> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if !T::ASYNC {
                return Some(false);
            }
            if let Ok(is_async) = dog_service_meta::__MakeSoundParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Some(is_async) = <<MammalService as Service>::Exposure as Exposure>::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Some(is_async) = <<WalkerService as Service>::Exposure as Exposure>::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            None
        }
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::ExposureWithEvents
    for DogServiceExposure<T> {
        type Events = DogEvents;
    }
    impl<T> core::ops::Deref for DogServiceExposure<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T> core::ops::DerefMut for DogServiceExposure<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    #[allow(clippy::from_over_into)]
    impl Into<MammalService> for DogServiceExposure<DogService> {
        fn into(self) -> MammalService {
            let base_services: (MammalService, WalkerService) = self.inner.into();
            base_services.0
        }
    }
    #[allow(clippy::from_over_into)]
    impl Into<WalkerService> for DogServiceExposure<DogService> {
        fn into(self) -> WalkerService {
            let base_services: (MammalService, WalkerService) = self.inner.into();
            base_services.1
        }
    }
    impl DogServiceExposure<DogService> {
        pub fn make_sound(&mut self) -> &'static str {
            self.emit_event(DogEvents::Barked).unwrap();
            "Woof! Woof!"
        }
        pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
            <Self as sails_rs::gstd::services::Exposure>::check_asyncness(input)
        }
        pub fn try_handle(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = dog_service_meta::__MakeSoundParams::decode_params(
                input,
            ) {
                let result = self.make_sound();
                let value = 0u128;
                if !dog_service_meta::__MakeSoundParams::is_empty_tuple::<
                    &'static str,
                >() {
                    dog_service_meta::__MakeSoundParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            let base_services: (MammalService, WalkerService) = self.inner.into();
            if base_services
                .0
                .expose(self.route)
                .try_handle(input, result_handler)
                .is_some()
            {
                return Some(());
            }
            if base_services
                .1
                .expose(self.route)
                .try_handle(input, result_handler)
                .is_some()
            {
                return Some(());
            }
            None
        }
        pub async fn try_handle_async(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            let base_services: (MammalService, WalkerService) = self.inner.into();
            if base_services
                .0
                .expose(self.route)
                .try_handle_async(input, result_handler)
                .await
                .is_some()
            {
                return Some(());
            }
            if base_services
                .1
                .expose(self.route)
                .try_handle_async(input, result_handler)
                .await
                .is_some()
            {
                return Some(());
            }
            None
        }
        pub fn emit_event(&self, event: DogEvents) -> sails_rs::errors::Result<()> {
            use sails_rs::gstd::services::ExposureWithEvents;
            self.emitter().emit_event(event)
        }
    }
    impl sails_rs::gstd::services::Service for DogService {
        type Exposure = DogServiceExposure<Self>;
        fn expose(self, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                route,
                inner: self,
            }
        }
    }
    impl sails_rs::meta::ServiceMeta for DogService {
        type CommandsMeta = dog_service_meta::CommandsMeta;
        type QueriesMeta = dog_service_meta::QueriesMeta;
        type EventsMeta = dog_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[
            sails_rs::meta::AnyServiceMeta::new::<MammalService>,
            sails_rs::meta::AnyServiceMeta::new::<WalkerService>,
        ];
        const ASYNC: bool = <MammalService as sails_rs::meta::ServiceMeta>::ASYNC
            || <WalkerService as sails_rs::meta::ServiceMeta>::ASYNC;
    }
    mod dog_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo};
        use sails_rs::gstd::InvocationIo;
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __MakeSoundParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __MakeSoundParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__MakeSoundParams {})
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
            impl sails_rs::scale_info::TypeInfo for __MakeSoundParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__MakeSoundParams",
                                "demo::dog::dog_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __MakeSoundParams {
            const ROUTE: &'static [u8] = &[
                36u8, 77u8, 97u8, 107u8, 101u8, 83u8, 111u8, 117u8, 110u8, 100u8,
            ];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            MakeSound(__MakeSoundParams, &'static str),
        }
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
                                "demo::dog::dog_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "MakeSound",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__MakeSoundParams>().type_name("__MakeSoundParams")
                                                    })
                                                    .field(|f| f.ty::<&'static str>().type_name("&'static str")),
                                            )
                                    },
                                ),
                        )
                }
            }
        };
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {}
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
                                "demo::dog::dog_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(sails_rs::scale_info::build::Variants::new())
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
                                "demo::dog::dog_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(sails_rs::scale_info::build::Variants::new())
                }
            }
        };
        pub type EventsMeta = DogEvents;
    }
}
mod mammal {
    use sails_rs::prelude::*;
    pub struct MammalService {
        avg_weight: u32,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for MammalService {
        #[inline]
        fn clone(&self) -> MammalService {
            MammalService {
                avg_weight: ::core::clone::Clone::clone(&self.avg_weight),
            }
        }
    }
    impl MammalService {
        pub const fn new(avg_weight: u32) -> Self {
            Self { avg_weight }
        }
    }
    pub struct MammalServiceExposure<T> {
        route: &'static [u8],
        inner: T,
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::Exposure
    for MammalServiceExposure<T> {
        fn route(&self) -> &'static [u8] {
            self.route
        }
        fn check_asyncness(input: &[u8]) -> Option<bool> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if !T::ASYNC {
                return Some(false);
            }
            if let Ok(is_async) = mammal_service_meta::__AvgWeightParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = mammal_service_meta::__MakeSoundParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            None
        }
    }
    impl<T> core::ops::Deref for MammalServiceExposure<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T> core::ops::DerefMut for MammalServiceExposure<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl MammalServiceExposure<MammalService> {
        pub fn make_sound(&mut self) -> &'static str {
            {
                ::core::panicking::panic_fmt(format_args!("Not implemented"));
            }
        }
        pub fn avg_weight(&self) -> u32 {
            self.avg_weight
        }
        pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
            <Self as sails_rs::gstd::services::Exposure>::check_asyncness(input)
        }
        pub fn try_handle(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = mammal_service_meta::__AvgWeightParams::decode_params(
                input,
            ) {
                let result = self.avg_weight();
                let value = 0u128;
                if !mammal_service_meta::__AvgWeightParams::is_empty_tuple::<u32>() {
                    mammal_service_meta::__AvgWeightParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = mammal_service_meta::__MakeSoundParams::decode_params(
                input,
            ) {
                let result = self.make_sound();
                let value = 0u128;
                if !mammal_service_meta::__MakeSoundParams::is_empty_tuple::<
                    &'static str,
                >() {
                    mammal_service_meta::__MakeSoundParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
        pub async fn try_handle_async(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            None
        }
    }
    impl sails_rs::gstd::services::Service for MammalService {
        type Exposure = MammalServiceExposure<Self>;
        fn expose(self, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                route,
                inner: self,
            }
        }
    }
    impl sails_rs::meta::ServiceMeta for MammalService {
        type CommandsMeta = mammal_service_meta::CommandsMeta;
        type QueriesMeta = mammal_service_meta::QueriesMeta;
        type EventsMeta = mammal_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
        const ASYNC: bool = false;
    }
    mod mammal_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo};
        use sails_rs::gstd::InvocationIo;
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __AvgWeightParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __AvgWeightParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__AvgWeightParams {})
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
            impl sails_rs::scale_info::TypeInfo for __AvgWeightParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__AvgWeightParams",
                                "demo::mammal::mammal_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __AvgWeightParams {
            const ROUTE: &'static [u8] = &[
                36u8, 65u8, 118u8, 103u8, 87u8, 101u8, 105u8, 103u8, 104u8, 116u8,
            ];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __MakeSoundParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __MakeSoundParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__MakeSoundParams {})
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
            impl sails_rs::scale_info::TypeInfo for __MakeSoundParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__MakeSoundParams",
                                "demo::mammal::mammal_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __MakeSoundParams {
            const ROUTE: &'static [u8] = &[
                36u8, 77u8, 97u8, 107u8, 101u8, 83u8, 111u8, 117u8, 110u8, 100u8,
            ];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            MakeSound(__MakeSoundParams, &'static str),
        }
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
                                "demo::mammal::mammal_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "MakeSound",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__MakeSoundParams>().type_name("__MakeSoundParams")
                                                    })
                                                    .field(|f| f.ty::<&'static str>().type_name("&'static str")),
                                            )
                                    },
                                ),
                        )
                }
            }
        };
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {
            AvgWeight(__AvgWeightParams, u32),
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
                                "demo::mammal::mammal_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "AvgWeight",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__AvgWeightParams>().type_name("__AvgWeightParams")
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
                                "demo::mammal::mammal_service_meta",
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
}
mod ping {
    use sails_rs::prelude::*;
    pub struct PingService(());
    #[automatically_derived]
    impl ::core::default::Default for PingService {
        #[inline]
        fn default() -> PingService {
            PingService(::core::default::Default::default())
        }
    }
    pub struct PingServiceExposure<T> {
        route: &'static [u8],
        inner: T,
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::Exposure
    for PingServiceExposure<T> {
        fn route(&self) -> &'static [u8] {
            self.route
        }
        fn check_asyncness(input: &[u8]) -> Option<bool> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if !T::ASYNC {
                return Some(false);
            }
            if let Ok(is_async) = ping_service_meta::__PingParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            None
        }
    }
    impl<T> core::ops::Deref for PingServiceExposure<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T> core::ops::DerefMut for PingServiceExposure<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl PingServiceExposure<PingService> {
        pub fn ping(&mut self, input: String) -> Result<String, String> {
            if input != "ping" { Err("Invalid input".into()) } else { Ok("pong".into()) }
        }
        pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
            <Self as sails_rs::gstd::services::Exposure>::check_asyncness(input)
        }
        pub fn try_handle(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = ping_service_meta::__PingParams::decode_params(input) {
                let result = self.ping(request.input);
                let value = 0u128;
                if !ping_service_meta::__PingParams::is_empty_tuple::<
                    Result<String, String>,
                >() {
                    ping_service_meta::__PingParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
        pub async fn try_handle_async(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            None
        }
    }
    impl sails_rs::gstd::services::Service for PingService {
        type Exposure = PingServiceExposure<Self>;
        fn expose(self, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                route,
                inner: self,
            }
        }
    }
    impl sails_rs::meta::ServiceMeta for PingService {
        type CommandsMeta = ping_service_meta::CommandsMeta;
        type QueriesMeta = ping_service_meta::QueriesMeta;
        type EventsMeta = ping_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
        const ASYNC: bool = false;
    }
    mod ping_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo};
        use sails_rs::gstd::InvocationIo;
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __PingParams {
            pub(super) input: String,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __PingParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__PingParams {
                        input: {
                            let __codec_res_edqy = <String as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__PingParams::input`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __PingParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__PingParams",
                                "demo::ping::ping_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| {
                                    f.ty::<String>().name("input").type_name("String")
                                }),
                        )
                }
            }
        };
        impl InvocationIo for __PingParams {
            const ROUTE: &'static [u8] = &[16u8, 80u8, 105u8, 110u8, 103u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            Ping(__PingParams, Result<String, String>),
        }
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
                                "demo::ping::ping_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "Ping",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__PingParams>().type_name("__PingParams"))
                                                    .field(|f| {
                                                        f
                                                            .ty::<Result<String, String>>()
                                                            .type_name("Result<String, String>")
                                                    }),
                                            )
                                    },
                                ),
                        )
                }
            }
        };
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {}
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
                                "demo::ping::ping_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(sails_rs::scale_info::build::Variants::new())
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
                                "demo::ping::ping_service_meta",
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
}
mod references {
    use core::ptr;
    use sails_rs::prelude::*;
    static mut COUNTER: ReferenceCount = ReferenceCount(0);
    static mut BYTES: Vec<u8> = Vec::new();
    pub struct ReferenceService<'a> {
        data: Option<ReferenceData<'a>>,
    }
    #[automatically_derived]
    impl<'a> ::core::default::Default for ReferenceService<'a> {
        #[inline]
        fn default() -> ReferenceService<'a> {
            ReferenceService {
                data: ::core::default::Default::default(),
            }
        }
    }
    struct ReferenceData<'a> {
        num: &'a mut u8,
        message: &'a str,
    }
    impl<'a> ReferenceService<'a> {
        pub fn new(num: &'a mut u8, message: &'a str) -> Self {
            let data = ReferenceData { num, message };
            Self { data: Some(data) }
        }
    }
    pub struct ReferenceServiceExposure<T> {
        route: &'static [u8],
        inner: T,
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::Exposure
    for ReferenceServiceExposure<T> {
        fn route(&self) -> &'static [u8] {
            self.route
        }
        fn check_asyncness(input: &[u8]) -> Option<bool> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if !T::ASYNC {
                return Some(false);
            }
            if let Ok(is_async) = reference_service_meta::__AddParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = reference_service_meta::__AddByteParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = reference_service_meta::__BakedParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = reference_service_meta::__GuessNumParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = reference_service_meta::__IncrParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = reference_service_meta::__LastByteParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = reference_service_meta::__MessageParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = reference_service_meta::__SetNumParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            None
        }
    }
    impl<T> core::ops::Deref for ReferenceServiceExposure<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T> core::ops::DerefMut for ReferenceServiceExposure<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl<'t> ReferenceServiceExposure<ReferenceService<'t>> {
        pub fn baked(&self) -> &'static str {
            "Static str!"
        }
        pub fn incr(&mut self) -> &'static ReferenceCount {
            unsafe {
                COUNTER.0 += 1;
                &*&raw const COUNTER
            }
        }
        #[allow(static_mut_refs)]
        pub fn add<'a>(&mut self, v: u32) -> &'a u32 {
            unsafe {
                COUNTER.0 += v;
                &COUNTER.0
            }
        }
        #[allow(static_mut_refs)]
        pub fn add_byte(&mut self, byte: u8) -> &'static [u8] {
            unsafe {
                BYTES.push(byte);
                &*&raw const BYTES
            }
        }
        #[allow(static_mut_refs)]
        pub async fn last_byte<'a>(&self) -> Option<&'a u8> {
            unsafe { BYTES.last() }
        }
        pub async fn guess_num(&mut self, number: u8) -> Result<&'t str, &'static str> {
            if number > 42 {
                Err("Number is too large")
            } else if let Some(data) = &self.data.as_ref() {
                if *data.num == number { Ok(data.message) } else { Err("Try again") }
            } else {
                Err("Data is not set")
            }
        }
        pub async fn message(&self) -> Option<&'t str> {
            self.data.as_ref().map(|d| d.message)
        }
        pub async fn set_num(&mut self, number: u8) -> Result<(), &'static str> {
            if number > 42 {
                Err("Number is too large")
            } else if let Some(data) = self.data.as_mut() {
                *data.num = number;
                Ok(())
            } else {
                Err("Data is not set")
            }
        }
        pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
            <Self as sails_rs::gstd::services::Exposure>::check_asyncness(input)
        }
        pub fn try_handle(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = reference_service_meta::__AddParams::decode_params(
                input,
            ) {
                let result = self.add(request.v);
                let value = 0u128;
                if !reference_service_meta::__AddParams::is_empty_tuple::<
                    &'static u32,
                >() {
                    reference_service_meta::__AddParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = reference_service_meta::__AddByteParams::decode_params(
                input,
            ) {
                let result = self.add_byte(request.byte);
                let value = 0u128;
                if !reference_service_meta::__AddByteParams::is_empty_tuple::<
                    &'static [u8],
                >() {
                    reference_service_meta::__AddByteParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = reference_service_meta::__BakedParams::decode_params(
                input,
            ) {
                let result = self.baked();
                let value = 0u128;
                if !reference_service_meta::__BakedParams::is_empty_tuple::<
                    &'static str,
                >() {
                    reference_service_meta::__BakedParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = reference_service_meta::__IncrParams::decode_params(
                input,
            ) {
                let result = self.incr();
                let value = 0u128;
                if !reference_service_meta::__IncrParams::is_empty_tuple::<
                    &'static ReferenceCount,
                >() {
                    reference_service_meta::__IncrParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
        pub async fn try_handle_async(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = reference_service_meta::__GuessNumParams::decode_params(
                input,
            ) {
                let result = self.guess_num(request.number).await;
                let value = 0u128;
                if !reference_service_meta::__GuessNumParams::is_empty_tuple::<
                    Result<&'static str, &'static str>,
                >() {
                    reference_service_meta::__GuessNumParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = reference_service_meta::__LastByteParams::decode_params(
                input,
            ) {
                let result = self.last_byte().await;
                let value = 0u128;
                if !reference_service_meta::__LastByteParams::is_empty_tuple::<
                    Option<&'static u8>,
                >() {
                    reference_service_meta::__LastByteParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = reference_service_meta::__MessageParams::decode_params(
                input,
            ) {
                let result = self.message().await;
                let value = 0u128;
                if !reference_service_meta::__MessageParams::is_empty_tuple::<
                    Option<&'static str>,
                >() {
                    reference_service_meta::__MessageParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = reference_service_meta::__SetNumParams::decode_params(
                input,
            ) {
                let result = self.set_num(request.number).await;
                let value = 0u128;
                if !reference_service_meta::__SetNumParams::is_empty_tuple::<
                    Result<(), &'static str>,
                >() {
                    reference_service_meta::__SetNumParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
    }
    impl<'t> sails_rs::gstd::services::Service for ReferenceService<'t> {
        type Exposure = ReferenceServiceExposure<Self>;
        fn expose(self, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                route,
                inner: self,
            }
        }
    }
    impl<'t> sails_rs::meta::ServiceMeta for ReferenceService<'t> {
        type CommandsMeta = reference_service_meta::CommandsMeta;
        type QueriesMeta = reference_service_meta::QueriesMeta;
        type EventsMeta = reference_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
        const ASYNC: bool = true;
    }
    mod reference_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo};
        use sails_rs::gstd::InvocationIo;
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __AddParams {
            pub(super) v: u32,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __AddParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__AddParams {
                        v: {
                            let __codec_res_edqy = <u32 as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__AddParams::v`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __AddParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__AddParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| f.ty::<u32>().name("v").type_name("u32")),
                        )
                }
            }
        };
        impl InvocationIo for __AddParams {
            const ROUTE: &'static [u8] = &[12u8, 65u8, 100u8, 100u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __AddByteParams {
            pub(super) byte: u8,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __AddByteParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__AddByteParams {
                        byte: {
                            let __codec_res_edqy = <u8 as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__AddByteParams::byte`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __AddByteParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__AddByteParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| f.ty::<u8>().name("byte").type_name("u8")),
                        )
                }
            }
        };
        impl InvocationIo for __AddByteParams {
            const ROUTE: &'static [u8] = &[
                28u8, 65u8, 100u8, 100u8, 66u8, 121u8, 116u8, 101u8,
            ];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __BakedParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __BakedParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__BakedParams {})
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
            impl sails_rs::scale_info::TypeInfo for __BakedParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__BakedParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __BakedParams {
            const ROUTE: &'static [u8] = &[20u8, 66u8, 97u8, 107u8, 101u8, 100u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __GuessNumParams {
            pub(super) number: u8,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __GuessNumParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__GuessNumParams {
                        number: {
                            let __codec_res_edqy = <u8 as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__GuessNumParams::number`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __GuessNumParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__GuessNumParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| f.ty::<u8>().name("number").type_name("u8")),
                        )
                }
            }
        };
        impl InvocationIo for __GuessNumParams {
            const ROUTE: &'static [u8] = &[
                32u8, 71u8, 117u8, 101u8, 115u8, 115u8, 78u8, 117u8, 109u8,
            ];
            type Params = Self;
            const ASYNC: bool = true;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __IncrParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __IncrParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__IncrParams {})
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
            impl sails_rs::scale_info::TypeInfo for __IncrParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__IncrParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __IncrParams {
            const ROUTE: &'static [u8] = &[16u8, 73u8, 110u8, 99u8, 114u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __LastByteParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __LastByteParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__LastByteParams {})
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
            impl sails_rs::scale_info::TypeInfo for __LastByteParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__LastByteParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __LastByteParams {
            const ROUTE: &'static [u8] = &[
                32u8, 76u8, 97u8, 115u8, 116u8, 66u8, 121u8, 116u8, 101u8,
            ];
            type Params = Self;
            const ASYNC: bool = true;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __MessageParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __MessageParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__MessageParams {})
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
            impl sails_rs::scale_info::TypeInfo for __MessageParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__MessageParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __MessageParams {
            const ROUTE: &'static [u8] = &[
                28u8, 77u8, 101u8, 115u8, 115u8, 97u8, 103u8, 101u8,
            ];
            type Params = Self;
            const ASYNC: bool = true;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __SetNumParams {
            pub(super) number: u8,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __SetNumParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__SetNumParams {
                        number: {
                            let __codec_res_edqy = <u8 as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__SetNumParams::number`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __SetNumParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__SetNumParams",
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| f.ty::<u8>().name("number").type_name("u8")),
                        )
                }
            }
        };
        impl InvocationIo for __SetNumParams {
            const ROUTE: &'static [u8] = &[24u8, 83u8, 101u8, 116u8, 78u8, 117u8, 109u8];
            type Params = Self;
            const ASYNC: bool = true;
        }
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            Add(__AddParams, &'static u32),
            AddByte(__AddByteParams, &'static [u8]),
            GuessNum(__GuessNumParams, Result<&'static str, &'static str>),
            Incr(__IncrParams, &'static ReferenceCount),
            SetNum(__SetNumParams, Result<(), &'static str>),
        }
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
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "Add",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__AddParams>().type_name("__AddParams"))
                                                    .field(|f| f.ty::<&'static u32>().type_name("&'static u32")),
                                            )
                                    },
                                )
                                .variant(
                                    "AddByte",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__AddByteParams>().type_name("__AddByteParams")
                                                    })
                                                    .field(|f| {
                                                        f.ty::<&'static [u8]>().type_name("&'static[u8]")
                                                    }),
                                            )
                                    },
                                )
                                .variant(
                                    "GuessNum",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__GuessNumParams>().type_name("__GuessNumParams")
                                                    })
                                                    .field(|f| {
                                                        f
                                                            .ty::<Result<&'static str, &'static str>>()
                                                            .type_name("Result<&'static str, &'static str>")
                                                    }),
                                            )
                                    },
                                )
                                .variant(
                                    "Incr",
                                    |v| {
                                        v
                                            .index(3usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__IncrParams>().type_name("__IncrParams"))
                                                    .field(|f| {
                                                        f
                                                            .ty::<&'static ReferenceCount>()
                                                            .type_name("&'static ReferenceCount")
                                                    }),
                                            )
                                    },
                                )
                                .variant(
                                    "SetNum",
                                    |v| {
                                        v
                                            .index(4usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__SetNumParams>().type_name("__SetNumParams")
                                                    })
                                                    .field(|f| {
                                                        f
                                                            .ty::<Result<(), &'static str>>()
                                                            .type_name("Result<(), &'static str>")
                                                    }),
                                            )
                                    },
                                ),
                        )
                }
            }
        };
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {
            Baked(__BakedParams, &'static str),
            LastByte(__LastByteParams, Option<&'static u8>),
            Message(__MessageParams, Option<&'static str>),
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
                                "demo::references::reference_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "Baked",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__BakedParams>().type_name("__BakedParams")
                                                    })
                                                    .field(|f| f.ty::<&'static str>().type_name("&'static str")),
                                            )
                                    },
                                )
                                .variant(
                                    "LastByte",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__LastByteParams>().type_name("__LastByteParams")
                                                    })
                                                    .field(|f| {
                                                        f
                                                            .ty::<Option<&'static u8>>()
                                                            .type_name("Option<&'static u8>")
                                                    }),
                                            )
                                    },
                                )
                                .variant(
                                    "Message",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__MessageParams>().type_name("__MessageParams")
                                                    })
                                                    .field(|f| {
                                                        f
                                                            .ty::<Option<&'static str>>()
                                                            .type_name("Option<&'static str>")
                                                    }),
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
                                "demo::references::reference_service_meta",
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
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct ReferenceCount(u32);
    #[automatically_derived]
    impl ::core::fmt::Debug for ReferenceCount {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(
                f,
                "ReferenceCount",
                &&self.0,
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Encode for ReferenceCount {
            fn size_hint(&self) -> usize {
                sails_rs::scale_codec::Encode::size_hint(&&self.0)
            }
            fn encode_to<
                __CodecOutputEdqy: sails_rs::scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                sails_rs::scale_codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(
                &self,
            ) -> sails_rs::scale_codec::alloc::vec::Vec<::core::primitive::u8> {
                sails_rs::scale_codec::Encode::encode(&&self.0)
            }
            fn using_encoded<
                __CodecOutputReturn,
                __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                        &[::core::primitive::u8],
                    ) -> __CodecOutputReturn,
            >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
                sails_rs::scale_codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl sails_rs::scale_codec::EncodeLike for ReferenceCount {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for ReferenceCount {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(
                    ReferenceCount({
                        let __codec_res_edqy = <u32 as sails_rs::scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `ReferenceCount.0`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    }),
                )
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
        impl sails_rs::scale_info::TypeInfo for ReferenceCount {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "ReferenceCount",
                            "demo::references",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        sails_rs::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<u32>().type_name("u32")),
                    )
            }
        }
    };
}
mod this_that {
    use sails_rs::{gstd::debug, prelude::*};
    pub struct MyService(());
    #[automatically_derived]
    impl ::core::default::Default for MyService {
        #[inline]
        fn default() -> MyService {
            MyService(::core::default::Default::default())
        }
    }
    pub struct MyServiceExposure<T> {
        route: &'static [u8],
        inner: T,
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::Exposure
    for MyServiceExposure<T> {
        fn route(&self) -> &'static [u8] {
            self.route
        }
        fn check_asyncness(input: &[u8]) -> Option<bool> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if !T::ASYNC {
                return Some(false);
            }
            if let Ok(is_async) = my_service_meta::__DoThatParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = my_service_meta::__DoThisParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            if let Ok(is_async) = my_service_meta::__NoopParams::check_asyncness(input) {
                return Some(is_async);
            }
            if let Ok(is_async) = my_service_meta::__ThatParams::check_asyncness(input) {
                return Some(is_async);
            }
            if let Ok(is_async) = my_service_meta::__ThisParams::check_asyncness(input) {
                return Some(is_async);
            }
            None
        }
    }
    impl<T> core::ops::Deref for MyServiceExposure<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T> core::ops::DerefMut for MyServiceExposure<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl MyServiceExposure<MyService> {
        #[allow(unused_variables)]
        pub async fn do_this(
            &mut self,
            p1: u32,
            p2: String,
            p3: (Option<H160>, NonZeroU8),
            p4: TupleStruct,
        ) -> (String, u32) {
            ::gcore::ext::stack_debug(
                format_args!(
                    "Handling \'do_this\': {0}, {1}, {2:?}, {3:?}",
                    p1,
                    p2,
                    p3,
                    p4,
                ),
            );
            (p2, p1)
        }
        pub fn do_that(
            &mut self,
            param: DoThatParam,
        ) -> Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)> {
            ::gcore::ext::stack_debug(
                format_args!("Handling \'do_that\': {0:?}", param),
            );
            let p3 = match param.p3 {
                ManyVariants::One => ManyVariantsReply::One,
                ManyVariants::Two(_) => ManyVariantsReply::Two,
                ManyVariants::Three(_) => ManyVariantsReply::Three,
                ManyVariants::Four { a: _, b: _ } => ManyVariantsReply::Four,
                ManyVariants::Five(_, _) => ManyVariantsReply::Five,
                ManyVariants::Six(_) => ManyVariantsReply::Six,
            };
            Ok((param.p2, param.p1, p3))
        }
        pub fn noop(&mut self) {
            ::gcore::ext::stack_debug(format_args!("Handling \'noop\'"));
        }
        pub fn this(&self) -> u32 {
            ::gcore::ext::stack_debug(format_args!("Handling \'this\'"));
            42
        }
        pub fn that(&self) -> Result<String, String> {
            ::gcore::ext::stack_debug(format_args!("Handling \'that\'"));
            Ok("Forty two".into())
        }
        pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
            <Self as sails_rs::gstd::services::Exposure>::check_asyncness(input)
        }
        pub fn try_handle(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = my_service_meta::__DoThatParams::decode_params(input) {
                let result = self.do_that(request.param);
                let value = 0u128;
                if !my_service_meta::__DoThatParams::is_empty_tuple::<
                    Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)>,
                >() {
                    my_service_meta::__DoThatParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = my_service_meta::__NoopParams::decode_params(input) {
                let result = self.noop();
                let value = 0u128;
                if !my_service_meta::__NoopParams::is_empty_tuple::<()>() {
                    my_service_meta::__NoopParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = my_service_meta::__ThatParams::decode_params(input) {
                let result = self.that();
                let value = 0u128;
                if !my_service_meta::__ThatParams::is_empty_tuple::<
                    Result<String, String>,
                >() {
                    my_service_meta::__ThatParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            if let Ok(request) = my_service_meta::__ThisParams::decode_params(input) {
                let result = self.this();
                let value = 0u128;
                if !my_service_meta::__ThisParams::is_empty_tuple::<u32>() {
                    my_service_meta::__ThisParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
        pub async fn try_handle_async(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = my_service_meta::__DoThisParams::decode_params(input) {
                let result = self
                    .do_this(request.p1, request.p2, request.p3, request.p4)
                    .await;
                let value = 0u128;
                if !my_service_meta::__DoThisParams::is_empty_tuple::<(String, u32)>() {
                    my_service_meta::__DoThisParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
    }
    impl sails_rs::gstd::services::Service for MyService {
        type Exposure = MyServiceExposure<Self>;
        fn expose(self, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                route,
                inner: self,
            }
        }
    }
    impl sails_rs::meta::ServiceMeta for MyService {
        type CommandsMeta = my_service_meta::CommandsMeta;
        type QueriesMeta = my_service_meta::QueriesMeta;
        type EventsMeta = my_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
        const ASYNC: bool = true;
    }
    mod my_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo};
        use sails_rs::gstd::InvocationIo;
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __DoThatParams {
            pub(super) param: DoThatParam,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __DoThatParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__DoThatParams {
                        param: {
                            let __codec_res_edqy = <DoThatParam as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__DoThatParams::param`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __DoThatParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__DoThatParams",
                                "demo::this_that::my_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| {
                                    f.ty::<DoThatParam>().name("param").type_name("DoThatParam")
                                }),
                        )
                }
            }
        };
        impl InvocationIo for __DoThatParams {
            const ROUTE: &'static [u8] = &[24u8, 68u8, 111u8, 84u8, 104u8, 97u8, 116u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __DoThisParams {
            pub(super) p1: u32,
            pub(super) p2: String,
            pub(super) p3: (Option<H160>, NonZeroU8),
            pub(super) p4: TupleStruct,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __DoThisParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__DoThisParams {
                        p1: {
                            let __codec_res_edqy = <u32 as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__DoThisParams::p1`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                        p2: {
                            let __codec_res_edqy = <String as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__DoThisParams::p2`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                        p3: {
                            let __codec_res_edqy = <(
                                Option<H160>,
                                NonZeroU8,
                            ) as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__DoThisParams::p3`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                        p4: {
                            let __codec_res_edqy = <TupleStruct as sails_rs::scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `__DoThisParams::p4`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
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
            impl sails_rs::scale_info::TypeInfo for __DoThisParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__DoThisParams",
                                "demo::this_that::my_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(
                            sails_rs::scale_info::build::Fields::named()
                                .field(|f| f.ty::<u32>().name("p1").type_name("u32"))
                                .field(|f| f.ty::<String>().name("p2").type_name("String"))
                                .field(|f| {
                                    f
                                        .ty::<(Option<H160>, NonZeroU8)>()
                                        .name("p3")
                                        .type_name("(Option<H160>, NonZeroU8)")
                                })
                                .field(|f| {
                                    f.ty::<TupleStruct>().name("p4").type_name("TupleStruct")
                                }),
                        )
                }
            }
        };
        impl InvocationIo for __DoThisParams {
            const ROUTE: &'static [u8] = &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8];
            type Params = Self;
            const ASYNC: bool = true;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __NoopParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __NoopParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__NoopParams {})
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
            impl sails_rs::scale_info::TypeInfo for __NoopParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__NoopParams",
                                "demo::this_that::my_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __NoopParams {
            const ROUTE: &'static [u8] = &[16u8, 78u8, 111u8, 111u8, 112u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __ThatParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __ThatParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__ThatParams {})
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
            impl sails_rs::scale_info::TypeInfo for __ThatParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__ThatParams",
                                "demo::this_that::my_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __ThatParams {
            const ROUTE: &'static [u8] = &[16u8, 84u8, 104u8, 97u8, 116u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __ThisParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __ThisParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__ThisParams {})
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
            impl sails_rs::scale_info::TypeInfo for __ThisParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__ThisParams",
                                "demo::this_that::my_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __ThisParams {
            const ROUTE: &'static [u8] = &[16u8, 84u8, 104u8, 105u8, 115u8];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            DoThat(
                __DoThatParams,
                Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)>,
            ),
            DoThis(__DoThisParams, (String, u32)),
            Noop(__NoopParams, ()),
        }
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
                                "demo::this_that::my_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "DoThat",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__DoThatParams>().type_name("__DoThatParams")
                                                    })
                                                    .field(|f| {
                                                        f
                                                            .ty::<
                                                                Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)>,
                                                            >()
                                                            .type_name(
                                                                "Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)>",
                                                            )
                                                    }),
                                            )
                                    },
                                )
                                .variant(
                                    "DoThis",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f.ty::<__DoThisParams>().type_name("__DoThisParams")
                                                    })
                                                    .field(|f| {
                                                        f.ty::<(String, u32)>().type_name("(String, u32)")
                                                    }),
                                            )
                                    },
                                )
                                .variant(
                                    "Noop",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__NoopParams>().type_name("__NoopParams"))
                                                    .field(|f| f.ty::<()>().type_name("()")),
                                            )
                                    },
                                ),
                        )
                }
            }
        };
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {
            That(__ThatParams, Result<String, String>),
            This(__ThisParams, u32),
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
                                "demo::this_that::my_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "That",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__ThatParams>().type_name("__ThatParams"))
                                                    .field(|f| {
                                                        f
                                                            .ty::<Result<String, String>>()
                                                            .type_name("Result<String, String>")
                                                    }),
                                            )
                                    },
                                )
                                .variant(
                                    "This",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| f.ty::<__ThisParams>().type_name("__ThisParams"))
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
                                "demo::this_that::my_service_meta",
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
    #[allow(dead_code)]
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct TupleStruct(bool);
    #[automatically_derived]
    #[allow(dead_code)]
    impl ::core::fmt::Debug for TupleStruct {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "TupleStruct", &&self.0)
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[allow(dead_code)]
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for TupleStruct {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(
                    TupleStruct({
                        let __codec_res_edqy = <bool as sails_rs::scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `TupleStruct.0`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    }),
                )
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
        impl sails_rs::scale_info::TypeInfo for TupleStruct {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "TupleStruct",
                            "demo::this_that",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        sails_rs::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<bool>().type_name("bool")),
                    )
            }
        }
    };
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct DoThatParam {
        pub p1: NonZeroU32,
        pub p2: ActorId,
        pub p3: ManyVariants,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DoThatParam {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "DoThatParam",
                "p1",
                &self.p1,
                "p2",
                &self.p2,
                "p3",
                &&self.p3,
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for DoThatParam {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(DoThatParam {
                    p1: {
                        let __codec_res_edqy = <NonZeroU32 as sails_rs::scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `DoThatParam::p1`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    p2: {
                        let __codec_res_edqy = <ActorId as sails_rs::scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `DoThatParam::p2`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    p3: {
                        let __codec_res_edqy = <ManyVariants as sails_rs::scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `DoThatParam::p3`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                })
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
        impl sails_rs::scale_info::TypeInfo for DoThatParam {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "DoThatParam",
                            "demo::this_that",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        sails_rs::scale_info::build::Fields::named()
                            .field(|f| {
                                f.ty::<NonZeroU32>().name("p1").type_name("NonZeroU32")
                            })
                            .field(|f| f.ty::<ActorId>().name("p2").type_name("ActorId"))
                            .field(|f| {
                                f.ty::<ManyVariants>().name("p3").type_name("ManyVariants")
                            }),
                    )
            }
        }
    };
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum ManyVariants {
        One,
        Two(u32),
        Three(Option<U256>),
        Four { a: u32, b: Option<u16> },
        Five(String, H256),
        Six((u32,)),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ManyVariants {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                ManyVariants::One => ::core::fmt::Formatter::write_str(f, "One"),
                ManyVariants::Two(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Two",
                        &__self_0,
                    )
                }
                ManyVariants::Three(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Three",
                        &__self_0,
                    )
                }
                ManyVariants::Four { a: __self_0, b: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "Four",
                        "a",
                        __self_0,
                        "b",
                        &__self_1,
                    )
                }
                ManyVariants::Five(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "Five",
                        __self_0,
                        &__self_1,
                    )
                }
                ManyVariants::Six(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Six",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for ManyVariants {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                #[automatically_derived]
                const _: () = {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    const indices: [(usize, &'static str); 6usize] = [
                        ((0usize) as ::core::primitive::usize, "One"),
                        ((1usize) as ::core::primitive::usize, "Two"),
                        ((2usize) as ::core::primitive::usize, "Three"),
                        ((3usize) as ::core::primitive::usize, "Four"),
                        ((4usize) as ::core::primitive::usize, "Five"),
                        ((5usize) as ::core::primitive::usize, "Six"),
                    ];
                    const fn search_for_invalid_index(
                        array: &[(usize, &'static str); 6usize],
                    ) -> (bool, usize) {
                        let mut i = 0;
                        while i < 6usize {
                            if array[i].0 > 255 {
                                return (true, i);
                            }
                            i += 1;
                        }
                        (false, 0)
                    }
                    const INVALID_INDEX: (bool, usize) = search_for_invalid_index(
                        &indices,
                    );
                    if INVALID_INDEX.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper("Found variant `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].1,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` with invalid index: `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].0,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Max supported index is 255.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                    const fn duplicate_info(
                        array: &[(usize, &'static str); 6usize],
                    ) -> (bool, usize, usize) {
                        let len = 6usize;
                        let mut i = 0usize;
                        while i < len {
                            let mut j = i + 1;
                            while j < len {
                                if array[i].0 == array[j].0 {
                                    return (true, i, j);
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        (false, 0, 0)
                    }
                    const DUP_INFO: (bool, usize, usize) = duplicate_info(&indices);
                    if DUP_INFO.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "Found variants that have duplicate indexes. Both `",
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` and `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.2].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` have the index `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].0)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Use different indexes for each variant.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                };
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e
                            .chain(
                                "Could not decode `ManyVariants`, failed to read variant byte",
                            )
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (0usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(ManyVariants::One)
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (1usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                ManyVariants::Two({
                                    let __codec_res_edqy = <u32 as sails_rs::scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `ManyVariants::Two.0`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                }),
                            )
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (2usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                ManyVariants::Three({
                                    let __codec_res_edqy = <Option<
                                        U256,
                                    > as sails_rs::scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `ManyVariants::Three.0`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                }),
                            )
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (3usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(ManyVariants::Four {
                                a: {
                                    let __codec_res_edqy = <u32 as sails_rs::scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `ManyVariants::Four::a`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                },
                                b: {
                                    let __codec_res_edqy = <Option<
                                        u16,
                                    > as sails_rs::scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `ManyVariants::Four::b`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                },
                            })
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (4usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                ManyVariants::Five(
                                    {
                                        let __codec_res_edqy = <String as sails_rs::scale_codec::Decode>::decode(
                                            __codec_input_edqy,
                                        );
                                        match __codec_res_edqy {
                                            ::core::result::Result::Err(e) => {
                                                return ::core::result::Result::Err(
                                                    e.chain("Could not decode `ManyVariants::Five.0`"),
                                                );
                                            }
                                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                                __codec_res_edqy
                                            }
                                        }
                                    },
                                    {
                                        let __codec_res_edqy = <H256 as sails_rs::scale_codec::Decode>::decode(
                                            __codec_input_edqy,
                                        );
                                        match __codec_res_edqy {
                                            ::core::result::Result::Err(e) => {
                                                return ::core::result::Result::Err(
                                                    e.chain("Could not decode `ManyVariants::Five.1`"),
                                                );
                                            }
                                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                                __codec_res_edqy
                                            }
                                        }
                                    },
                                ),
                            )
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (5usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                ManyVariants::Six({
                                    let __codec_res_edqy = <(
                                        u32,
                                    ) as sails_rs::scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `ManyVariants::Six.0`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                }),
                            )
                        })();
                    }
                    _ => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Err(
                                <_ as ::core::convert::Into<
                                    _,
                                >>::into(
                                    "Could not decode `ManyVariants`, variant doesn't exist",
                                ),
                            )
                        })();
                    }
                }
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
        impl sails_rs::scale_info::TypeInfo for ManyVariants {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "ManyVariants",
                            "demo::this_that",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant("One", |v| v.index(0usize as ::core::primitive::u8))
                            .variant(
                                "Two",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<u32>().type_name("u32")),
                                        )
                                },
                            )
                            .variant(
                                "Three",
                                |v| {
                                    v
                                        .index(2usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<Option<U256>>().type_name("Option<U256>")),
                                        )
                                },
                            )
                            .variant(
                                "Four",
                                |v| {
                                    v
                                        .index(3usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::named()
                                                .field(|f| f.ty::<u32>().name("a").type_name("u32"))
                                                .field(|f| {
                                                    f.ty::<Option<u16>>().name("b").type_name("Option<u16>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "Five",
                                |v| {
                                    v
                                        .index(4usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<String>().type_name("String"))
                                                .field(|f| f.ty::<H256>().type_name("H256")),
                                        )
                                },
                            )
                            .variant(
                                "Six",
                                |v| {
                                    v
                                        .index(5usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<(u32,)>().type_name("(u32,)")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum ManyVariantsReply {
        One,
        Two,
        Three,
        Four,
        Five,
        Six,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ManyVariantsReply {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    ManyVariantsReply::One => "One",
                    ManyVariantsReply::Two => "Two",
                    ManyVariantsReply::Three => "Three",
                    ManyVariantsReply::Four => "Four",
                    ManyVariantsReply::Five => "Five",
                    ManyVariantsReply::Six => "Six",
                },
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Encode for ManyVariantsReply {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        ManyVariantsReply::One => 0_usize,
                        ManyVariantsReply::Two => 0_usize,
                        ManyVariantsReply::Three => 0_usize,
                        ManyVariantsReply::Four => 0_usize,
                        ManyVariantsReply::Five => 0_usize,
                        ManyVariantsReply::Six => 0_usize,
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: sails_rs::scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                #[automatically_derived]
                const _: () = {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    const indices: [(usize, &'static str); 6usize] = [
                        ((0usize) as ::core::primitive::usize, "One"),
                        ((1usize) as ::core::primitive::usize, "Two"),
                        ((2usize) as ::core::primitive::usize, "Three"),
                        ((3usize) as ::core::primitive::usize, "Four"),
                        ((4usize) as ::core::primitive::usize, "Five"),
                        ((5usize) as ::core::primitive::usize, "Six"),
                    ];
                    const fn search_for_invalid_index(
                        array: &[(usize, &'static str); 6usize],
                    ) -> (bool, usize) {
                        let mut i = 0;
                        while i < 6usize {
                            if array[i].0 > 255 {
                                return (true, i);
                            }
                            i += 1;
                        }
                        (false, 0)
                    }
                    const INVALID_INDEX: (bool, usize) = search_for_invalid_index(
                        &indices,
                    );
                    if INVALID_INDEX.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper("Found variant `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].1,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` with invalid index: `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].0,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Max supported index is 255.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                    const fn duplicate_info(
                        array: &[(usize, &'static str); 6usize],
                    ) -> (bool, usize, usize) {
                        let len = 6usize;
                        let mut i = 0usize;
                        while i < len {
                            let mut j = i + 1;
                            while j < len {
                                if array[i].0 == array[j].0 {
                                    return (true, i, j);
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        (false, 0, 0)
                    }
                    const DUP_INFO: (bool, usize, usize) = duplicate_info(&indices);
                    if DUP_INFO.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "Found variants that have duplicate indexes. Both `",
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` and `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.2].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` have the index `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].0)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Use different indexes for each variant.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                };
                match *self {
                    ManyVariantsReply::One => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((0usize) as ::core::primitive::u8);
                    }
                    ManyVariantsReply::Two => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((1usize) as ::core::primitive::u8);
                    }
                    ManyVariantsReply::Three => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((2usize) as ::core::primitive::u8);
                    }
                    ManyVariantsReply::Four => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((3usize) as ::core::primitive::u8);
                    }
                    ManyVariantsReply::Five => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((4usize) as ::core::primitive::u8);
                    }
                    ManyVariantsReply::Six => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((5usize) as ::core::primitive::u8);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl sails_rs::scale_codec::EncodeLike for ManyVariantsReply {}
    };
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for ManyVariantsReply {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "ManyVariantsReply",
                            "demo::this_that",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant("One", |v| v.index(0usize as ::core::primitive::u8))
                            .variant("Two", |v| v.index(1usize as ::core::primitive::u8))
                            .variant(
                                "Three",
                                |v| v.index(2usize as ::core::primitive::u8),
                            )
                            .variant(
                                "Four",
                                |v| v.index(3usize as ::core::primitive::u8),
                            )
                            .variant(
                                "Five",
                                |v| v.index(4usize as ::core::primitive::u8),
                            )
                            .variant("Six", |v| v.index(5usize as ::core::primitive::u8)),
                    )
            }
        }
    };
}
mod value_fee {
    use sails_rs::prelude::*;
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum FeeEvents {
        Withheld(ValueUnit),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for FeeEvents {
        #[inline]
        fn clone(&self) -> FeeEvents {
            match self {
                FeeEvents::Withheld(__self_0) => {
                    FeeEvents::Withheld(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for FeeEvents {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                FeeEvents::Withheld(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Withheld",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for FeeEvents {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for FeeEvents {
        #[inline]
        fn eq(&self, other: &FeeEvents) -> bool {
            match (self, other) {
                (FeeEvents::Withheld(__self_0), FeeEvents::Withheld(__arg1_0)) => {
                    __self_0 == __arg1_0
                }
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Encode for FeeEvents {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        FeeEvents::Withheld(ref aa) => {
                            0_usize
                                .saturating_add(
                                    sails_rs::scale_codec::Encode::size_hint(aa),
                                )
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: sails_rs::scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                #[automatically_derived]
                const _: () = {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    const indices: [(usize, &'static str); 1usize] = [
                        ((0usize) as ::core::primitive::usize, "Withheld"),
                    ];
                    const fn search_for_invalid_index(
                        array: &[(usize, &'static str); 1usize],
                    ) -> (bool, usize) {
                        let mut i = 0;
                        while i < 1usize {
                            if array[i].0 > 255 {
                                return (true, i);
                            }
                            i += 1;
                        }
                        (false, 0)
                    }
                    const INVALID_INDEX: (bool, usize) = search_for_invalid_index(
                        &indices,
                    );
                    if INVALID_INDEX.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper("Found variant `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].1,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` with invalid index: `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].0,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Max supported index is 255.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                    const fn duplicate_info(
                        array: &[(usize, &'static str); 1usize],
                    ) -> (bool, usize, usize) {
                        let len = 1usize;
                        let mut i = 0usize;
                        while i < len {
                            let mut j = i + 1;
                            while j < len {
                                if array[i].0 == array[j].0 {
                                    return (true, i, j);
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        (false, 0, 0)
                    }
                    const DUP_INFO: (bool, usize, usize) = duplicate_info(&indices);
                    if DUP_INFO.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "Found variants that have duplicate indexes. Both `",
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` and `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.2].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` have the index `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].0)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Use different indexes for each variant.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            #[cold]
                            #[track_caller]
                            #[inline(never)]
                            #[rustc_const_panic_str]
                            #[rustc_do_not_const_check]
                            const fn panic_cold_display<T: ::core::fmt::Display>(
                                arg: &T,
                            ) -> ! {
                                ::core::panicking::panic_display(arg)
                            }
                            panic_cold_display(&msg);
                        };
                    }
                };
                match *self {
                    FeeEvents::Withheld(ref aa) => {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_dest_edqy.push_byte((0usize) as ::core::primitive::u8);
                        sails_rs::scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl sails_rs::scale_codec::EncodeLike for FeeEvents {}
    };
    #[allow(
        non_upper_case_globals,
        deprecated,
        unused_attributes,
        unused_qualifications
    )]
    const _: () = {
        impl sails_rs::scale_info::TypeInfo for FeeEvents {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "FeeEvents",
                            "demo::value_fee",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant(
                                "Withheld",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<ValueUnit>().type_name("ValueUnit")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    impl sails_rs::SailsEvent for FeeEvents {
        fn encoded_event_name(&self) -> &'static [u8] {
            match self {
                FeeEvents::Withheld(..) => {
                    &[32u8, 87u8, 105u8, 116u8, 104u8, 104u8, 101u8, 108u8, 100u8]
                }
            }
        }
        fn skip_bytes() -> usize {
            1
        }
    }
    pub struct FeeService {
        fee: ValueUnit,
    }
    impl FeeService {
        pub fn new(fee: ValueUnit) -> Self {
            Self { fee }
        }
    }
    pub struct FeeServiceExposure<T> {
        route: &'static [u8],
        inner: T,
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::Exposure
    for FeeServiceExposure<T> {
        fn route(&self) -> &'static [u8] {
            self.route
        }
        fn check_asyncness(input: &[u8]) -> Option<bool> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if !T::ASYNC {
                return Some(false);
            }
            if let Ok(is_async) = fee_service_meta::__DoSomethingAndTakeFeeParams::check_asyncness(
                input,
            ) {
                return Some(is_async);
            }
            None
        }
    }
    impl<T: sails_rs::meta::ServiceMeta> sails_rs::gstd::services::ExposureWithEvents
    for FeeServiceExposure<T> {
        type Events = FeeEvents;
    }
    impl<T> core::ops::Deref for FeeServiceExposure<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T> core::ops::DerefMut for FeeServiceExposure<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl FeeServiceExposure<FeeService> {
        /// Return flag if fee taken and remain value,
        /// using special type `CommandReply<T>`
        pub fn do_something_and_take_fee(&mut self) -> CommandReply<bool> {
            let value = Syscall::message_value();
            if value == 0 {
                return false.into();
            }
            if value < self.fee {
                {
                    ::core::panicking::panic_fmt(format_args!("Not enough value"));
                };
            }
            self.emit_event(FeeEvents::Withheld(self.fee)).unwrap();
            let to_return = value - self.fee;
            if to_return < Syscall::env_vars().existential_deposit {
                true.into()
            } else {
                CommandReply::new(true).with_value(to_return)
            }
        }
        pub fn check_asyncness(&self, input: &[u8]) -> Option<bool> {
            <Self as sails_rs::gstd::services::Exposure>::check_asyncness(input)
        }
        pub fn try_handle(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            if let Ok(request) = fee_service_meta::__DoSomethingAndTakeFeeParams::decode_params(
                input,
            ) {
                let command_reply: CommandReply<bool> = self
                    .do_something_and_take_fee()
                    .into();
                let (result, value) = command_reply.to_tuple();
                if !fee_service_meta::__DoSomethingAndTakeFeeParams::is_empty_tuple::<
                    bool,
                >() {
                    fee_service_meta::__DoSomethingAndTakeFeeParams::with_optimized_encode(
                        &result,
                        self.route().as_ref(),
                        |encoded_result| result_handler(encoded_result, value),
                    );
                }
                return Some(());
            }
            None
        }
        pub async fn try_handle_async(
            mut self,
            input: &[u8],
            result_handler: fn(&[u8], u128),
        ) -> Option<()> {
            use sails_rs::gstd::InvocationIo;
            use sails_rs::gstd::services::{Service, Exposure};
            None
        }
        pub fn emit_event(&self, event: FeeEvents) -> sails_rs::errors::Result<()> {
            use sails_rs::gstd::services::ExposureWithEvents;
            self.emitter().emit_event(event)
        }
    }
    impl sails_rs::gstd::services::Service for FeeService {
        type Exposure = FeeServiceExposure<Self>;
        fn expose(self, route: &'static [u8]) -> Self::Exposure {
            Self::Exposure {
                route,
                inner: self,
            }
        }
    }
    impl sails_rs::meta::ServiceMeta for FeeService {
        type CommandsMeta = fee_service_meta::CommandsMeta;
        type QueriesMeta = fee_service_meta::QueriesMeta;
        type EventsMeta = fee_service_meta::EventsMeta;
        const BASE_SERVICES: &'static [sails_rs::meta::AnyServiceMetaFn] = &[];
        const ASYNC: bool = false;
    }
    mod fee_service_meta {
        use super::*;
        use sails_rs::{Decode, TypeInfo};
        use sails_rs::gstd::InvocationIo;
        #[codec(crate = sails_rs::scale_codec)]
        #[scale_info(crate = sails_rs::scale_info)]
        pub struct __DoSomethingAndTakeFeeParams {}
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl sails_rs::scale_codec::Decode for __DoSomethingAndTakeFeeParams {
                fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                    ::core::result::Result::Ok(__DoSomethingAndTakeFeeParams {})
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
            impl sails_rs::scale_info::TypeInfo for __DoSomethingAndTakeFeeParams {
                type Identity = Self;
                fn type_info() -> sails_rs::scale_info::Type {
                    sails_rs::scale_info::Type::builder()
                        .path(
                            sails_rs::scale_info::Path::new_with_replace(
                                "__DoSomethingAndTakeFeeParams",
                                "demo::value_fee::fee_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .composite(sails_rs::scale_info::build::Fields::named())
                }
            }
        };
        impl InvocationIo for __DoSomethingAndTakeFeeParams {
            const ROUTE: &'static [u8] = &[
                84u8, 68u8, 111u8, 83u8, 111u8, 109u8, 101u8, 116u8, 104u8, 105u8, 110u8,
                103u8, 65u8, 110u8, 100u8, 84u8, 97u8, 107u8, 101u8, 70u8, 101u8, 101u8,
            ];
            type Params = Self;
            const ASYNC: bool = false;
        }
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum CommandsMeta {
            /// Return flag if fee taken and remain value,
            /// using special type `CommandReply<T>`
            DoSomethingAndTakeFee(__DoSomethingAndTakeFeeParams, bool),
        }
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
                                "demo::value_fee::fee_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            sails_rs::scale_info::build::Variants::new()
                                .variant(
                                    "DoSomethingAndTakeFee",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .fields(
                                                sails_rs::scale_info::build::Fields::unnamed()
                                                    .field(|f| {
                                                        f
                                                            .ty::<__DoSomethingAndTakeFeeParams>()
                                                            .type_name("__DoSomethingAndTakeFeeParams")
                                                    })
                                                    .field(|f| f.ty::<bool>().type_name("bool")),
                                            )
                                            .docs(
                                                &[
                                                    "Return flag if fee taken and remain value,",
                                                    "using special type `CommandReply<T>`",
                                                ],
                                            )
                                    },
                                ),
                        )
                }
            }
        };
        #[scale_info(crate = sails_rs::scale_info)]
        pub enum QueriesMeta {}
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
                                "demo::value_fee::fee_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(sails_rs::scale_info::build::Variants::new())
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
                                "demo::value_fee::fee_service_meta",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(sails_rs::scale_info::build::Variants::new())
                }
            }
        };
        pub type EventsMeta = FeeEvents;
    }
}
static mut DOG_DATA: Option<RefCell<walker::WalkerData>> = None;
#[allow(static_mut_refs)]
fn dog_data() -> &'static RefCell<walker::WalkerData> {
    unsafe {
        DOG_DATA
            .as_ref()
            .unwrap_or_else(|| {
                ::core::panicking::panic_fmt(
                    format_args!("`Dog` data should be initialized first"),
                );
            })
    }
}
pub struct DemoProgram {
    counter_data: RefCell<counter::CounterData>,
    ref_data: u8,
}
impl DemoProgram {
    #[allow(clippy::should_implement_trait)]
    /// Program constructor (called once at the very beginning of the program lifetime)
    pub fn default() -> Self {
        unsafe {
            DOG_DATA = Some(
                RefCell::new(
                    walker::WalkerData::new(Default::default(), Default::default()),
                ),
            );
        }
        Self {
            counter_data: RefCell::new(counter::CounterData::new(Default::default())),
            ref_data: 42,
        }
    }
    /// Another program constructor (called once at the very beginning of the program lifetime)
    pub fn new(
        counter: Option<u32>,
        dog_position: Option<(i32, i32)>,
    ) -> Result<Self, String> {
        unsafe {
            let dog_position = dog_position.unwrap_or_default();
            DOG_DATA = Some(
                RefCell::new(walker::WalkerData::new(dog_position.0, dog_position.1)),
            );
        }
        Ok(Self {
            counter_data: RefCell::new(
                counter::CounterData::new(counter.unwrap_or_default()),
            ),
            ref_data: 42,
        })
    }
    fn __ping(&self) -> Result<ping::PingService, String> {
        Ok(ping::PingService::default())
    }
    fn __counter(&self) -> counter::CounterService<'_> {
        counter::CounterService::new(&self.counter_data)
    }
    fn __dog(&self) -> dog::DogService {
        dog::DogService::new(walker::WalkerService::new(dog_data()))
    }
    fn __references(&mut self) -> references::ReferenceService<'_> {
        references::ReferenceService::new(&mut self.ref_data, "demo")
    }
    fn __this_that(&self) -> this_that::MyService {
        this_that::MyService::default()
    }
    fn __value_fee(&self) -> value_fee::FeeService {
        value_fee::FeeService::new(10_000_000_000_000)
    }
    pub fn ping(
        &self,
    ) -> <ping::PingService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__ping().unwrap();
        let exposure = <ping::PingService as sails_rs::gstd::services::Service>::expose(
            service,
            __ROUTE_PINGPONG.as_ref(),
        );
        exposure
    }
    pub fn counter(
        &self,
    ) -> <counter::CounterService<'_> as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__counter();
        let exposure = <counter::CounterService<
            '_,
        > as sails_rs::gstd::services::Service>::expose(
            service,
            __ROUTE_COUNTER.as_ref(),
        );
        exposure
    }
    pub fn dog(
        &self,
    ) -> <dog::DogService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__dog();
        let exposure = <dog::DogService as sails_rs::gstd::services::Service>::expose(
            service,
            __ROUTE_DOG.as_ref(),
        );
        exposure
    }
    pub fn references(
        &mut self,
    ) -> <references::ReferenceService<
        '_,
    > as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__references();
        let exposure = <references::ReferenceService<
            '_,
        > as sails_rs::gstd::services::Service>::expose(
            service,
            __ROUTE_REFERENCES.as_ref(),
        );
        exposure
    }
    pub fn this_that(
        &self,
    ) -> <this_that::MyService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__this_that();
        let exposure = <this_that::MyService as sails_rs::gstd::services::Service>::expose(
            service,
            __ROUTE_THISTHAT.as_ref(),
        );
        exposure
    }
    pub fn value_fee(
        &self,
    ) -> <value_fee::FeeService as sails_rs::gstd::services::Service>::Exposure {
        let service = self.__value_fee();
        let exposure = <value_fee::FeeService as sails_rs::gstd::services::Service>::expose(
            service,
            __ROUTE_VALUEFEE.as_ref(),
        );
        exposure
    }
}
const __ROUTE_PINGPONG: [u8; 9usize] = [
    32u8, 80u8, 105u8, 110u8, 103u8, 80u8, 111u8, 110u8, 103u8,
];
const __ROUTE_COUNTER: [u8; 8usize] = [
    28u8, 67u8, 111u8, 117u8, 110u8, 116u8, 101u8, 114u8,
];
const __ROUTE_DOG: [u8; 4usize] = [12u8, 68u8, 111u8, 103u8];
const __ROUTE_REFERENCES: [u8; 11usize] = [
    40u8, 82u8, 101u8, 102u8, 101u8, 114u8, 101u8, 110u8, 99u8, 101u8, 115u8,
];
const __ROUTE_THISTHAT: [u8; 9usize] = [
    32u8, 84u8, 104u8, 105u8, 115u8, 84u8, 104u8, 97u8, 116u8,
];
const __ROUTE_VALUEFEE: [u8; 9usize] = [
    32u8, 86u8, 97u8, 108u8, 117u8, 101u8, 70u8, 101u8, 101u8,
];
impl sails_rs::meta::ProgramMeta for DemoProgram {
    type ConstructorsMeta = meta_in_program::ConstructorsMeta;
    const SERVICES: &'static [(&'static str, sails_rs::meta::AnyServiceMetaFn)] = &[
        ("PingPong", sails_rs::meta::AnyServiceMeta::new::<ping::PingService>),
        ("Counter", sails_rs::meta::AnyServiceMeta::new::<counter::CounterService<'_>>),
        ("Dog", sails_rs::meta::AnyServiceMeta::new::<dog::DogService>),
        (
            "References",
            sails_rs::meta::AnyServiceMeta::new::<references::ReferenceService<'_>>,
        ),
        ("ThisThat", sails_rs::meta::AnyServiceMeta::new::<this_that::MyService>),
        ("ValueFee", sails_rs::meta::AnyServiceMeta::new::<value_fee::FeeService>),
    ];
    const ASYNC: bool = <ping::PingService as sails_rs::meta::ServiceMeta>::ASYNC
        || <counter::CounterService<'_> as sails_rs::meta::ServiceMeta>::ASYNC
        || <dog::DogService as sails_rs::meta::ServiceMeta>::ASYNC
        || <references::ReferenceService<'_> as sails_rs::meta::ServiceMeta>::ASYNC
        || <this_that::MyService as sails_rs::meta::ServiceMeta>::ASYNC
        || <value_fee::FeeService as sails_rs::meta::ServiceMeta>::ASYNC;
}
mod meta_in_program {
    use super::*;
    use sails_rs::gstd::InvocationIo;
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __DefaultParams {}
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for __DefaultParams {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(__DefaultParams {})
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
        impl sails_rs::scale_info::TypeInfo for __DefaultParams {
            type Identity = Self;
            fn type_info() -> sails_rs::scale_info::Type {
                sails_rs::scale_info::Type::builder()
                    .path(
                        sails_rs::scale_info::Path::new_with_replace(
                            "__DefaultParams",
                            "demo::meta_in_program",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(sails_rs::scale_info::build::Fields::named())
            }
        }
    };
    impl InvocationIo for __DefaultParams {
        const ROUTE: &'static [u8] = &[
            28u8, 68u8, 101u8, 102u8, 97u8, 117u8, 108u8, 116u8,
        ];
        type Params = Self;
        const ASYNC: bool = false;
    }
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct __NewParams {
        pub(super) counter: Option<u32>,
        pub(super) dog_position: Option<(i32, i32)>,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl sails_rs::scale_codec::Decode for __NewParams {
            fn decode<__CodecInputEdqy: sails_rs::scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, sails_rs::scale_codec::Error> {
                ::core::result::Result::Ok(__NewParams {
                    counter: {
                        let __codec_res_edqy = <Option<
                            u32,
                        > as sails_rs::scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `__NewParams::counter`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    dog_position: {
                        let __codec_res_edqy = <Option<
                            (i32, i32),
                        > as sails_rs::scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `__NewParams::dog_position`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                })
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
                            "demo::meta_in_program",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        sails_rs::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<Option<u32>>()
                                    .name("counter")
                                    .type_name("Option<u32>")
                            })
                            .field(|f| {
                                f
                                    .ty::<Option<(i32, i32)>>()
                                    .name("dog_position")
                                    .type_name("Option<(i32, i32)>")
                            }),
                    )
            }
        }
    };
    impl InvocationIo for __NewParams {
        const ROUTE: &'static [u8] = &[12u8, 78u8, 101u8, 119u8];
        type Params = Self;
        const ASYNC: bool = false;
    }
    #[scale_info(crate = sails_rs::scale_info)]
    pub enum ConstructorsMeta {
        /// Program constructor (called once at the very beginning of the program lifetime)
        Default(__DefaultParams),
        /// Another program constructor (called once at the very beginning of the program lifetime)
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
                            "demo::meta_in_program",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        sails_rs::scale_info::build::Variants::new()
                            .variant(
                                "Default",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f.ty::<__DefaultParams>().type_name("__DefaultParams")
                                                }),
                                        )
                                        .docs(
                                            &[
                                                "Program constructor (called once at the very beginning of the program lifetime)",
                                            ],
                                        )
                                },
                            )
                            .variant(
                                "New",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            sails_rs::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<__NewParams>().type_name("__NewParams")),
                                        )
                                        .docs(
                                            &[
                                                "Another program constructor (called once at the very beginning of the program lifetime)",
                                            ],
                                        )
                                },
                            ),
                    )
            }
        }
    };
}
pub mod wasm {
    use super::*;
    use sails_rs::{gstd, hex, prelude::*};
    static mut PROGRAM: Option<DemoProgram> = None;
    #[unsafe(no_mangle)]
    extern "C" fn init() {
        use gstd::InvocationIo;
        let mut input: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
        if let Ok(request) = meta_in_program::__DefaultParams::decode_params(input) {
            let program = DemoProgram::default();
            unsafe {
                PROGRAM = Some(program);
            }
        } else if let Ok(request) = meta_in_program::__NewParams::decode_params(input) {
            let program = DemoProgram::new(request.counter, request.dog_position)
                .unwrap();
            unsafe {
                PROGRAM = Some(program);
            }
        } else {
            gstd::unknown_input_panic("Unexpected ctor", input)
        };
    }
    #[unsafe(no_mangle)]
    extern "C" fn handle() {
        if gstd::msg::value() > 0 && gstd::msg::size() == 0 {
            return;
        }
        let mut input = gstd::msg::load_bytes().expect("Failed to read input");
        let program_ref = unsafe { PROGRAM.as_mut() }.expect("Program not initialized");
        if input.starts_with(&__ROUTE_PINGPONG) {
            let mut service = program_ref.ping();
            let is_async = service
                .check_asyncness(&input[__ROUTE_PINGPONG.len()..])
                .unwrap_or_else(|| {
                    gstd::unknown_input_panic(
                        "Unknown call",
                        &input[__ROUTE_PINGPONG.len()..],
                    )
                });
            if is_async {
                gstd::message_loop(async move {
                    service
                        .try_handle_async(
                            &input[__ROUTE_PINGPONG.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| {
                            gstd::unknown_input_panic("Unknown request", &input)
                        });
                });
            } else {
                service
                    .try_handle(
                        &input[__ROUTE_PINGPONG.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .unwrap_or_else(|| gstd::unknown_input_panic(
                        "Unknown request",
                        &input,
                    ));
            }
        } else if input.starts_with(&__ROUTE_COUNTER) {
            let mut service = program_ref.counter();
            let is_async = service
                .check_asyncness(&input[__ROUTE_COUNTER.len()..])
                .unwrap_or_else(|| {
                    gstd::unknown_input_panic(
                        "Unknown call",
                        &input[__ROUTE_COUNTER.len()..],
                    )
                });
            if is_async {
                gstd::message_loop(async move {
                    service
                        .try_handle_async(
                            &input[__ROUTE_COUNTER.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| {
                            gstd::unknown_input_panic("Unknown request", &input)
                        });
                });
            } else {
                service
                    .try_handle(
                        &input[__ROUTE_COUNTER.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .unwrap_or_else(|| gstd::unknown_input_panic(
                        "Unknown request",
                        &input,
                    ));
            }
        } else if input.starts_with(&__ROUTE_DOG) {
            let mut service = program_ref.dog();
            let is_async = service
                .check_asyncness(&input[__ROUTE_DOG.len()..])
                .unwrap_or_else(|| {
                    gstd::unknown_input_panic(
                        "Unknown call",
                        &input[__ROUTE_DOG.len()..],
                    )
                });
            if is_async {
                gstd::message_loop(async move {
                    service
                        .try_handle_async(
                            &input[__ROUTE_DOG.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| {
                            gstd::unknown_input_panic("Unknown request", &input)
                        });
                });
            } else {
                service
                    .try_handle(
                        &input[__ROUTE_DOG.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .unwrap_or_else(|| gstd::unknown_input_panic(
                        "Unknown request",
                        &input,
                    ));
            }
        } else if input.starts_with(&__ROUTE_REFERENCES) {
            let mut service = program_ref.references();
            let is_async = service
                .check_asyncness(&input[__ROUTE_REFERENCES.len()..])
                .unwrap_or_else(|| {
                    gstd::unknown_input_panic(
                        "Unknown call",
                        &input[__ROUTE_REFERENCES.len()..],
                    )
                });
            if is_async {
                gstd::message_loop(async move {
                    service
                        .try_handle_async(
                            &input[__ROUTE_REFERENCES.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| {
                            gstd::unknown_input_panic("Unknown request", &input)
                        });
                });
            } else {
                service
                    .try_handle(
                        &input[__ROUTE_REFERENCES.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .unwrap_or_else(|| gstd::unknown_input_panic(
                        "Unknown request",
                        &input,
                    ));
            }
        } else if input.starts_with(&__ROUTE_THISTHAT) {
            let mut service = program_ref.this_that();
            let is_async = service
                .check_asyncness(&input[__ROUTE_THISTHAT.len()..])
                .unwrap_or_else(|| {
                    gstd::unknown_input_panic(
                        "Unknown call",
                        &input[__ROUTE_THISTHAT.len()..],
                    )
                });
            if is_async {
                gstd::message_loop(async move {
                    service
                        .try_handle_async(
                            &input[__ROUTE_THISTHAT.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| {
                            gstd::unknown_input_panic("Unknown request", &input)
                        });
                });
            } else {
                service
                    .try_handle(
                        &input[__ROUTE_THISTHAT.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .unwrap_or_else(|| gstd::unknown_input_panic(
                        "Unknown request",
                        &input,
                    ));
            }
        } else if input.starts_with(&__ROUTE_VALUEFEE) {
            let mut service = program_ref.value_fee();
            let is_async = service
                .check_asyncness(&input[__ROUTE_VALUEFEE.len()..])
                .unwrap_or_else(|| {
                    gstd::unknown_input_panic(
                        "Unknown call",
                        &input[__ROUTE_VALUEFEE.len()..],
                    )
                });
            if is_async {
                gstd::message_loop(async move {
                    service
                        .try_handle_async(
                            &input[__ROUTE_VALUEFEE.len()..],
                            |encoded_result, value| {
                                gstd::msg::reply_bytes(encoded_result, value)
                                    .expect("Failed to send output");
                            },
                        )
                        .await
                        .unwrap_or_else(|| {
                            gstd::unknown_input_panic("Unknown request", &input)
                        });
                });
            } else {
                service
                    .try_handle(
                        &input[__ROUTE_VALUEFEE.len()..],
                        |encoded_result, value| {
                            gstd::msg::reply_bytes(encoded_result, value)
                                .expect("Failed to send output");
                        },
                    )
                    .unwrap_or_else(|| gstd::unknown_input_panic(
                        "Unknown request",
                        &input,
                    ));
            }
        } else {
            gstd::unknown_input_panic("Unexpected service", &input)
        };
    }
    #[unsafe(no_mangle)]
    extern "C" fn handle_reply() {
        use sails_rs::meta::ProgramMeta;
        if DemoProgram::ASYNC {
            gstd::handle_reply_with_hook();
        }
    }
    #[unsafe(no_mangle)]
    extern "C" fn handle_signal() {
        use sails_rs::meta::ProgramMeta;
        if DemoProgram::ASYNC {
            gstd::handle_signal();
        }
    }
}
