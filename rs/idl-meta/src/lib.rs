#![no_std]

extern crate alloc;

pub use sails_idl_ast::InterfaceId;
use sails_type_registry::{MetaType, TypeInfo};

mod header;
pub use header::*;

/// A trait for types that have a static Interface ID.
pub trait Identifiable {
    const INTERFACE_ID: InterfaceId;
}

/// A trait for types that represent a service method, providing its Entry ID.
pub trait MethodMeta: Identifiable {
    const ENTRY_ID: u16;
}

#[derive(Debug, Clone)]
pub struct BaseServiceMeta {
    pub name: &'static str,
    pub interface_id: InterfaceId,
    pub meta: AnyServiceMeta,
    pub base: &'static [BaseServiceMeta],
}

impl BaseServiceMeta {
    pub const fn new<S: ServiceMeta + ?Sized>(name: &'static str) -> Self {
        Self {
            name,
            interface_id: S::INTERFACE_ID,
            meta: S::META,
            base: S::BASE_SERVICES,
        }
    }
}

/// Metadata for a service method.
#[derive(Debug)]
pub struct MethodMetadata {
    pub name: &'static str,
    pub entry_id: u16,
    pub hash: [u8; 32],
    pub is_async: bool,
}

pub trait ServiceMeta: Identifiable {
    type CommandsMeta: TypeInfo;
    type QueriesMeta: TypeInfo;
    type EventsMeta: TypeInfo;
    /// The order of base services here is lexicographical by their names
    const BASE_SERVICES: &'static [BaseServiceMeta];
    /// The order of base services here is lexicographical by their names
    // const BASE_SERVICES_IDS: &'static [AnyServiceIds];
    const METHODS: &'static [MethodMetadata];
    const ASYNC: bool;
    const META: AnyServiceMeta = AnyServiceMeta::new::<Self>();
}

#[derive(Debug, Clone)]
pub struct AnyServiceMeta {
    commands: MetaType,
    queries: MetaType,
    events: MetaType,
    base_services: &'static [BaseServiceMeta],
    interface_id: InterfaceId,
}

impl AnyServiceMeta {
    pub const fn new<S: ServiceMeta + ?Sized>() -> Self {
        Self {
            commands: S::CommandsMeta::META,
            queries: S::QueriesMeta::META,
            events: S::EventsMeta::META,
            base_services: S::BASE_SERVICES,
            interface_id: S::INTERFACE_ID,
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

    pub fn base_services(&self) -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        self.base_services
            .iter()
            .map(|base| (base.name, base.meta.clone()))
    }

    pub fn interface_id(&self) -> InterfaceId {
        self.interface_id
    }
}

pub trait ProgramMeta {
    type ConstructorsMeta: TypeInfo;
    const SERVICES: &'static [(&'static str, AnyServiceMeta)];
    const ASYNC: bool;
}

pub const fn count_base_services<S: ServiceMeta>() -> usize {
    let mut counter = 0;

    let direct_base_services = S::BASE_SERVICES;
    let mut idx = 0;
    while idx != direct_base_services.len() {
        let base = &direct_base_services[idx];
        count_base_services_recursive(&mut counter, base);
        idx += 1;
    }

    counter
}

const fn count_base_services_recursive(counter: &mut usize, base: &BaseServiceMeta) {
    *counter += 1;

    let base_services = base.base;
    let mut idx = 0;
    while idx != base_services.len() {
        count_base_services_recursive(counter, &base_services[idx]);
        idx += 1;
    }
}

/// Generate interface IDs array from exposed services
pub const fn interface_ids<const N: usize>(
    exposed_services: &'static [BaseServiceMeta],
) -> [(InterfaceId, u8); N] {
    let mut output = [(InterfaceId([0u8; 8]), 0u8); N];

    let mut exposed_svc_idx = 0;
    let mut output_offset = 0;
    let mut route_id = 1;
    while exposed_svc_idx != exposed_services.len() {
        let service = &exposed_services[exposed_svc_idx];
        fill_interface_ids_recursive(&mut output, &mut output_offset, service, route_id);
        exposed_svc_idx += 1;
        route_id += 1;
    }

    assert!(output_offset == N, "Mismatched interface IDs count");

    output
}

const fn fill_interface_ids_recursive(
    arr: &mut [(InterfaceId, u8)],
    offset: &mut usize,
    service: &BaseServiceMeta,
    route_id: u8,
) {
    arr[*offset] = (service.interface_id, route_id);
    *offset += 1;
    let base_services = service.base;
    let mut idx = 0;
    while idx != base_services.len() {
        fill_interface_ids_recursive(arr, offset, &base_services[idx], route_id);
        idx += 1;
    }
}

pub const fn service_has_interface_id(
    service: &BaseServiceMeta,
    interface_id: InterfaceId,
) -> bool {
    if service.interface_id.as_u64() == interface_id.as_u64() {
        true
    } else {
        let mut idx = 0;
        while idx != service.base.len() {
            if service_has_interface_id(&service.base[idx], interface_id) {
                return true;
            }
            idx += 1;
        }
        false
    }
}

pub const fn str_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut i = 0;
    while i < a_bytes.len() {
        if a_bytes[i] != b_bytes[i] {
            return false;
        }
        i += 1;
    }
    true
}

pub const fn bytes32_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut i = 0;
    while i < 32 {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

pub const fn find_method_data(
    methods: &'static [MethodMetadata],
    name: &str,
    entry_id: Option<u16>,
) -> Option<&'static MethodMetadata> {
    if let Some(id) = entry_id {
        let i = id as usize;
        if i < methods.len() {
            return Some(&methods[i]);
        }
        return None;
    }
    let mut i = 0;
    while i < methods.len() {
        let m = &methods[i];
        if str_eq(m.name, name) {
            return Some(m);
        }
        i += 1;
    }
    None
}

pub const fn find_id(methods: &'static [MethodMetadata], name: &str) -> u16 {
    let mut i = 0;
    while i < methods.len() {
        if str_eq(methods[i].name, name) {
            return methods[i].entry_id;
        }
        i += 1;
    }
    0
}
