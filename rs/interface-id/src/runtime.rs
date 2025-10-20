#![allow(clippy::result_large_err)]

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};
use sails_idl_meta::{AnyServiceMeta, ServiceMeta};
use scale_info::{MetaType, PortableRegistry, Registry, TypeDef, Variant, form::PortableForm};
use std::sync::{Mutex, OnceLock};

use core::any::TypeId;

use crate::{
    canonical::{
        CanonicalDocument, CanonicalEvent, CanonicalExtendedInterface, CanonicalFunction,
        CanonicalParam, CanonicalService, CanonicalType, FunctionKind,
    },
    canonical_type::{self as canonical_visitor, CanonicalTypeError},
    compute_ids_from_document,
};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("metadata mismatch for {kind}: expected {expected} entries, found {found}")]
    MetadataMismatch {
        kind: &'static str,
        expected: usize,
        found: usize,
    },
    #[error("could not resolve type id `{0}` in registry")]
    UnknownType(u32),
    #[error("unsupported parameter type referenced by `{0}`")]
    UnsupportedType(String),
}

pub type Result<T> = core::result::Result<T, BuildError>;

// Reuse canonical documents per service to avoid repeated SCALE registry traversals.
static CANONICAL_DOC_CACHE: OnceLock<Mutex<BTreeMap<TypeId, CanonicalDocument>>> = OnceLock::new();

impl From<CanonicalTypeError> for BuildError {
    fn from(value: CanonicalTypeError) -> Self {
        match value {
            CanonicalTypeError::UnknownType(id) => BuildError::UnknownType(id),
        }
    }
}

pub fn build_canonical_document<S: ServiceMeta + 'static>() -> Result<CanonicalDocument> {
    let type_id = TypeId::of::<S>();
    let cache = CANONICAL_DOC_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    {
        let cache_guard = cache
            .lock()
            .expect("canonical document cache mutex poisoned");
        if let Some(doc) = cache_guard.get(&type_id) {
            return Ok(doc.clone());
        }
    }

    let meta = AnyServiceMeta::new::<S>();
    let doc = build_canonical_document_from_meta(&meta)?;

    let mut cache_guard = cache
        .lock()
        .expect("canonical document cache mutex poisoned");
    let doc_ref = cache_guard.entry(type_id).or_insert(doc);
    Ok(doc_ref.clone())
}

fn extract_params(type_id: u32, registry: &PortableRegistry) -> Result<Vec<CanonicalParam>> {
    let ty = registry
        .resolve(type_id)
        .ok_or_else(|| BuildError::UnknownType(type_id))?;
    match &ty.type_def {
        TypeDef::Composite(def) => Ok(def
            .fields
            .iter()
            .enumerate()
            .map(|(idx, field)| {
                let name = field
                    .name
                    .clone()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| format!("arg{idx}"));
                let ty = canonical_visitor::canonical_type(registry, field.ty.id)
                    .unwrap_or_else(|_| canonical_visitor::named_type(registry, field.ty.id));
                CanonicalParam { name, ty }
            })
            .collect()),
        _ => Ok(vec![CanonicalParam {
            name: "arg0".to_owned(),
            ty: canonical_visitor::canonical_type(registry, type_id)?,
        }]),
    }
}

fn extract_event_payload(
    variant: &Variant<PortableForm>,
    registry: &PortableRegistry,
) -> Result<Option<CanonicalType>> {
    if let Some(field) = variant.fields.first() {
        let ty = registry
            .resolve(field.ty.id)
            .ok_or_else(|| BuildError::UnknownType(field.ty.id))?;
        match &ty.type_def {
            TypeDef::Tuple(def) if def.fields.is_empty() => Ok(None),
            _ => Ok(Some(canonical_visitor::canonical_type(
                registry,
                field.ty.id,
            )?)),
        }
    } else {
        Ok(None)
    }
}

