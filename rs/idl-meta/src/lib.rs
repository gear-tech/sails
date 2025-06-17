#![no_std]

use scale_info::{MetaType, StaticTypeInfo, prelude::vec::Vec};

pub type AnyServiceMetaFn = fn() -> AnyServiceMeta;

pub trait ServiceMeta {
    type CommandsMeta: StaticTypeInfo;
    type QueriesMeta: StaticTypeInfo;
    type EventsMeta: StaticTypeInfo;
    const BASE_SERVICES: &'static [AnyServiceMetaFn];
    const ASYNC: bool;

    fn commands() -> MetaType {
        MetaType::new::<Self::CommandsMeta>()
    }

    fn queries() -> MetaType {
        MetaType::new::<Self::QueriesMeta>()
    }

    fn events() -> MetaType {
        MetaType::new::<Self::EventsMeta>()
    }

    fn base_services() -> impl Iterator<Item = AnyServiceMeta> {
        Self::BASE_SERVICES.iter().map(|f| f())
    }
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
    type ConstructorsMeta: StaticTypeInfo;
    const SERVICES: &'static [(&'static str, AnyServiceMetaFn)];
    const ASYNC: bool;

    fn constructors() -> MetaType {
        MetaType::new::<Self::ConstructorsMeta>()
    }

    fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        Self::SERVICES.iter().map(|(s, f)| (*s, f()))
    }
}
