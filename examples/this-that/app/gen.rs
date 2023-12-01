#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
use gmeta::{InOut, Metadata};
#[allow(unused_imports)]
use gstd::debug;
use gstd::prelude::*;
use sails_macros::{command_handlers, query_handlers};
use sails_service::{BoxedFuture, SimpleService};
pub struct ProgramMetadata;
impl Metadata for ProgramMetadata {
    type Init = ();
    type Handle = InOut<commands::Commands, commands::CommandResponses>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<queries::Queries, queries::QueryResponses>;
}
pub struct CommandProcessorMeta;
impl sails_service::CommandProcessorMeta for CommandProcessorMeta {
    type Request = commands::Commands;
    type Response = commands::CommandResponses;
    type ProcessFn = fn(Self::Request) -> BoxedFuture<(Self::Response, bool)>;
}
pub struct QueryProcessorMeta;
impl sails_service::QueryProcessorMeta for QueryProcessorMeta {
    type Request = queries::Queries;
    type Response = queries::QueryResponses;
    type ProcessFn = fn(Self::Request) -> (Self::Response, bool);
}
pub type Service = SimpleService<CommandProcessorMeta, QueryProcessorMeta>;
pub struct DoThatParam {
    pub p1: u32,
    pub p2: String,
    pub p3: ManyVariants,
}
#[automatically_derived]
impl ::core::fmt::Debug for DoThatParam {
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
    impl ::parity_scale_codec::Encode for DoThatParam {
        fn size_hint(&self) -> usize {
            0_usize
                .saturating_add(::parity_scale_codec::Encode::size_hint(&self.p1))
                .saturating_add(::parity_scale_codec::Encode::size_hint(&self.p2))
                .saturating_add(::parity_scale_codec::Encode::size_hint(&self.p3))
        }
        fn encode_to<
            __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
        >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
            ::parity_scale_codec::Encode::encode_to(&self.p1, __codec_dest_edqy);
            ::parity_scale_codec::Encode::encode_to(&self.p2, __codec_dest_edqy);
            ::parity_scale_codec::Encode::encode_to(&self.p3, __codec_dest_edqy);
        }
    }
    #[automatically_derived]
    impl ::parity_scale_codec::EncodeLike for DoThatParam {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::parity_scale_codec::Decode for DoThatParam {
        fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
            ::core::result::Result::Ok(DoThatParam {
                p1: {
                    let __codec_res_edqy = <u32 as ::parity_scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `DoThatParam::p1`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
                p2: {
                    let __codec_res_edqy = <String as ::parity_scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `DoThatParam::p2`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
                p3: {
                    let __codec_res_edqy = <ManyVariants as ::parity_scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `DoThatParam::p3`"),
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
    impl ::scale_info::TypeInfo for DoThatParam {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(::scale_info::Path::new("DoThatParam", "this_that_app"))
                .type_params(::alloc::vec::Vec::new())
                .composite(
                    ::scale_info::build::Fields::named()
                        .field(|f| f.ty::<u32>().name("p1").type_name("u32"))
                        .field(|f| f.ty::<String>().name("p2").type_name("String"))
                        .field(|f| {
                            f.ty::<ManyVariants>().name("p3").type_name("ManyVariants")
                        }),
                )
        }
    }
};
pub struct TupleStruct(pub bool);
#[automatically_derived]
impl ::core::fmt::Debug for TupleStruct {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field1_finish(f, "TupleStruct", &&self.0)
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::parity_scale_codec::Encode for TupleStruct {
        fn size_hint(&self) -> usize {
            ::parity_scale_codec::Encode::size_hint(&&self.0)
        }
        fn encode_to<
            __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
        >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
            ::parity_scale_codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
        }
        fn encode(
            &self,
        ) -> ::parity_scale_codec::alloc::vec::Vec<::core::primitive::u8> {
            ::parity_scale_codec::Encode::encode(&&self.0)
        }
        fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
            &self,
            f: F,
        ) -> R {
            ::parity_scale_codec::Encode::using_encoded(&&self.0, f)
        }
    }
    #[automatically_derived]
    impl ::parity_scale_codec::EncodeLike for TupleStruct {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::parity_scale_codec::Decode for TupleStruct {
        fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
            ::core::result::Result::Ok(
                TupleStruct({
                    let __codec_res_edqy = <bool as ::parity_scale_codec::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `TupleStruct.0`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }),
            )
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for TupleStruct {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(::scale_info::Path::new("TupleStruct", "this_that_app"))
                .type_params(::alloc::vec::Vec::new())
                .composite(
                    ::scale_info::build::Fields::unnamed()
                        .field(|f| f.ty::<bool>().type_name("bool")),
                )
        }
    }
};
pub enum ManyVariants {
    One,
    Two(u32),
    Three(Option<Vec<u8>>),
    Four { a: u32, b: Option<u16> },
    Five(String, u32),
    Six((u32,)),
}
#[automatically_derived]
impl ::core::fmt::Debug for ManyVariants {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ManyVariants::One => ::core::fmt::Formatter::write_str(f, "One"),
            ManyVariants::Two(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Two", &__self_0)
            }
            ManyVariants::Three(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Three", &__self_0)
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
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Six", &__self_0)
            }
        }
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::parity_scale_codec::Encode for ManyVariants {
        fn size_hint(&self) -> usize {
            1_usize
                + match *self {
                    ManyVariants::One => 0_usize,
                    ManyVariants::Two(ref aa) => {
                        0_usize
                            .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                    }
                    ManyVariants::Three(ref aa) => {
                        0_usize
                            .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                    }
                    ManyVariants::Four { ref a, ref b } => {
                        0_usize
                            .saturating_add(::parity_scale_codec::Encode::size_hint(a))
                            .saturating_add(::parity_scale_codec::Encode::size_hint(b))
                    }
                    ManyVariants::Five(ref aa, ref ba) => {
                        0_usize
                            .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                            .saturating_add(::parity_scale_codec::Encode::size_hint(ba))
                    }
                    ManyVariants::Six(ref aa) => {
                        0_usize
                            .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                    }
                    _ => 0_usize,
                }
        }
        fn encode_to<
            __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
        >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
            match *self {
                ManyVariants::One => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                }
                ManyVariants::Two(ref aa) => {
                    __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                    ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                ManyVariants::Three(ref aa) => {
                    __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                    ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                ManyVariants::Four { ref a, ref b } => {
                    __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                    ::parity_scale_codec::Encode::encode_to(a, __codec_dest_edqy);
                    ::parity_scale_codec::Encode::encode_to(b, __codec_dest_edqy);
                }
                ManyVariants::Five(ref aa, ref ba) => {
                    __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                    ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    ::parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                }
                ManyVariants::Six(ref aa) => {
                    __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                    ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                _ => {}
            }
        }
    }
    #[automatically_derived]
    impl ::parity_scale_codec::EncodeLike for ManyVariants {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::parity_scale_codec::Decode for ManyVariants {
        fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
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
                __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || { ::core::result::Result::Ok(ManyVariants::One) })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            ManyVariants::Two({
                                let __codec_res_edqy = <u32 as ::parity_scale_codec::Decode>::decode(
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
                __codec_x_edqy if __codec_x_edqy == 2usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            ManyVariants::Three({
                                let __codec_res_edqy = <Option<
                                    Vec<u8>,
                                > as ::parity_scale_codec::Decode>::decode(
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
                __codec_x_edqy if __codec_x_edqy == 3usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(ManyVariants::Four {
                            a: {
                                let __codec_res_edqy = <u32 as ::parity_scale_codec::Decode>::decode(
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
                                > as ::parity_scale_codec::Decode>::decode(
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
                __codec_x_edqy if __codec_x_edqy == 4usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            ManyVariants::Five(
                                {
                                    let __codec_res_edqy = <String as ::parity_scale_codec::Decode>::decode(
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
                                    let __codec_res_edqy = <u32 as ::parity_scale_codec::Decode>::decode(
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
                __codec_x_edqy if __codec_x_edqy == 5usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            ManyVariants::Six({
                                let __codec_res_edqy = <(
                                    u32,
                                ) as ::parity_scale_codec::Decode>::decode(
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
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for ManyVariants {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(::scale_info::Path::new("ManyVariants", "this_that_app"))
                .type_params(::alloc::vec::Vec::new())
                .variant(
                    ::scale_info::build::Variants::new()
                        .variant("One", |v| v.index(0usize as ::core::primitive::u8))
                        .variant(
                            "Two",
                            |v| {
                                v
                                    .index(1usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
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
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| {
                                                f.ty::<Option<Vec<u8>>>().type_name("Option<Vec<u8>>")
                                            }),
                                    )
                            },
                        )
                        .variant(
                            "Four",
                            |v| {
                                v
                                    .index(3usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::named()
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
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| f.ty::<String>().type_name("String"))
                                            .field(|f| f.ty::<u32>().type_name("u32")),
                                    )
                            },
                        )
                        .variant(
                            "Six",
                            |v| {
                                v
                                    .index(5usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| f.ty::<(u32,)>().type_name("(u32,)")),
                                    )
                            },
                        ),
                )
        }
    }
};
pub mod commands {
    extern crate parity_scale_codec as commands_scale_codec;
    extern crate scale_info as commands_scale_info;
    pub enum Commands {
        DoThis(u32, String, (Option<String>, u8), TupleStruct),
        DoThat(DoThatParam),
        Fail(String),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Commands {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Commands::DoThis(__self_0, __self_1, __self_2, __self_3) => {
                    ::core::fmt::Formatter::debug_tuple_field4_finish(
                        f,
                        "DoThis",
                        __self_0,
                        __self_1,
                        __self_2,
                        &__self_3,
                    )
                }
                Commands::DoThat(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "DoThat",
                        &__self_0,
                    )
                }
                Commands::Fail(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Fail",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Encode for Commands {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        Commands::DoThis(ref aa, ref ba, ref ca, ref da) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                                .saturating_add(::parity_scale_codec::Encode::size_hint(ba))
                                .saturating_add(::parity_scale_codec::Encode::size_hint(ca))
                                .saturating_add(::parity_scale_codec::Encode::size_hint(da))
                        }
                        Commands::DoThat(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        Commands::Fail(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                match *self {
                    Commands::DoThis(ref aa, ref ba, ref ca, ref da) => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        ::parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                        ::parity_scale_codec::Encode::encode_to(ca, __codec_dest_edqy);
                        ::parity_scale_codec::Encode::encode_to(da, __codec_dest_edqy);
                    }
                    Commands::DoThat(ref aa) => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    Commands::Fail(ref aa) => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::parity_scale_codec::EncodeLike for Commands {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Decode for Commands {
            fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e
                            .chain(
                                "Could not decode `Commands`, failed to read variant byte",
                            )
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 0usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                Commands::DoThis(
                                    {
                                        let __codec_res_edqy = <u32 as ::parity_scale_codec::Decode>::decode(
                                            __codec_input_edqy,
                                        );
                                        match __codec_res_edqy {
                                            ::core::result::Result::Err(e) => {
                                                return ::core::result::Result::Err(
                                                    e.chain("Could not decode `Commands::DoThis.0`"),
                                                );
                                            }
                                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                                __codec_res_edqy
                                            }
                                        }
                                    },
                                    {
                                        let __codec_res_edqy = <String as ::parity_scale_codec::Decode>::decode(
                                            __codec_input_edqy,
                                        );
                                        match __codec_res_edqy {
                                            ::core::result::Result::Err(e) => {
                                                return ::core::result::Result::Err(
                                                    e.chain("Could not decode `Commands::DoThis.1`"),
                                                );
                                            }
                                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                                __codec_res_edqy
                                            }
                                        }
                                    },
                                    {
                                        let __codec_res_edqy = <(
                                            Option<String>,
                                            u8,
                                        ) as ::parity_scale_codec::Decode>::decode(
                                            __codec_input_edqy,
                                        );
                                        match __codec_res_edqy {
                                            ::core::result::Result::Err(e) => {
                                                return ::core::result::Result::Err(
                                                    e.chain("Could not decode `Commands::DoThis.2`"),
                                                );
                                            }
                                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                                __codec_res_edqy
                                            }
                                        }
                                    },
                                    {
                                        let __codec_res_edqy = <TupleStruct as ::parity_scale_codec::Decode>::decode(
                                            __codec_input_edqy,
                                        );
                                        match __codec_res_edqy {
                                            ::core::result::Result::Err(e) => {
                                                return ::core::result::Result::Err(
                                                    e.chain("Could not decode `Commands::DoThis.3`"),
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
                    __codec_x_edqy if __codec_x_edqy
                        == 1usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                Commands::DoThat({
                                    let __codec_res_edqy = <DoThatParam as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `Commands::DoThat.0`"),
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
                    __codec_x_edqy if __codec_x_edqy
                        == 2usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                Commands::Fail({
                                    let __codec_res_edqy = <String as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `Commands::Fail.0`"),
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
                                    "Could not decode `Commands`, variant doesn't exist",
                                ),
                            )
                        })();
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Commands {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Commands", "this_that_app::commands"))
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        ::scale_info::build::Variants::new()
                            .variant(
                                "DoThis",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<u32>().type_name("u32"))
                                                .field(|f| f.ty::<String>().type_name("String"))
                                                .field(|f| {
                                                    f
                                                        .ty::<(Option<String>, u8)>()
                                                        .type_name("(Option<String>, u8)")
                                                })
                                                .field(|f| f.ty::<TupleStruct>().type_name("TupleStruct")),
                                        )
                                },
                            )
                            .variant(
                                "DoThat",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<DoThatParam>().type_name("DoThatParam")),
                                        )
                                },
                            )
                            .variant(
                                "Fail",
                                |v| {
                                    v
                                        .index(2usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| f.ty::<String>().type_name("String")),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    pub enum CommandResponses {
        DoThis(Result<(String, u32), String>),
        DoThat(Result<(String, u32), (String,)>),
        Fail(Result<(), String>),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for CommandResponses {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                CommandResponses::DoThis(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "DoThis",
                        &__self_0,
                    )
                }
                CommandResponses::DoThat(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "DoThat",
                        &__self_0,
                    )
                }
                CommandResponses::Fail(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Fail",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Encode for CommandResponses {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        CommandResponses::DoThis(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        CommandResponses::DoThat(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        CommandResponses::Fail(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                match *self {
                    CommandResponses::DoThis(ref aa) => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    CommandResponses::DoThat(ref aa) => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    CommandResponses::Fail(ref aa) => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::parity_scale_codec::EncodeLike for CommandResponses {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Decode for CommandResponses {
            fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e
                            .chain(
                                "Could not decode `CommandResponses`, failed to read variant byte",
                            )
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 0usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                CommandResponses::DoThis({
                                    let __codec_res_edqy = <Result<
                                        (String, u32),
                                        String,
                                    > as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `CommandResponses::DoThis.0`"),
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
                    __codec_x_edqy if __codec_x_edqy
                        == 1usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                CommandResponses::DoThat({
                                    let __codec_res_edqy = <Result<
                                        (String, u32),
                                        (String,),
                                    > as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `CommandResponses::DoThat.0`"),
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
                    __codec_x_edqy if __codec_x_edqy
                        == 2usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                CommandResponses::Fail({
                                    let __codec_res_edqy = <Result<
                                        (),
                                        String,
                                    > as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `CommandResponses::Fail.0`"),
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
                                    "Could not decode `CommandResponses`, variant doesn't exist",
                                ),
                            )
                        })();
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for CommandResponses {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new(
                            "CommandResponses",
                            "this_that_app::commands",
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        ::scale_info::build::Variants::new()
                            .variant(
                                "DoThis",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f
                                                        .ty::<Result<(String, u32), String>>()
                                                        .type_name("Result<(String, u32), String>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "DoThat",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f
                                                        .ty::<Result<(String, u32), (String,)>>()
                                                        .type_name("Result<(String, u32), (String,)>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "Fail",
                                |v| {
                                    v
                                        .index(2usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f.ty::<Result<(), String>>().type_name("Result<(), String>")
                                                }),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    use super::*;
    #[cfg(feature = "handlers")]
    pub mod handlers {
        use super::*;
        pub async fn process_commands(request: Commands) -> (CommandResponses, bool) {
            match request {
                Commands::DoThis(v0, v1, v2, v3) => {
                    let result: Result<_, _> = do_this(v0, v1, v2, v3).await;
                    let is_error = result.is_err();
                    (CommandResponses::DoThis(result), is_error)
                }
                Commands::DoThat(v0) => {
                    let result: Result<_, _> = do_that(v0);
                    let is_error = result.is_err();
                    (CommandResponses::DoThat(result), is_error)
                }
                Commands::Fail(v0) => {
                    let result: Result<_, _> = fail(v0);
                    let is_error = result.is_err();
                    (CommandResponses::Fail(result), is_error)
                }
            }
        }
        async fn do_this(
            p1: u32,
            p2: String,
            p3: (Option<String>, u8),
            p4: TupleStruct,
        ) -> Result<(String, u32), String> {
            ::gstd::ext::debug(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "Handling \'do_this\': {0}, {1}, {2:?}, {3:?}",
                                p1,
                                p2,
                                p3,
                                p4,
                            ),
                        );
                        res
                    },
                )
                .unwrap();
            Ok((p2, p1))
        }
        fn do_that(param: DoThatParam) -> Result<(String, u32), (String,)> {
            ::gstd::ext::debug(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("Handling \'do_that\': {0:?}", param),
                        );
                        res
                    },
                )
                .unwrap();
            Ok((param.p2, param.p1))
        }
        fn fail(message: String) -> Result<(), String> {
            ::gstd::ext::debug(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("Handling \'fail\': {0}", message),
                        );
                        res
                    },
                )
                .unwrap();
            Err(message)
        }
        #[cfg(feature = "client")]
        pub mod client {
            extern crate gtest;
            extern crate sails_client;
            use gtest::{Program, RunResult};
            use sails_client::Call;
            use super::*;
            pub struct Client {}
            impl Client {
                pub fn new() -> Self {
                    Self {}
                }
                pub fn do_this(
                    &self,
                    p1: u32,
                    p2: String,
                    p3: (Option<String>, u8),
                    p4: TupleStruct,
                ) -> Call<
                    CommandResponses,
                    fn(CommandResponses) -> Option<Result<(String, u32), String>>,
                    Result<(String, u32), String>,
                > {
                    let payload = Commands::DoThis(p1, p2, p3, p4).encode();
                    let map = |
                        r: CommandResponses,
                    | -> Option<Result<(String, u32), String>> {
                        match r {
                            CommandResponses::DoThis(result) => Some(result),
                            _ => None,
                        }
                    };
                    Call::<
                        CommandResponses,
                        fn(CommandResponses) -> Option<Result<(String, u32), String>>,
                        Result<(String, u32), String>,
                    >::new(payload, map)
                }
                pub fn do_that(
                    &self,
                    param: DoThatParam,
                ) -> Call<
                    CommandResponses,
                    fn(CommandResponses) -> Option<Result<(String, u32), (String,)>>,
                    Result<(String, u32), (String,)>,
                > {
                    let payload = Commands::DoThat(param).encode();
                    let map = |
                        r: CommandResponses,
                    | -> Option<Result<(String, u32), (String,)>> {
                        match r {
                            CommandResponses::DoThat(result) => Some(result),
                            _ => None,
                        }
                    };
                    Call::<
                        CommandResponses,
                        fn(CommandResponses) -> Option<Result<(String, u32), (String,)>>,
                        Result<(String, u32), (String,)>,
                    >::new(payload, map)
                }
                pub fn fail(
                    &self,
                    message: String,
                ) -> Call<
                    CommandResponses,
                    fn(CommandResponses) -> Option<Result<(), String>>,
                    Result<(), String>,
                > {
                    let payload = Commands::Fail(message).encode();
                    let map = |r: CommandResponses| -> Option<Result<(), String>> {
                        match r {
                            CommandResponses::Fail(result) => Some(result),
                            _ => None,
                        }
                    };
                    Call::<
                        CommandResponses,
                        fn(CommandResponses) -> Option<Result<(), String>>,
                        Result<(), String>,
                    >::new(payload, map)
                }
            }
        }
    }
}
pub mod queries {
    extern crate parity_scale_codec as queries_scale_codec;
    extern crate scale_info as queries_scale_info;
    pub enum Queries {
        This(),
        That(),
        Fail(),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Queries {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Queries::This() => "This",
                    Queries::That() => "That",
                    Queries::Fail() => "Fail",
                },
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Encode for Queries {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        Queries::This() => 0_usize,
                        Queries::That() => 0_usize,
                        Queries::Fail() => 0_usize,
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                match *self {
                    Queries::This() => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                    }
                    Queries::That() => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                    }
                    Queries::Fail() => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::parity_scale_codec::EncodeLike for Queries {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Decode for Queries {
            fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e
                            .chain(
                                "Could not decode `Queries`, failed to read variant byte",
                            )
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 0usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Queries::This())
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 1usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Queries::That())
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 2usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Queries::Fail())
                        })();
                    }
                    _ => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Err(
                                <_ as ::core::convert::Into<
                                    _,
                                >>::into(
                                    "Could not decode `Queries`, variant doesn't exist",
                                ),
                            )
                        })();
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Queries {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Queries", "this_that_app::queries"))
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        ::scale_info::build::Variants::new()
                            .variant(
                                "This",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(::scale_info::build::Fields::unnamed())
                                },
                            )
                            .variant(
                                "That",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(::scale_info::build::Fields::unnamed())
                                },
                            )
                            .variant(
                                "Fail",
                                |v| {
                                    v
                                        .index(2usize as ::core::primitive::u8)
                                        .fields(::scale_info::build::Fields::unnamed())
                                },
                            ),
                    )
            }
        }
    };
    pub enum QueryResponses {
        This(Result<u32, String>),
        That(Result<String, String>),
        Fail(Result<(), String>),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for QueryResponses {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                QueryResponses::This(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "This",
                        &__self_0,
                    )
                }
                QueryResponses::That(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "That",
                        &__self_0,
                    )
                }
                QueryResponses::Fail(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Fail",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Encode for QueryResponses {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        QueryResponses::This(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        QueryResponses::That(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        QueryResponses::Fail(ref aa) => {
                            0_usize
                                .saturating_add(::parity_scale_codec::Encode::size_hint(aa))
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                match *self {
                    QueryResponses::This(ref aa) => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    QueryResponses::That(ref aa) => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    QueryResponses::Fail(ref aa) => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        ::parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::parity_scale_codec::EncodeLike for QueryResponses {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Decode for QueryResponses {
            fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e
                            .chain(
                                "Could not decode `QueryResponses`, failed to read variant byte",
                            )
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 0usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                QueryResponses::This({
                                    let __codec_res_edqy = <Result<
                                        u32,
                                        String,
                                    > as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `QueryResponses::This.0`"),
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
                    __codec_x_edqy if __codec_x_edqy
                        == 1usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                QueryResponses::That({
                                    let __codec_res_edqy = <Result<
                                        String,
                                        String,
                                    > as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `QueryResponses::That.0`"),
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
                    __codec_x_edqy if __codec_x_edqy
                        == 2usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(
                                QueryResponses::Fail({
                                    let __codec_res_edqy = <Result<
                                        (),
                                        String,
                                    > as ::parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `QueryResponses::Fail.0`"),
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
                                    "Could not decode `QueryResponses`, variant doesn't exist",
                                ),
                            )
                        })();
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for QueryResponses {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new(
                            "QueryResponses",
                            "this_that_app::queries",
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .variant(
                        ::scale_info::build::Variants::new()
                            .variant(
                                "This",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f
                                                        .ty::<Result<u32, String>>()
                                                        .type_name("Result<u32, String>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "That",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f
                                                        .ty::<Result<String, String>>()
                                                        .type_name("Result<String, String>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "Fail",
                                |v| {
                                    v
                                        .index(2usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::unnamed()
                                                .field(|f| {
                                                    f.ty::<Result<(), String>>().type_name("Result<(), String>")
                                                }),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    use super::*;
    #[cfg(feature = "handlers")]
    pub mod handlers {
        use super::*;
        pub fn process_queries(request: Queries) -> (QueryResponses, bool) {
            match request {
                Queries::This() => {
                    let result: Result<_, _> = this();
                    let is_error = result.is_err();
                    (QueryResponses::This(result), is_error)
                }
                Queries::That() => {
                    let result: Result<_, _> = that();
                    let is_error = result.is_err();
                    (QueryResponses::That(result), is_error)
                }
                Queries::Fail() => {
                    let result: Result<_, _> = fail();
                    let is_error = result.is_err();
                    (QueryResponses::Fail(result), is_error)
                }
            }
        }
        fn this() -> Result<u32, String> {
            ::gstd::ext::debug(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("Handling \'this\'"),
                        );
                        res
                    },
                )
                .unwrap();
            Ok(42)
        }
        fn that() -> Result<String, String> {
            ::gstd::ext::debug(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("Handling \'that\'"),
                        );
                        res
                    },
                )
                .unwrap();
            Ok("Forty two".into())
        }
        fn fail() -> Result<(), String> {
            ::gstd::ext::debug(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("Handling \'fail\'"),
                        );
                        res
                    },
                )
                .unwrap();
            Err("Failed".into())
        }
        #[cfg(feature = "client")]
        pub mod client {
            extern crate gtest;
            extern crate sails_client;
            use gtest::{Program, RunResult};
            use sails_client::Call;
            use super::*;
            pub struct Client {}
            impl Client {
                pub fn new() -> Self {
                    Self {}
                }
                pub fn this(
                    &self,
                ) -> Call<
                    QueryResponses,
                    fn(QueryResponses) -> Option<Result<u32, String>>,
                    Result<u32, String>,
                > {
                    let payload = Queries::This().encode();
                    let map = |r: QueryResponses| -> Option<Result<u32, String>> {
                        match r {
                            QueryResponses::This(result) => Some(result),
                            _ => None,
                        }
                    };
                    Call::<
                        QueryResponses,
                        fn(QueryResponses) -> Option<Result<u32, String>>,
                        Result<u32, String>,
                    >::new(payload, map)
                }
                pub fn that(
                    &self,
                ) -> Call<
                    QueryResponses,
                    fn(QueryResponses) -> Option<Result<String, String>>,
                    Result<String, String>,
                > {
                    let payload = Queries::That().encode();
                    let map = |r: QueryResponses| -> Option<Result<String, String>> {
                        match r {
                            QueryResponses::That(result) => Some(result),
                            _ => None,
                        }
                    };
                    Call::<
                        QueryResponses,
                        fn(QueryResponses) -> Option<Result<String, String>>,
                        Result<String, String>,
                    >::new(payload, map)
                }
                pub fn fail(
                    &self,
                ) -> Call<
                    QueryResponses,
                    fn(QueryResponses) -> Option<Result<(), String>>,
                    Result<(), String>,
                > {
                    let payload = Queries::Fail().encode();
                    let map = |r: QueryResponses| -> Option<Result<(), String>> {
                        match r {
                            QueryResponses::Fail(result) => Some(result),
                            _ => None,
                        }
                    };
                    Call::<
                        QueryResponses,
                        fn(QueryResponses) -> Option<Result<(), String>>,
                        Result<(), String>,
                    >::new(payload, map)
                }
            }
        }
    }
}