fn collect_user_type_ids(registry: &PortableRegistry, type_id: u32, acc: &mut BTreeSet<u32>) {
    fn visit(
        registry: &PortableRegistry,
        type_id: u32,
        acc: &mut BTreeSet<u32>,
        visited: &mut BTreeSet<u32>,
    ) {
        if !visited.insert(type_id) {
            return;
        }
        let Some(ty) = registry.resolve(type_id) else {
            return;
        };
        let has_private_name = ty
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.starts_with("__"));
        let has_system_prefix = ty
            .path
            .segments
            .first()
            .is_some_and(|segment| matches!(segment.as_ref(), "alloc" | "core" | "std"));
        let should_include = matches!(ty.type_def, TypeDef::Composite(_) | TypeDef::Variant(_))
            && !has_private_name
            && !has_system_prefix;
        if should_include {
            acc.insert(type_id);
        }
        match &ty.type_def {
            TypeDef::Composite(def) => {
                for field in &def.fields {
                    visit(registry, field.ty.id, acc, visited);
                }
            }
            TypeDef::Variant(def) => {
                for variant in &def.variants {
                    for field in &variant.fields {
                        visit(registry, field.ty.id, acc, visited);
                    }
                }
            }
            TypeDef::Sequence(def) => {
                visit(registry, def.type_param.id, acc, visited);
            }
            TypeDef::Array(def) => {
                visit(registry, def.type_param.id, acc, visited);
            }
            TypeDef::Tuple(def) => {
                for field in &def.fields {
                    visit(registry, field.id, acc, visited);
                }
            }
            TypeDef::Compact(def) => {
                visit(registry, def.type_param.id, acc, visited);
            }
            TypeDef::Primitive(_) | TypeDef::BitSequence(_) => {}
        }
    }

    let mut visited = BTreeSet::new();
    visit(registry, type_id, acc, &mut visited);
}

fn canonical_types_from_ids(
    registry: &PortableRegistry,
    type_ids: &BTreeSet<u32>,
) -> Result<BTreeMap<String, CanonicalType>> {
    let mut types = BTreeMap::new();
    for type_id in type_ids {
        let ty = registry
            .resolve(*type_id)
            .ok_or_else(|| BuildError::UnknownType(*type_id))?;
        let name = if ty.path.segments.is_empty() {
            format!("type_{}", type_id)
        } else {
            ty.path.segments.join("::")
        };
        let canonical = canonical_visitor::canonical_type(registry, *type_id)?;
        types.entry(name).or_insert(canonical);
    }
    Ok(types)
}
fn register_builtin_types(registry: &mut Registry) {
    let _: Vec<_> = registry.register_types([] as [MetaType; 0]);
}

pub fn build_canonical_document_from_meta(meta: &AnyServiceMeta) -> Result<CanonicalDocument> {
    let mut services = BTreeMap::new();
    let mut visited = BTreeSet::new();
    let mut types = BTreeMap::new();
    collect_service(meta, &mut services, &mut visited, &mut types)?;

    Ok(CanonicalDocument {
        version: crate::canonical::CANONICAL_VERSION.to_owned(),
        services,
        types,
    })
}

fn collect_service(
    meta: &AnyServiceMeta,
    services: &mut BTreeMap<String, CanonicalService>,
    visited: &mut BTreeSet<String>,
    collected_types: &mut BTreeMap<String, CanonicalType>,
) -> Result<()> {
    let name = meta.interface_path().to_owned();
    if visited.contains(&name) {
        return Ok(());
    }

    for base in meta.base_services() {
        collect_service(base, services, visited, collected_types)?;
    }

    if services.contains_key(&name) {
        return Ok(());
    }

    visited.insert(name.clone());
    let (service, types) = build_service(meta, services)?;
    for (name, ty) in types {
        collected_types.entry(name).or_insert(ty);
    }
    services.insert(name, service);

    Ok(())
}

