#![no_std]

use scale_info::{MetaType, StaticTypeInfo, prelude::vec::Vec};

pub type AnyServiceMetaFn = fn() -> AnyServiceMeta;

pub struct ExtendedInterface {
    pub name: &'static str,
    pub interface_id32: u32,
    pub interface_uid64: u64,
}

pub trait ServiceMeta {
    type CommandsMeta: StaticTypeInfo;
    type QueriesMeta: StaticTypeInfo;
    type EventsMeta: StaticTypeInfo;
    const BASE_SERVICES: &'static [AnyServiceMetaFn];
    const ASYNC: bool;
    const INTERFACE_PATH: &'static str;
    const INTERFACE_ID32: u32 = 0;
    const INTERFACE_UID64: u64 = 0;
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

    fn command_opcodes() -> Vec<u16> {
        Vec::new()
    }

    fn local_command_opcodes() -> &'static [u16] {
        &[]
    }

    fn query_opcodes() -> Vec<u16> {
        Vec::new()
    }

    fn local_query_opcodes() -> &'static [u16] {
        &[]
    }

    fn event_codes() -> Vec<u16> {
        Vec::new()
    }

    fn local_event_codes() -> Vec<u16> {
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

pub trait EventCodeMeta {
    fn event_codes() -> Vec<u16>;
}

pub struct AnyServiceMeta {
    commands: MetaType,
    queries: MetaType,
    events: MetaType,
    base_services: Vec<AnyServiceMeta>,
    interface_path: &'static str,
    interface_id32: u32,
    interface_uid64: u64,
    extends: &'static [ExtendedInterface],
    command_opcodes: Vec<u16>,
    query_opcodes: Vec<u16>,
    event_codes: Vec<u16>,
    local_command_opcodes: &'static [u16],
    local_query_opcodes: &'static [u16],
    local_event_codes: fn() -> Vec<u16>,
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
            interface_id32: S::INTERFACE_ID32,
            interface_uid64: S::INTERFACE_UID64,
            extends: S::extends(),
            command_opcodes: S::command_opcodes(),
            query_opcodes: S::query_opcodes(),
            event_codes: S::event_codes(),
            local_command_opcodes: S::local_command_opcodes(),
            local_query_opcodes: S::local_query_opcodes(),
            local_event_codes: S::local_event_codes,
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

    pub fn interface_id32(&self) -> u32 {
        self.interface_id32
    }

    pub fn interface_uid64(&self) -> u64 {
        self.interface_uid64
    }

    pub fn extends(&self) -> &'static [ExtendedInterface] {
        self.extends
    }

    pub fn command_opcodes(&self) -> &[u16] {
        &self.command_opcodes
    }

    pub fn query_opcodes(&self) -> &[u16] {
        &self.query_opcodes
    }

    pub fn local_command_opcodes(&self) -> &'static [u16] {
        self.local_command_opcodes
    }

    pub fn local_query_opcodes(&self) -> &'static [u16] {
        self.local_query_opcodes
    }

    pub fn event_codes(&self) -> &[u16] {
        &self.event_codes
    }

    pub fn local_event_codes(&self) -> Vec<u16> {
        (self.local_event_codes)()
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
