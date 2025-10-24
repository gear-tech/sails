#![no_std]

use scale_info::{MetaType, StaticTypeInfo, prelude::vec::Vec};

pub type AnyServiceMetaFn = fn() -> AnyServiceMeta;

pub struct ExtendedInterface {
    pub name: &'static str,
    pub interface_id: u64,
}

pub trait ServiceMeta {
    type CommandsMeta: StaticTypeInfo;
    type QueriesMeta: StaticTypeInfo;
    type EventsMeta: StaticTypeInfo;
    const BASE_SERVICES: &'static [AnyServiceMetaFn];
    const ASYNC: bool;
    const INTERFACE_PATH: &'static str;
    const INTERFACE_ID: u64 = 0;
    const EXTENDS: &'static [ExtendedInterface] = &[];

    fn commands() -> MetaType {
        MetaType::new::<Self::CommandsMeta>()
    }

    fn queries() -> MetaType {
        MetaType::new::<Self::QueriesMeta>()
    }

    fn events() -> MetaType {
        MetaType::new::<Self::EventsMeta>()
    }

    fn command_entry_ids() -> Vec<u16> {
        Vec::new()
    }

    fn local_command_entry_ids() -> &'static [u16] {
        &[]
    }

    fn query_entry_ids() -> Vec<u16> {
        Vec::new()
    }

    fn local_query_entry_ids() -> &'static [u16] {
        &[]
    }

    fn event_entry_ids() -> Vec<u16> {
        Vec::new()
    }

    fn local_event_entry_ids() -> Vec<u16> {
        Vec::new()
    }

    fn base_services() -> impl Iterator<Item = AnyServiceMeta> {
        Self::BASE_SERVICES.iter().map(|f| f())
    }

    fn canonical_service() -> &'static [u8] {
        &[]
    }

    fn extends() -> &'static [ExtendedInterface] {
        Self::EXTENDS
    }
}

pub trait EventEntryIdMeta {
    fn event_entry_ids() -> Vec<u16>;
}

pub struct AnyServiceMeta {
    commands: MetaType,
    queries: MetaType,
    events: MetaType,
    base_services: Vec<AnyServiceMeta>,
    interface_path: &'static str,
    interface_id: u64,
    extends: &'static [ExtendedInterface],
    command_entry_ids: Vec<u16>,
    query_entry_ids: Vec<u16>,
    event_entry_ids: Vec<u16>,
    local_command_entry_ids: &'static [u16],
    local_query_entry_ids: &'static [u16],
    local_event_entry_ids: fn() -> Vec<u16>,
    canonical_service: fn() -> &'static [u8],
}

impl AnyServiceMeta {
    pub fn new<S: ServiceMeta>() -> Self {
        Self {
            commands: S::commands(),
            queries: S::queries(),
            events: S::events(),
            base_services: S::base_services().collect(),
            interface_path: S::INTERFACE_PATH,
            interface_id: S::INTERFACE_ID,
            extends: S::extends(),
            command_entry_ids: S::command_entry_ids(),
            query_entry_ids: S::query_entry_ids(),
            event_entry_ids: S::event_entry_ids(),
            local_command_entry_ids: S::local_command_entry_ids(),
            local_query_entry_ids: S::local_query_entry_ids(),
            local_event_entry_ids: S::local_event_entry_ids,
            canonical_service: S::canonical_service,
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

    pub fn interface_path(&self) -> &'static str {
        self.interface_path
    }

    pub fn interface_id(&self) -> u64 {
        self.interface_id
    }

    pub fn extends(&self) -> &'static [ExtendedInterface] {
        self.extends
    }

    pub fn command_entry_ids(&self) -> &[u16] {
        &self.command_entry_ids
    }

    pub fn query_entry_ids(&self) -> &[u16] {
        &self.query_entry_ids
    }

    pub fn local_command_entry_ids(&self) -> &'static [u16] {
        self.local_command_entry_ids
    }

    pub fn local_query_entry_ids(&self) -> &'static [u16] {
        self.local_query_entry_ids
    }

    pub fn event_entry_ids(&self) -> &[u16] {
        &self.event_entry_ids
    }

    pub fn local_event_entry_ids(&self) -> Vec<u16> {
        (self.local_event_entry_ids)()
    }

    pub fn canonical_service(&self) -> &'static [u8] {
        (self.canonical_service)()
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
