use crate::meta::{AnyServiceIds, InterfaceId, ServiceMeta};

/// Count the total number of base services recursively for a service
pub const fn count_base_services<S: ServiceMeta>() -> usize {
    let mut counter = 0;

    let direct_base_services = S::BASE_SERVICES_IDS;
    let mut idx = 0;
    while idx != direct_base_services.len() {
        let any_svc_meta_fn = direct_base_services[idx];
        count_base_services_recursive(&mut counter, any_svc_meta_fn);
        idx += 1;
    }

    counter
}

const fn count_base_services_recursive(counter: &mut usize, base: AnyServiceIds) {
    *counter += 1;

    let base_services = base.base_services;
    let mut idx = 0;
    while idx != base_services.len() {
        count_base_services_recursive(counter, base_services[idx]);
        idx += 1;
    }
}

/// Generate interface IDs array from exposed services
pub const fn interface_ids<const N: usize>(
    exposed_services: &'static [AnyServiceIds],
) -> [(InterfaceId, u8); N] {
    let mut output = [(InterfaceId([0u8; 8]), 0u8); N];

    let mut exposed_svc_idx = 0;
    let mut output_offset = 0;
    let mut route_id = 1;
    while exposed_svc_idx != exposed_services.len() {
        let service = exposed_services[exposed_svc_idx];
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
    service: AnyServiceIds,
    route_id: u8,
) {
    arr[*offset] = (InterfaceId(service.interface_id), route_id);
    *offset += 1;
    let base_services = service.base_services;
    let mut idx = 0;
    while idx != base_services.len() {
        fill_interface_ids_recursive(arr, offset, base_services[idx], route_id);
        idx += 1;
    }
}
