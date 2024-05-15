#![no_std]

pub use hex::{self};
pub use prelude::*;

pub mod calls;
pub mod errors;
pub mod gstd;
#[cfg(not(target_arch = "wasm32"))]
pub mod gtest;
pub mod prelude;
mod types;

pub mod meta {
    use scale_info::MetaType;

    pub trait ServiceMeta {
        fn commands() -> MetaType;
        fn queries() -> MetaType;
        fn events() -> MetaType;
    }

    pub struct AnyServiceMeta {
        commands: MetaType,
        queries: MetaType,
        events: MetaType,
    }

    impl AnyServiceMeta {
        pub fn new<S: ServiceMeta>() -> Self {
            Self {
                commands: S::commands(),
                queries: S::queries(),
                events: S::events(),
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
    }

    pub trait ProgramMeta {
        fn constructors() -> MetaType;
        fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)>;
    }
}