fn build_service(
    meta: &AnyServiceMeta,
    services: &BTreeMap<String, CanonicalService>,
) -> Result<(CanonicalService, BTreeMap<String, CanonicalType>)> {
    let mut registry = Registry::new();
    register_builtin_types(&mut registry);

    let command_type_id = registry.register_type(meta.commands()).id;
    let query_type_id = registry.register_type(meta.queries()).id;
    let event_type_id = registry.register_type(meta.events()).id;

    let portable = PortableRegistry::from(registry);
    let mut type_ids = BTreeSet::new();

    let mut functions = Vec::new();
    functions.extend(collect_functions(
        &portable,
        command_type_id,
        meta.local_command_entry_ids(),
        FunctionKind::Command,
        &mut type_ids,
    )?);
    functions.extend(collect_functions(
        &portable,
        query_type_id,
        meta.local_query_entry_ids(),
        FunctionKind::Query,
        &mut type_ids,
    )?);

    let local_event_entry_ids = meta.local_event_entry_ids();
    let mut events = collect_events(
        &portable,
        event_type_id,
        &local_event_entry_ids,
        &mut type_ids,
    )?;

    let mut extends: Vec<CanonicalExtendedInterface> = Vec::new();
    for ext in meta.extends() {
        let name = ext.name.to_owned();
        if let Some(base_service) = services.get(&name) {
            let mut single_services = BTreeMap::new();
            single_services.insert(name.clone(), base_service.clone());
            let single_doc = CanonicalDocument {
                version: crate::canonical::CANONICAL_VERSION.to_owned(),
                services: single_services,
                types: BTreeMap::new(),
            };
            let (interface_id32, interface_uid64) = compute_ids_from_document(&single_doc);
            extends.push(CanonicalExtendedInterface {
                name,
                interface_id32,
                interface_uid64,
                service: Some(Box::new(base_service.clone())),
            });
        } else {
            extends.push(CanonicalExtendedInterface {
                name,
                interface_id32: ext.interface_id32,
                interface_uid64: ext.interface_uid64,
                service: None,
            });
        }
    }

    functions.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.name.cmp(&b.name)));
    extends.sort_by(|a, b| a.name.cmp(&b.name));
    events.sort_by(|a, b| a.name.cmp(&b.name));

    let service = CanonicalService {
        name: meta.interface_path().to_owned(),
        extends,
        functions,
        events,
    };
    let types = canonical_types_from_ids(&portable, &type_ids)?;
    Ok((service, types))
}

fn collect_functions(
    registry: &PortableRegistry,
    type_id: u32,
    entry_ids: &[u16],
    kind: FunctionKind,
    collected_types: &mut BTreeSet<u32>,
) -> Result<Vec<CanonicalFunction>> {
    if entry_ids.is_empty() {
        return Ok(Vec::new());
    }

    let portable = registry
        .resolve(type_id)
        .ok_or_else(|| BuildError::UnknownType(type_id))?;

    let TypeDef::Variant(variant) = &portable.type_def else {
        let kind_name = match kind {
            FunctionKind::Command => "command",
            FunctionKind::Query => "query",
        };
        return Err(BuildError::UnsupportedType(kind_name.to_owned()));
    };

    if variant.variants.len() != entry_ids.len() {
        return Err(BuildError::MetadataMismatch {
            kind: "function",
            expected: variant.variants.len(),
            found: entry_ids.len(),
        });
    }

    let mut functions = Vec::with_capacity(entry_ids.len());
    for (item, entry_id) in variant.variants.iter().zip(entry_ids.iter()) {
        if item.fields.len() != 2 {
            return Err(BuildError::UnsupportedType(item.name.to_string()));
        }
        collect_user_type_ids(registry, item.fields[0].ty.id, collected_types);
        collect_user_type_ids(registry, item.fields[1].ty.id, collected_types);
        let params = extract_params(item.fields[0].ty.id, registry)?;
        let returns = canonical_visitor::canonical_type(registry, item.fields[1].ty.id)?;
        functions.push(CanonicalFunction {
            kind,
            name: item.name.to_string(),
            route: None,
            params,
            returns,
            entry_id_override: Some(*entry_id),
        });
    }

    Ok(functions)
}

