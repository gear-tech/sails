#![allow(clippy::result_large_err)]

extern crate alloc;

use alloc::{
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
fn register_builtin_types(registry: &mut Registry) {
    let _: Vec<_> = registry.register_types([] as [MetaType; 0]);
}

pub fn build_canonical_document_from_meta(meta: &AnyServiceMeta) -> Result<CanonicalDocument> {
    let mut services = BTreeMap::new();
    let mut visited = BTreeSet::new();
    collect_service(meta, &mut services, &mut visited)?;

    Ok(CanonicalDocument {
        version: crate::canonical::CANONICAL_VERSION.to_owned(),
        services,
        types: BTreeMap::new(),
    })
}

fn collect_service(
    meta: &AnyServiceMeta,
    services: &mut BTreeMap<String, CanonicalService>,
    visited: &mut BTreeSet<String>,
) -> Result<()> {
    let name = meta.interface_path().to_owned();
    if visited.contains(&name) {
        return Ok(());
    }

    for base in meta.base_services() {
        collect_service(base, services, visited)?;
    }

    if services.contains_key(&name) {
        return Ok(());
    }

    visited.insert(name.clone());
    let service = build_service(meta, services)?;
    services.insert(name, service);

    Ok(())
}

fn build_service(
    meta: &AnyServiceMeta,
    services: &BTreeMap<String, CanonicalService>,
) -> Result<CanonicalService> {
    let mut registry = Registry::new();
    register_builtin_types(&mut registry);

    let command_type_id = registry.register_type(meta.commands()).id;
    let query_type_id = registry.register_type(meta.queries()).id;
    let event_type_id = registry.register_type(meta.events()).id;

    let portable = PortableRegistry::from(registry);

    let mut functions = Vec::new();
    functions.extend(collect_functions(
        &portable,
        command_type_id,
        meta.local_command_entry_ids(),
        FunctionKind::Command,
    )?);
    functions.extend(collect_functions(
        &portable,
        query_type_id,
        meta.local_query_entry_ids(),
        FunctionKind::Query,
    )?);

    let local_event_entry_ids = meta.local_event_entry_ids();
    let mut events = collect_events(&portable, event_type_id, &local_event_entry_ids)?;

    let mut extends: Vec<CanonicalExtendedInterface> = meta
        .extends()
        .iter()
        .map(|ext| {
            let name = ext.name.to_owned();
            let (interface_id32, interface_uid64) = if let Some(base_service) = services.get(&name)
            {
                let mut single_services = BTreeMap::new();
                single_services.insert(name.clone(), base_service.clone());
                let single_doc = CanonicalDocument {
                    version: crate::canonical::CANONICAL_VERSION.to_owned(),
                    services: single_services,
                    types: BTreeMap::new(),
                };
                compute_ids_from_document(&single_doc)
            } else {
                (ext.interface_id32, ext.interface_uid64)
            };
            CanonicalExtendedInterface {
                name,
                interface_id32,
                interface_uid64,
            }
        })
        .collect();

    functions.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.name.cmp(&b.name)));
    extends.sort_by(|a, b| a.name.cmp(&b.name));
    events.sort_by(|a, b| a.name.cmp(&b.name));

    let service = CanonicalService {
        name: meta.interface_path().to_owned(),
        extends,
        functions,
        events,
    };
    Ok(service)
}

fn collect_functions(
    registry: &PortableRegistry,
    type_id: u32,
    entry_ids: &[u16],
    kind: FunctionKind,
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
        let payload = extract_event_payload(item, registry)?;
        events.push(CanonicalEvent {
            name: item.name.to_string(),
            payload,
            entry_id_override: Some(*entry_id),
        });
    }

    Ok(events)
}
