#![no_std]

//! Portable type metadata and registry for Sails.
//!
//! This crate defines the metadata model used by Sails to describe Rust types
//! in a portable form. Types implement [`TypeInfo`], and a [`Registry`]
//! collects their descriptions into a deduplicated table that can be consumed
//! by IDL generation and related tooling.
//!
//! Most users interact with this crate through `#[derive(TypeInfo)]`. Manual
//! construction is available through the builder types re-exported here.

pub extern crate alloc;
pub use core;

/// Builders for constructing [`ast::Type`] metadata manually.
pub mod builder;
/// Type-erased wrapper around a [`TypeInfo`] implementation.
mod meta_type;
/// Registry and trait entry points for portable type metadata.
mod registry;
/// Built-in `TypeInfo` implementations for standard Rust types.
mod trait_impls;

pub use crate::builder::{
    CompositeBuilder, FieldBuilder, ParamBuilder, TypeBuilder, VariantBuilder, VariantDefBuilder,
};
pub use crate::meta_type::MetaType;
pub use crate::registry::{Registry, TypeInfo, TypeRef, const_suffixed_name};
pub use sails_idl_ast as ast;

/// Derive macro for generating [`TypeInfo`] implementations.
#[cfg(feature = "derive")]
pub use sails_type_registry_derive::TypeInfo;
