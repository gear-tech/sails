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

/// Annotation key carrying the Rust module path of a user-defined type.
/// Written by the derive macro, consumed by IDL generation for name disambiguation.
pub const PATH_ANNOTATION: &str = "path";

/// Builders for constructing [`Type`] metadata manually.
pub mod builder;
/// Type-erased wrapper around a [`TypeInfo`] implementation.
pub mod meta_type;
/// Registry and trait entry points for portable type metadata.
pub mod registry;
/// Built-in `TypeInfo` implementations for standard Rust types.
pub mod trait_impls;
/// Core metadata structures stored in the registry.
pub use crate::builder::{
    CompositeBuilder, FieldBuilder, TypeBuilder, VariantBuilder, VariantDefBuilder,
};
pub use crate::meta_type::MetaType;
pub use crate::registry::{Registry, TypeInfo, TypeRef};
pub use sails_idl_ast::{NamedParam, Type, TypeDecl};

/// Derive macro for generating [`TypeInfo`] implementations.
#[cfg(feature = "derive")]
pub use sails_type_registry_derive::TypeInfo;

/// Common imports for crates that build or consume type metadata directly.
pub mod prelude {
    pub use crate::alloc;
    pub use crate::builder::{
        CompositeBuilder, FieldBuilder, TypeBuilder, VariantBuilder, VariantDefBuilder,
    };
    pub use crate::registry::{Registry, TypeInfo, TypeRef};
    pub use sails_idl_ast::{NamedParam, Type, TypeDecl};
}
