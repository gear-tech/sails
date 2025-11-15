use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, VecDeque},
    format,
    string::{String, ToString},
    vec::Vec,
};
use blake3::Hasher;
use core::fmt;
use serde::Serialize;
use serde_json_canonicalizer as jcs;

use crate::ast::{
    EnumVariant, ServiceEvent, ServiceFunc, ServiceUnit, StructDef, Type, TypeDecl, TypeDef,
};

/// Canonical envelope described in `docs/interface-hashing.md`.
///
/// At this stage the structure mirrors the JSON payload without committing
/// to a particular serializer. The goal is to keep canonicalization pure and
/// independent from any specific output format so the same logic can be reused
/// by proc-macros, CLIs or tests.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CanonicalEnvelope {
    pub canon_schema: &'static str,
    pub canon_version: &'static str,
    pub hash: CanonicalHashSettings,
    pub service: CanonicalService,
    pub types: BTreeMap<String, CanonicalNamedType>,
}

impl Default for CanonicalEnvelope {
    fn default() -> Self {
        Self {
            canon_schema: CanonicalHashSettings::SCHEMA,
            canon_version: CanonicalHashSettings::VERSION,
            hash: CanonicalHashSettings::default(),
            service: CanonicalService::default(),
            types: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalHashSettings {
    pub algo: &'static str,
    pub domain: &'static str,
}

impl CanonicalHashSettings {
    pub const SCHEMA: &'static str = "sails-idl-jcs";
    pub const VERSION: &'static str = "1";
    pub const HASH_DOMAIN: &'static str = "SAILS-IDL/v1/interface-id";
    pub const HASH_ALGO: &'static str = "blake3";
}

impl Default for CanonicalHashSettings {
    fn default() -> Self {
        Self {
            algo: Self::HASH_ALGO,
            domain: Self::HASH_DOMAIN,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CanonicalService {
    pub extends: Vec<CanonicalParent>,
    pub functions: Vec<CanonicalFunction>,
    pub events: Vec<CanonicalEvent>,
}

impl Default for CanonicalService {
    fn default() -> Self {
        Self {
            extends: Vec::new(),
            functions: Vec::new(),
            events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalParent {
    pub interface_id: u64,
    #[serde(skip_serializing)]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CanonicalFunction {
    pub name: String,
    pub kind: CanonicalFunctionKind,
    pub params: Vec<CanonicalType>,
    pub output: CanonicalType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throws: Option<CanonicalType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CanonicalFunctionKind {
    Command,
    Query,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CanonicalEvent {
    pub name: String,
    pub payload: CanonicalAggregate,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CanonicalNamedType {
    pub kind: CanonicalNamedTypeKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanonicalNamedTypeKind {
    Struct { fields: Vec<CanonicalType> },
    Enum { variants: Vec<CanonicalVariant> },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CanonicalVariant {
    pub name: String,
    pub payload: CanonicalAggregate,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CanonicalAggregate {
    pub fields: Vec<CanonicalType>,
}

impl CanonicalAggregate {
    pub fn unit() -> Self {
        Self { fields: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanonicalType {
    Primitive {
        name: &'static str,
    },
    Slice {
        item: Box<CanonicalType>,
    },
    Array {
        item: Box<CanonicalType>,
        len: u32,
    },
    Tuple {
        items: Vec<CanonicalType>,
    },
    Option {
        item: Box<CanonicalType>,
    },
    Result {
        ok: Box<CanonicalType>,
        err: Box<CanonicalType>,
    },
    Named {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<CanonicalType>,
    },
}

#[derive(Clone, Copy)]
pub enum TypeScope<'a> {
    Local,
    Parent(&'a ParentInterface<'a>),
}

impl<'a> TypeScope<'a> {
    fn canonical_key(&self, ty_name: &str) -> String {
        if ty_name.contains("::") {
            ty_name.to_owned()
        } else {
            match self {
                TypeScope::Local => format!("self::{ty_name}"),
                TypeScope::Parent(parent) => {
                    format!("{}::{ty_name}", parent_type_prefix(parent.interface_id))
                }
            }
        }
    }
}

fn parent_type_prefix(interface_id: u64) -> String {
    format!("parent@{interface_id:016x}")
}

#[derive(Default)]
pub struct TypeRegistry<'a> {
    entries: BTreeMap<String, RegisteredType<'a>>,
}

pub struct RegisteredType<'a> {
    pub owner: TypeScope<'a>,
    pub ty: &'a Type,
}

impl<'a> TypeRegistry<'a> {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn insert_local_service(&mut self, service: &'a ServiceUnit) {
        self.insert_with_scope(service, TypeScope::Local);
    }

    pub fn insert_parent_service(&mut self, parent: &'a ParentInterface<'a>) {
        self.insert_with_scope(parent.service, TypeScope::Parent(parent));
    }

    fn insert_with_scope(&mut self, service: &'a ServiceUnit, scope: TypeScope<'a>) {
        for ty in &service.types {
            let key = scope.canonical_key(&ty.name);
            self.entries
                .insert(key, RegisteredType { owner: scope, ty });
        }
    }

    pub fn get(&self, qualified: &str) -> Option<&RegisteredType<'a>> {
        self.entries.get(qualified)
    }

    pub fn contains(&self, qualified: &str) -> bool {
        self.entries.contains_key(qualified)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &RegisteredType<'a>)> {
        self.entries.iter()
    }
}

pub struct TypeResolver<'a, 'b> {
    registry: &'a TypeRegistry<'b>,
    parents: &'a ResolvedParents<'b>,
    local_service_name: &'b str,
}

impl<'a, 'b> TypeResolver<'a, 'b> {
    pub fn new(
        registry: &'a TypeRegistry<'b>,
        parents: &'a ResolvedParents<'b>,
        local_service_name: &'b str,
    ) -> Self {
        Self {
            registry,
            parents,
            local_service_name,
        }
    }

    pub fn canonical_type(
        &self,
        ty: &TypeDecl,
        scope: TypeScope<'b>,
    ) -> Result<CanonicalType, CanonicalError> {
        Ok(match ty {
            TypeDecl::Primitive(primitive) => CanonicalType::Primitive {
                name: primitive.as_str(),
            },
            TypeDecl::Slice(inner) => CanonicalType::Slice {
                item: Box::new(self.canonical_type(inner, scope)?),
            },
            TypeDecl::Array { item, len } => CanonicalType::Array {
                item: Box::new(self.canonical_type(item, scope)?),
                len: *len,
            },
            TypeDecl::Tuple(items) => CanonicalType::Tuple {
                items: items
                    .iter()
                    .map(|item| self.canonical_type(item, scope))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            TypeDecl::Option(inner) => CanonicalType::Option {
                item: Box::new(self.canonical_type(inner, scope)?),
            },
            TypeDecl::Result { ok, err } => CanonicalType::Result {
                ok: Box::new(self.canonical_type(ok, scope)?),
                err: Box::new(self.canonical_type(err, scope)?),
            },
            TypeDecl::UserDefined { name, generics } => {
                let qualified = self.qualify_name(name, scope)?;
                CanonicalType::Named {
                    name: qualified,
                    args: generics
                        .iter()
                        .map(|arg| self.canonical_type(arg, scope))
                        .collect::<Result<Vec<_>, _>>()?,
                }
            }
            TypeDecl::Generic(name) => {
                return Err(CanonicalError::UnsupportedGenericParameter(name.clone()));
            }
        })
    }

    pub fn qualify_name(&self, raw: &str, scope: TypeScope<'b>) -> Result<String, CanonicalError> {
        if let Some(candidate) = self.try_absolute_name(raw) {
            return Ok(candidate);
        }

        if let Some(candidate) = self.try_scoped_name(raw, scope) {
            return Ok(candidate);
        }

        for (_, parent) in self.parents.iter() {
            if let Some(candidate) = self.try_scoped_name(raw, TypeScope::Parent(parent)) {
                return Ok(candidate);
            }
        }

        Err(CanonicalError::UnknownType(raw.to_owned()))
    }

    fn try_absolute_name(&self, raw: &str) -> Option<String> {
        if raw.contains("::") && self.registry.contains(raw) {
            Some(raw.to_owned())
        } else {
            None
        }
    }

    fn try_scoped_name(&self, raw: &str, scope: TypeScope<'b>) -> Option<String> {
        let candidate = if raw.contains("::") {
            let remainder = self.strip_scope_prefix(raw, scope)?;
            scope.canonical_key(remainder)
        } else {
            scope.canonical_key(raw)
        };

        if self.registry.contains(&candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    fn strip_scope_prefix<'c>(&self, raw: &'c str, scope: TypeScope<'b>) -> Option<&'c str> {
        let (prefix, needs_separator) = match scope {
            TypeScope::Local => (self.local_service_name, true),
            TypeScope::Parent(parent) => (parent.name, true),
        };
        raw.strip_prefix(prefix).and_then(|rest| {
            if needs_separator {
                rest.strip_prefix("::")
            } else {
                Some(rest)
            }
        })
    }
}

/// Optional context for canonicalization, allowing the caller to supply
/// interface IDs for parent services and additional lookup facilities.
pub type ParentResolver<'a> = dyn Fn(&str) -> Option<ParentInterface<'a>> + 'a;

#[derive(Debug, Clone, PartialEq)]
pub struct ParentInterface<'a> {
    pub name: &'a str,
    pub interface_id: u64,
    pub service: &'a ServiceUnit,
}

impl<'a> ParentInterface<'a> {
    pub fn new(service: &'a ServiceUnit, interface_id: u64) -> Self {
        Self {
            name: service.name.as_str(),
            interface_id,
            service,
        }
    }
}

pub struct CanonicalizationContext<'a> {
    pub parent_interfaces: &'a [ParentInterface<'a>],
    pub parent_resolver: Option<&'a ParentResolver<'a>>,
}

impl<'a> fmt::Debug for CanonicalizationContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanonicalizationContext")
            .field("parent_interfaces", &self.parent_interfaces)
            .field(
                "parent_resolver",
                &self
                    .parent_resolver
                    .as_ref()
                    .map(|_| "<resolver>")
                    .unwrap_or("<none>"),
            )
            .finish()
    }
}

impl<'a> Default for CanonicalizationContext<'a> {
    fn default() -> Self {
        Self {
            parent_interfaces: &[],
            parent_resolver: None,
        }
    }
}

impl<'a> CanonicalizationContext<'a> {
    pub fn with_parents(parents: &'a [ParentInterface<'a>]) -> Self {
        Self {
            parent_interfaces: parents,
            parent_resolver: None,
        }
    }

    pub fn with_resolver(resolver: &'a ParentResolver<'a>) -> Self {
        Self {
            parent_interfaces: &[],
            parent_resolver: Some(resolver),
        }
    }

    pub fn resolve_parent(&self, name: &str) -> Option<ParentInterface<'a>> {
        self.parent_interfaces
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .or_else(|| self.parent_resolver.and_then(|resolver| resolver(name)))
    }
}

/// Canonicalization error.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CanonicalError {
    #[error("missing interface id for parent `{0}`")]
    MissingParentInterface(String),
    #[error("cyclic extends detected at `{0}`")]
    CyclicExtends(String),
    #[error("unknown type `{0}`")]
    UnknownType(String),
    #[error("unsupported generic parameter `{0}`")]
    UnsupportedGenericParameter(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Fully resolved view of all parent services reachable via `extends`.
pub struct ResolvedParents<'a> {
    map: BTreeMap<String, ParentInterface<'a>>,
}

impl<'a> ResolvedParents<'a> {
    pub fn new(map: BTreeMap<String, ParentInterface<'a>>) -> Self {
        Self { map }
    }

    pub fn as_map(&self) -> &BTreeMap<String, ParentInterface<'a>> {
        &self.map
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParentInterface<'a>)> {
        self.map.iter()
    }
}

/// Public entry point for turning a service AST into its canonical envelope.
pub fn canonicalize_service(
    service: &ServiceUnit,
    ctx: &CanonicalizationContext,
) -> Result<CanonicalEnvelope, CanonicalError> {
    let resolved_parents = resolve_parents(service, ctx)?;
    let mut registry = TypeRegistry::new();
    registry.insert_local_service(service);
    for (_, parent) in resolved_parents.iter() {
        registry.insert_parent_service(parent);
    }

    let type_resolver = TypeResolver::new(&registry, &resolved_parents, service.name.as_str());
    let extends = canonicalize_extends(resolved_parents.as_map());
    let functions = canonicalize_functions(&service.funcs, TypeScope::Local, &type_resolver)?;
    let events = canonicalize_events(&service.events, TypeScope::Local, &type_resolver)?;
    let reachable = collect_reachable_types(service, &resolved_parents, &registry, &type_resolver)?;
    let mut types = BTreeMap::new();
    for qualified in reachable {
        if let Some(entry) = registry.get(&qualified) {
            let canonical = canonicalize_named_type(entry.ty, entry.owner, &type_resolver)?;
            types.insert(qualified, canonical);
        }
    }

    Ok(CanonicalEnvelope {
        service: CanonicalService {
            extends,
            functions,
            events,
        },
        types,
        ..CanonicalEnvelope::default()
    })
}

pub struct InterfaceIdResult {
    pub envelope: CanonicalEnvelope,
    pub canonical_json: Vec<u8>,
    pub interface_id: u64,
}

pub fn compute_interface_id(
    service: &ServiceUnit,
    ctx: &CanonicalizationContext,
) -> Result<InterfaceIdResult, CanonicalError> {
    let envelope = canonicalize_service(service, ctx)?;
    let canonical_json =
        jcs::to_vec(&envelope).map_err(|err| CanonicalError::Serialization(err.to_string()))?;
    let mut hasher = Hasher::new();
    hasher.update(CanonicalHashSettings::HASH_DOMAIN.as_bytes());
    hasher.update(&canonical_json);
    let digest = hasher.finalize();
    let mut id_bytes = [0u8; 8];
    id_bytes.copy_from_slice(&digest.as_bytes()[..8]);
    let interface_id = u64::from_le_bytes(id_bytes);

    Ok(InterfaceIdResult {
        envelope,
        canonical_json,
        interface_id,
    })
}

fn canonicalize_extends(
    parent_map: &BTreeMap<String, ParentInterface<'_>>,
) -> Vec<CanonicalParent> {
    let mut entries: Vec<_> = parent_map.iter().collect();
    entries.sort_by_key(|(_, parent)| parent.interface_id);
    entries
        .into_iter()
        .map(|(name, parent)| CanonicalParent {
            interface_id: parent.interface_id,
            name: name.clone(),
        })
        .collect()
}

pub fn canonicalize_parent_types(
    parents: &ResolvedParents,
    resolver: &TypeResolver,
) -> Result<BTreeMap<String, CanonicalNamedType>, CanonicalError> {
    let mut types = BTreeMap::new();
    for (_, parent) in parents.iter() {
        for ty in &parent.service.types {
            let qualified = TypeScope::Parent(parent).canonical_key(&ty.name);
            if types.contains_key(&qualified) {
                continue;
            }
            let canonical = canonicalize_named_type(ty, TypeScope::Parent(parent), resolver)?;
            types.insert(qualified, canonical);
        }
    }
    Ok(types)
}

fn canonicalize_functions(
    funcs: &[ServiceFunc],
    scope: TypeScope,
    resolver: &TypeResolver,
) -> Result<Vec<CanonicalFunction>, CanonicalError> {
    let mut canonicalized = Vec::with_capacity(funcs.len());
    for func in funcs {
        let params = func
            .params
            .iter()
            .map(|param| resolver.canonical_type(&param.type_decl, scope))
            .collect::<Result<Vec<_>, _>>()?;
        let output = resolver.canonical_type(&func.output, scope)?;
        let throws = match &func.throws {
            Some(ty) => Some(resolver.canonical_type(ty, scope)?),
            None => None,
        };

        canonicalized.push(CanonicalFunction {
            name: func.name.clone(),
            kind: if func.is_query {
                CanonicalFunctionKind::Query
            } else {
                CanonicalFunctionKind::Command
            },
            params,
            output,
            throws,
        });
    }

    canonicalized.sort_by(|lhs, rhs| {
        canonical_function_sort_key(lhs).cmp(&canonical_function_sort_key(rhs))
    });
    Ok(canonicalized)
}

fn canonicalize_events(
    events: &[ServiceEvent],
    scope: TypeScope,
    resolver: &TypeResolver,
) -> Result<Vec<CanonicalEvent>, CanonicalError> {
    let mut canonicalized = Vec::with_capacity(events.len());
    for event in events {
        canonicalized.push(CanonicalEvent {
            name: event.name.clone(),
            payload: canonicalize_aggregate(&event.def, scope, resolver)?,
        });
    }

    canonicalized
        .sort_by(|lhs, rhs| canonical_event_sort_key(lhs).cmp(&canonical_event_sort_key(rhs)));
    Ok(canonicalized)
}

fn canonicalize_named_type(
    ty: &Type,
    scope: TypeScope,
    resolver: &TypeResolver,
) -> Result<CanonicalNamedType, CanonicalError> {
    if !ty.type_params.is_empty() {
        return Err(CanonicalError::UnsupportedGenericParameter(ty.name.clone()));
    }

    let kind = match &ty.def {
        TypeDef::Struct(def) => CanonicalNamedTypeKind::Struct {
            fields: canonicalize_aggregate(def, scope, resolver)?.fields,
        },
        TypeDef::Enum(enum_def) => {
            let mut variants = enum_def
                .variants
                .iter()
                .map(|variant| canonicalize_variant(variant, scope, resolver))
                .collect::<Result<Vec<_>, _>>()?;
            variants.sort_by(|lhs, rhs| {
                canonical_variant_sort_key(lhs).cmp(&canonical_variant_sort_key(rhs))
            });
            CanonicalNamedTypeKind::Enum { variants }
        }
    };

    Ok(CanonicalNamedType { kind })
}

fn canonicalize_variant(
    variant: &EnumVariant,
    scope: TypeScope,
    resolver: &TypeResolver,
) -> Result<CanonicalVariant, CanonicalError> {
    Ok(CanonicalVariant {
        name: variant.name.clone(),
        payload: canonicalize_aggregate(&variant.def, scope, resolver)?,
    })
}

fn canonicalize_aggregate(
    def: &StructDef,
    scope: TypeScope,
    resolver: &TypeResolver,
) -> Result<CanonicalAggregate, CanonicalError> {
    let fields = def
        .fields
        .iter()
        .map(|field| resolver.canonical_type(&field.type_decl, scope))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CanonicalAggregate { fields })
}

pub fn resolve_parents<'a>(
    service: &ServiceUnit,
    ctx: &CanonicalizationContext<'a>,
) -> Result<ResolvedParents<'a>, CanonicalError> {
    let mut resolved = BTreeMap::new();
    let mut visiting = BTreeSet::new();
    for parent in &service.extends {
        collect_parent_recursive(parent, ctx, &mut visiting, &mut resolved)?;
    }
    Ok(ResolvedParents::new(resolved))
}

fn collect_parent_recursive<'a>(
    name: &str,
    ctx: &CanonicalizationContext<'a>,
    visiting: &mut BTreeSet<String>,
    resolved: &mut BTreeMap<String, ParentInterface<'a>>,
) -> Result<(), CanonicalError> {
    if !visiting.insert(name.to_owned()) {
        return Err(CanonicalError::CyclicExtends(name.to_owned()));
    }

    let parent = ctx
        .resolve_parent(name)
        .ok_or_else(|| CanonicalError::MissingParentInterface(name.to_owned()))?;

    for ancestor in &parent.service.extends {
        collect_parent_recursive(ancestor, ctx, visiting, resolved)?;
    }

    resolved.entry(parent.name.to_owned()).or_insert(parent);
    visiting.remove(name);
    Ok(())
}

fn canonical_function_sort_key(func: &CanonicalFunction) -> (String, String) {
    let mut signature = String::new();
    signature.push_str(match func.kind {
        CanonicalFunctionKind::Command => "command",
        CanonicalFunctionKind::Query => "query",
    });
    signature.push('|');
    signature.push_str(&join_canonical_type_list(&func.params));
    signature.push('|');
    signature.push_str(&canonical_type_repr(&func.output));
    if let Some(throws) = &func.throws {
        signature.push('|');
        signature.push_str(&canonical_type_repr(throws));
    }

    (func.name.clone(), signature)
}

fn canonical_event_sort_key(event: &CanonicalEvent) -> (String, String) {
    let signature = join_canonical_type_list(&event.payload.fields);
    (event.name.clone(), signature)
}

fn canonical_variant_sort_key(variant: &CanonicalVariant) -> (String, String) {
    let signature = join_canonical_type_list(&variant.payload.fields);
    (variant.name.clone(), signature)
}

fn join_canonical_type_list(types: &[CanonicalType]) -> String {
    let mut acc = String::new();
    let mut first = true;
    for ty in types {
        if !first {
            acc.push(',');
        }
        first = false;
        acc.push_str(&canonical_type_repr(ty));
    }
    acc
}

fn canonical_type_repr(ty: &CanonicalType) -> String {
    match ty {
        CanonicalType::Primitive { name } => name.to_string(),
        CanonicalType::Slice { item } => {
            format!("[{}]", canonical_type_repr(item))
        }
        CanonicalType::Array { item, len } => {
            format!("[{}; {len}]", canonical_type_repr(item))
        }
        CanonicalType::Tuple { items } => {
            let mut repr = String::from("(");
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    repr.push_str(", ");
                }
                repr.push_str(&canonical_type_repr(item));
            }
            repr.push(')');
            repr
        }
        CanonicalType::Option { item } => {
            format!("Option<{}>", canonical_type_repr(item))
        }
        CanonicalType::Result { ok, err } => format!(
            "Result<{}, {}>",
            canonical_type_repr(ok),
            canonical_type_repr(err)
        ),
        CanonicalType::Named { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                let mut repr = String::new();
                repr.push_str(name);
                repr.push('<');
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        repr.push_str(", ");
                    }
                    repr.push_str(&canonical_type_repr(arg));
                }
                repr.push('>');
                repr
            }
        }
    }
}

