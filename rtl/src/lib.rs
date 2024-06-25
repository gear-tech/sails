#![no_std]

pub use hex::{self};
pub use prelude::*;
pub use spin::{self};

pub mod calls;
pub mod errors;
#[cfg(not(target_arch = "wasm32"))]
pub mod gsdk;
pub mod gstd;
#[cfg(not(target_arch = "wasm32"))]
pub mod gtest;
pub mod prelude;
mod types;
#[cfg(not(target_arch = "wasm32"))]
pub mod event_listener;

pub mod meta {
    use crate::Vec;
    use scale_info::MetaType;

    pub trait ServiceMeta {
        fn commands() -> MetaType;
        fn queries() -> MetaType;
        fn events() -> MetaType;
        fn base_services() -> impl Iterator<Item = AnyServiceMeta>;
    }

    pub struct AnyServiceMeta {
        commands: MetaType,
        queries: MetaType,
        events: MetaType,
        base_services: Vec<AnyServiceMeta>,
    }

    impl AnyServiceMeta {
        pub fn new<S: ServiceMeta>() -> Self {
            Self {
                commands: S::commands(),
                queries: S::queries(),
                events: S::events(),
                base_services: S::base_services().collect(),
            }
        }

        pub fn commands(&self) -> &MetaType {
            &self.commands
        }

        pub fn queries(&self) -> &MetaType {
            &self.queries
        }

        pub fn events(&self) -> &MetaType {
            &self.events
        }

        pub fn base_services(&self) -> impl Iterator<Item = &AnyServiceMeta> {
            self.base_services.iter()
        }
    }

    pub trait ProgramMeta {
        fn constructors() -> MetaType;
        fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)>;
    }
}
