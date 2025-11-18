use sails_idl_meta::{AnyServiceMeta, ServiceMeta, build_service_unit};

#[derive(scale_info::TypeInfo)]
pub enum CommandsMeta {
    Add(AddParams, Result<u32, String>),
    Reset(ResetParams, ()),
}

#[derive(scale_info::TypeInfo)]
pub struct AddParams {
    pub left: u32,
    pub right: u32,
}

#[derive(scale_info::TypeInfo)]
pub struct ResetParams {}

#[derive(scale_info::TypeInfo)]
pub enum QueriesMeta {
    Sum(SumParams, u32),
}

#[derive(scale_info::TypeInfo)]
pub struct SumParams {}

#[derive(scale_info::TypeInfo)]
pub enum EventsMeta {
    Updated(UpdatedEvent),
}

#[derive(scale_info::TypeInfo)]
pub struct UpdatedEvent {
    pub total: u32,
}

pub struct TestServiceMeta;

impl ServiceMeta for TestServiceMeta {
    type CommandsMeta = CommandsMeta;
    type QueriesMeta = QueriesMeta;
    type EventsMeta = EventsMeta;
    const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
    const ASYNC: bool = false;
}

#[test]
fn builds_service_unit_from_meta() {
    let unit = build_service_unit("Calculator", &AnyServiceMeta::new::<TestServiceMeta>()).unwrap();
    assert_eq!(unit.name, "Calculator");
    assert_eq!(unit.funcs.len(), 3);
    assert_eq!(unit.events.len(), 1);
    assert!(!unit.types.is_empty());
}