fn collect_events(
    registry: &PortableRegistry,
    type_id: u32,
    entry_ids: &[u16],
    collected_types: &mut BTreeSet<u32>,
) -> Result<Vec<CanonicalEvent>> {
    if entry_ids.is_empty() {
        return Ok(Vec::new());
    }

    let portable = registry
        .resolve(type_id)
        .ok_or_else(|| BuildError::UnknownType(type_id))?;

    let TypeDef::Variant(variant) = &portable.type_def else {
        return Err(BuildError::UnsupportedType("events".to_owned()));
    };

    if variant.variants.len() != entry_ids.len() {
        return Err(BuildError::MetadataMismatch {
            kind: "event",
            expected: variant.variants.len(),
            found: entry_ids.len(),
        });
    }

    let mut events = Vec::with_capacity(entry_ids.len());
    for (item, entry_id) in variant.variants.iter().zip(entry_ids.iter()) {
        for field in &item.fields {
            collect_user_type_ids(registry, field.ty.id, collected_types);
        }
        let payload = extract_event_payload(item, registry)?;
        events.push(CanonicalEvent {
            name: item.name.to_string(),
            payload,
            entry_id_override: Some(*entry_id),
        });
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sails_idl_meta::{AnyServiceMeta, AnyServiceMetaFn, ExtendedInterface, ServiceMeta};
    use scale_info::TypeInfo;

    const ROOT_INTERFACE_PATH: &str = "test::RootService";
    const BASE_INTERFACE_PATH: &str = "test::BaseService";
    const DERIVED_INTERFACE_PATH: &str = "test::DerivedService";

    const ROOT_COMMAND_ID: u16 = 0x0100;
    const BASE_COMMAND_ID: u16 = 0x0200;

    static ROOT_COMMAND_ENTRY_IDS: [u16; 1] = [ROOT_COMMAND_ID];
    static BASE_COMMAND_ENTRY_IDS: [u16; 1] = [BASE_COMMAND_ID];
    static EMPTY_ENTRY_IDS: [u16; 0] = [];
    static ROOT_EXTENDS: [ExtendedInterface; 0] = [];
    static BASE_EXTENDS: [ExtendedInterface; 1] = [ExtendedInterface {
        name: ROOT_INTERFACE_PATH,
        interface_id32: 0,
        interface_uid64: 0,
    }];
    static DERIVED_EXTENDS: [ExtendedInterface; 1] = [ExtendedInterface {
        name: BASE_INTERFACE_PATH,
        interface_id32: 0,
        interface_uid64: 0,
    }];

    #[derive(TypeInfo)]
    struct NoParams;

    #[derive(TypeInfo)]
    enum RootCommandsMeta {
        Identify(NoParams, ()),
    }

    #[derive(TypeInfo)]
    enum BaseCommandsMeta {
        MakeSound(NoParams, ()),
    }

    #[derive(TypeInfo)]
    enum EmptyCommandsMeta {}

    #[derive(TypeInfo)]
    enum EmptyQueriesMeta {}

    #[derive(TypeInfo)]
    enum EmptyEventsMeta {}

    struct RootServiceMeta;

    impl ServiceMeta for RootServiceMeta {
        type CommandsMeta = RootCommandsMeta;
        type QueriesMeta = EmptyQueriesMeta;
        type EventsMeta = EmptyEventsMeta;

        const BASE_SERVICES: &'static [AnyServiceMetaFn] = &[];
        const ASYNC: bool = false;
        const INTERFACE_PATH: &'static str = ROOT_INTERFACE_PATH;
        const EXTENDS: &'static [ExtendedInterface] = &ROOT_EXTENDS;

        fn command_entry_ids() -> Vec<u16> {
            ROOT_COMMAND_ENTRY_IDS.to_vec()
        }

        fn local_command_entry_ids() -> &'static [u16] {
            &ROOT_COMMAND_ENTRY_IDS
        }

        fn query_entry_ids() -> Vec<u16> {
            Vec::new()
        }

        fn local_query_entry_ids() -> &'static [u16] {
            &EMPTY_ENTRY_IDS
        }
    }

    struct BaseServiceMeta;

    impl ServiceMeta for BaseServiceMeta {
        type CommandsMeta = BaseCommandsMeta;
        type QueriesMeta = EmptyQueriesMeta;
        type EventsMeta = EmptyEventsMeta;

        const BASE_SERVICES: &'static [AnyServiceMetaFn] =
            &[AnyServiceMeta::new::<RootServiceMeta>];
        const ASYNC: bool = false;
        const INTERFACE_PATH: &'static str = BASE_INTERFACE_PATH;
        const EXTENDS: &'static [ExtendedInterface] = &BASE_EXTENDS;

        fn command_entry_ids() -> Vec<u16> {
            let mut ids = BASE_COMMAND_ENTRY_IDS.to_vec();
            ids.extend(RootServiceMeta::command_entry_ids());
            ids
        }

        fn local_command_entry_ids() -> &'static [u16] {
            &BASE_COMMAND_ENTRY_IDS
        }

        fn query_entry_ids() -> Vec<u16> {
            RootServiceMeta::query_entry_ids()
        }

        fn local_query_entry_ids() -> &'static [u16] {
            &EMPTY_ENTRY_IDS
        }
    }

    struct DerivedServiceMeta;

    impl ServiceMeta for DerivedServiceMeta {
        type CommandsMeta = EmptyCommandsMeta;
        type QueriesMeta = EmptyQueriesMeta;
        type EventsMeta = EmptyEventsMeta;

        const BASE_SERVICES: &'static [AnyServiceMetaFn] =
            &[AnyServiceMeta::new::<BaseServiceMeta>];
        const ASYNC: bool = false;
        const INTERFACE_PATH: &'static str = DERIVED_INTERFACE_PATH;
        const EXTENDS: &'static [ExtendedInterface] = &DERIVED_EXTENDS;

        fn command_entry_ids() -> Vec<u16> {
            BaseServiceMeta::command_entry_ids()
        }

        fn local_command_entry_ids() -> &'static [u16] {
            &EMPTY_ENTRY_IDS
        }

        fn query_entry_ids() -> Vec<u16> {
            BaseServiceMeta::query_entry_ids()
        }

        fn local_query_entry_ids() -> &'static [u16] {
            &EMPTY_ENTRY_IDS
        }
    }

    #[test]
    fn extends_embed_base_services_recursively() {
        let meta = AnyServiceMeta::new::<DerivedServiceMeta>();
        let doc = build_canonical_document_from_meta(&meta)
            .expect("canonical document should be constructed");

        let derived = doc
            .services
            .get(DERIVED_INTERFACE_PATH)
            .expect("derived service exists");
        let base_ext = derived
            .extends
            .iter()
            .find(|ext| ext.name == BASE_INTERFACE_PATH)
            .expect("base extension present");

        assert!(
            base_ext.interface_id32 != 0,
            "interface id should be derived from canonical document"
        );
        let base_service = base_ext
            .service
            .as_ref()
            .expect("embedded base canonical service");
        assert_eq!(
            base_service
                .functions
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            vec!["MakeSound"]
        );
        assert_eq!(
            base_service.functions[0].entry_id_override,
            Some(BASE_COMMAND_ID)
        );

        let root_ext = base_service
            .extends
            .iter()
            .find(|ext| ext.name == ROOT_INTERFACE_PATH)
            .expect("root extension present");
        let root_service = root_ext
            .service
            .as_ref()
            .expect("embedded root canonical service");
        assert_eq!(
            root_service.functions[0].entry_id_override,
            Some(ROOT_COMMAND_ID)
        );
    }
}
