extern crate alloc;

pub use alloc::{
    borrow,
    borrow::ToOwned,
    boxed,
    boxed::Box,
    fmt, format, rc, str, string,
    string::{String, ToString},
    vec,
    vec::Vec,
};
pub use core::{
    any, array, ascii, assert_eq, assert_ne, cell, char, clone, cmp, convert, debug_assert,
    debug_assert_eq, debug_assert_ne, default, future, hash, hint, iter, marker, matches, mem, num,
    ops, option, panic, pin, prelude::rust_2021::*, primitive, ptr, result, slice, task, time,
    todo, unimplemented, unreachable, write, writeln,
};

/// Collection types.
///
/// See [`alloc::collections`] & [`hashbrown`].
pub mod collections {
    extern crate alloc;

    pub use ::hashbrown::{HashMap, HashSet, hash_map, hash_set};
    pub use alloc::collections::*;

    /// Reexports from [`hashbrown`].
    pub mod hashbrown {
        pub use ::hashbrown::{Equivalent, TryReserveError};
    }
}
/// Utilities related to FFI bindings.
///
/// See [`alloc::ffi`] & [`core::ffi`].
pub mod ffi {
    extern crate alloc;

    pub use alloc::ffi::{CString, FromVecWithNulError, IntoStringError, NulError};
    pub use core::ffi::*;
}

#[cfg(feature = "gstd")]
pub use crate::gstd::{CommandReply, export, program, route, service};
pub use crate::types::*;
pub use gear_core_errors::{
    self as gear_core_errors, ErrorReplyReason, ReplyCode, SimpleExecutionError,
    SimpleUnavailableActorError, SuccessReplyReason,
};

pub use parity_scale_codec::{self as scale_codec, Decode, Encode, EncodeLike, Output};
pub use scale_info::{self as scale_info, TypeInfo};

#[cfg(all(feature = "gstd", feature = "ethexe"))]
pub use crate::gstd::EthEvent;
#[cfg(feature = "ethexe")]
pub use crate::solidity;
#[cfg(feature = "ethexe")]
pub use alloy_primitives;
#[cfg(feature = "ethexe")]
pub use alloy_sol_types;
#[cfg(feature = "ethexe")]
pub use keccak_const;
#[cfg(all(feature = "gstd", feature = "ethexe"))]
pub use sails_macros::event;

/// Channels, Streams and Futures extensions types
///
/// See [`futures::stream`], [`futures::channel::oneshot`]
pub use futures;
