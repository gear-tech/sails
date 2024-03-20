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
///
/// [`alloc::collections`]: ::alloc::collections
pub mod collections {
    extern crate alloc;

    pub use ::hashbrown::{hash_map, hash_set, HashMap, HashSet};
    pub use alloc::collections::*;

    /// Reexports from [`hashbrown`].
    pub mod hashbrown {
        pub use ::hashbrown::{Equivalent, TryReserveError};
    }
}
/// Utilities related to FFI bindings.
///
/// See [`alloc::ffi`] & [`core::ffi`].
///
/// [`alloc::ffi`]: ::alloc::ffi
pub mod ffi {
    extern crate alloc;

    pub use alloc::ffi::*;
    pub use core::ffi::*;
}

pub use crate::types::*;

// Reexports from third-party libraries
pub use crate::scale::*;
