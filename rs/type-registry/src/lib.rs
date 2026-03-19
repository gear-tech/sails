#![no_std]

pub extern crate alloc;
pub use core;

pub mod prelude {
    pub use crate::alloc;
    pub use core;
}

pub mod builder;
pub mod meta_type;
pub mod registry;
pub mod trait_impls;
pub mod ty;

pub use builder::{CompositeBuilder, TypeBuilder, VariantDefBuilder};
pub use meta_type::MetaType;
pub use registry::{Registry, TypeInfo, TypeRef};

#[cfg(feature = "derive")]
pub use sails_type_registry_derive::TypeInfo;