fn collect_reachable_types<'a>(
    service: &'a ServiceUnit,
    parents: &'a ResolvedParents<'a>,
    registry: &TypeRegistry<'a>,
    resolver: &TypeResolver<'_, 'a>,
) -> Result<BTreeSet<String>, CanonicalError> {
    let mut reachable = BTreeSet::new();
    let mut pending = VecDeque::new();

    fn visit_decl<'a>(
        decl: &TypeDecl,
        scope: TypeScope<'a>,
        resolver: &TypeResolver<'_, 'a>,
        reachable: &mut BTreeSet<String>,
        pending: &mut VecDeque<String>,
    ) -> Result<(), CanonicalError> {
        match decl {
            TypeDecl::Slice(inner) | TypeDecl::Option(inner) => {
                visit_decl(inner, scope, resolver, reachable, pending)?
            }
            TypeDecl::Array { item, .. } => visit_decl(item, scope, resolver, reachable, pending)?,
            TypeDecl::Tuple(items) => {
                for item in items {
                    visit_decl(item, scope, resolver, reachable, pending)?;
                }
            }
            TypeDecl::Result { ok, err } => {
                visit_decl(ok, scope, resolver, reachable, pending)?;
                visit_decl(err, scope, resolver, reachable, pending)?;
            }
            TypeDecl::UserDefined { name, generics } => {
                let qualified = resolver.qualify_name(name, scope)?;
                if reachable.insert(qualified.clone()) {
                    pending.push_back(qualified);
                }
                for arg in generics {
                    visit_decl(arg, scope, resolver, reachable, pending)?;
                }
            }
            TypeDecl::Generic(name) => {
                return Err(CanonicalError::UnsupportedGenericParameter(name.clone()));
            }
            TypeDecl::Primitive(_) => {}
        }
        Ok(())
    }

    let mut process_service =
        |unit: &ServiceUnit, scope: TypeScope<'a>| -> Result<(), CanonicalError> {
            for func in &unit.funcs {
                for param in &func.params {
                    visit_decl(
                        &param.type_decl,
                        scope,
                        resolver,
                        &mut reachable,
                        &mut pending,
                    )?;
                }
                visit_decl(&func.output, scope, resolver, &mut reachable, &mut pending)?;
                if let Some(throws) = &func.throws {
                    visit_decl(throws, scope, resolver, &mut reachable, &mut pending)?;
                }
            }
            for event in &unit.events {
                for field in &event.def.fields {
                    visit_decl(
                        &field.type_decl,
                        scope,
                        resolver,
                        &mut reachable,
                        &mut pending,
                    )?;
                }
            }
            Ok(())
        };

    process_service(service, TypeScope::Local)?;
    for (_, parent) in parents.iter() {
        process_service(parent.service, TypeScope::Parent(parent))?;
    }

    while let Some(name) = pending.pop_front() {
        if let Some(entry) = registry.get(&name) {
            match &entry.ty.def {
                TypeDef::Struct(def) => {
                    for field in &def.fields {
                        visit_decl(
                            &field.type_decl,
                            entry.owner,
                            resolver,
                            &mut reachable,
                            &mut pending,
                        )?;
                    }
                }
                TypeDef::Enum(def) => {
                    for variant in &def.variants {
                        for field in &variant.def.fields {
                            visit_decl(
                                &field.type_decl,
                                entry.owner,
                                resolver,
                                &mut reachable,
                                &mut pending,
                            )?;
                        }
                    }
                }
            }
        }
    }

    Ok(reachable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{vec, vec::Vec};

    fn unit_struct(name: &str) -> Type {
        Type {
            name: name.to_string(),
            type_params: Vec::new(),
            def: TypeDef::Struct(StructDef { fields: Vec::new() }),
            docs: Vec::new(),
            annotations: Vec::new(),
        }
    }

    fn service_unit(name: &str, extends: &[&str], types: Vec<Type>) -> ServiceUnit {
        ServiceUnit {
            name: name.to_string(),
            extends: extends.iter().map(|s| s.to_string()).collect(),
            funcs: Vec::new(),
            events: Vec::new(),
            types,
            docs: Vec::new(),
            annotations: Vec::new(),
        }
    }

    #[test]
    fn skips_unreferenced_parent_types() {
        let base = service_unit("Base", &[], vec![unit_struct("Foo")]);
        let mid = service_unit("Mid", &["Base"], vec![unit_struct("Bar")]);
        let child = service_unit("Child", &["Mid"], vec![unit_struct("Baz")]);

        let resolver = |name: &str| -> Option<ParentInterface<'_>> {
            match name {
                "Base" => Some(ParentInterface::new(&base, 10)),
                "Mid" => Some(ParentInterface::new(&mid, 20)),
                _ => None,
            }
        };

        let ctx = CanonicalizationContext::with_resolver(&resolver);
        let envelope = canonicalize_service(&child, &ctx).expect("canonicalization should succeed");

        let extend_names: Vec<_> = envelope
            .service
            .extends
            .iter()
            .map(|parent| parent.name.as_str())
            .collect();
        assert_eq!(extend_names, vec!["Base", "Mid"]);
        assert_eq!(envelope.service.extends[0].interface_id, 10);
        assert_eq!(envelope.service.extends[1].interface_id, 20);

        // No types are referenced by the service surface, so the canonical envelope
        // should not contain any type definitions.
        assert!(envelope.types.is_empty());
    }

    #[test]
    fn detects_cycles_in_parent_hierarchy() {
        let looping = service_unit("Loop", &["Loop"], Vec::new());
        let child = service_unit("Child", &["Loop"], Vec::new());

        let resolver = |name: &str| -> Option<ParentInterface<'_>> {
            match name {
                "Loop" => Some(ParentInterface::new(&looping, 42)),
                _ => None,
            }
        };

        let ctx = CanonicalizationContext::with_resolver(&resolver);
        let err = canonicalize_service(&child, &ctx).expect_err("cycle must fail");
        assert!(matches!(
            err,
            CanonicalError::CyclicExtends(ref name) if name == "Loop"
        ));
    }

    #[test]
    fn resolved_parents_exposes_types() {
        let base = service_unit("Base", &[], vec![unit_struct("Foo")]);
        let child = service_unit("Child", &["Base"], Vec::new());

        let resolver = |name: &str| -> Option<ParentInterface<'_>> {
            match name {
                "Base" => Some(ParentInterface::new(&base, 7)),
                _ => None,
            }
        };

        let ctx = CanonicalizationContext::with_resolver(&resolver);
        let resolved = resolve_parents(&child, &ctx).expect("parents resolved");
        let mut registry = TypeRegistry::new();
        for (_, parent) in resolved.iter() {
            registry.insert_parent_service(parent);
        }
        let type_resolver = TypeResolver::new(&registry, &resolved, child.name.as_str());
        let inherited = canonicalize_parent_types(&resolved, &type_resolver).expect("types merged");

        assert_eq!(resolved.iter().count(), 1);
        let key = TypeScope::Parent(resolved.iter().next().unwrap().1).canonical_key("Foo");
        assert!(inherited.contains_key(&key));
    }
}
