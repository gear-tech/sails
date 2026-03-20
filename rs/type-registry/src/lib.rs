#![no_std]

pub extern crate alloc;
pub use core;

pub mod builder;
pub mod meta_type;
pub mod registry;
pub mod trait_impls;
pub mod ty;

pub use crate::builder::{CompositeBuilder, TypeBuilder, VariantBuilder, VariantDefBuilder};
pub use crate::meta_type::MetaType;
pub use crate::registry::{Registry, TypeInfo, TypeRef};
pub use crate::ty::Type;

#[cfg(feature = "derive")]
pub use sails_type_registry_derive::TypeInfo;

pub mod prelude {
    pub use crate::alloc;
    pub use crate::builder::{CompositeBuilder, TypeBuilder, VariantBuilder, VariantDefBuilder};
    pub use crate::registry::{Registry, TypeInfo, TypeRef};
    pub use crate::ty::Type;
    pub use core;
}
