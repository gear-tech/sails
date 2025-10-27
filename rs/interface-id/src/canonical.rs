use sails_idl_parser::ast::{
    EnumVariant, ParseError as IdlParseError, Service, StructField, TypeDecl, TypeDef, parse_idl,
};
use serde::{Deserialize, Serialize, de::Error as _};
use serde_json::{Map, Value};
use serde_json_canonicalizer::to_vec as to_canonical_vec;
use std::collections::BTreeMap;
use std::fmt;

/// Canonical schema identifier.
pub const CANONICAL_SCHEMA: &str = crate::CANONICAL_SCHEMA;
/// Canonical schema version identifier.
pub const CANONICAL_VERSION: &str = crate::CANONICAL_VERSION;
/// Canonical hash algorithm identifier.
pub const CANONICAL_HASH_ALGO: &str = crate::CANONICAL_HASH_ALGO;

/// Errors that can occur during IDL canonicalization.
#[derive(Debug)]
pub enum CanonicalizationError {
    /// The provided IDL payload is not valid JSON.
    InvalidJson(serde_json::Error),
    /// Canonicalized payload failed to serialize back into JSON.
    Serialization(serde_json::Error),
    /// Textual IDL parsing failed.
    IdlParse(String),
}

impl fmt::Display for CanonicalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(err) => write!(f, "failed to decode IDL JSON: {err}"),
            Self::Serialization(err) => {
                write!(f, "failed to serialize canonicalized IDL: {err}")
            }
            Self::IdlParse(err) => write!(f, "failed to parse textual IDL: {err}"),
        }
    }
}

impl std::error::Error for CanonicalizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidJson(err) | Self::Serialization(err) => Some(err),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for CanonicalizationError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidJson(value)
    }
}

impl From<IdlParseError> for CanonicalizationError {
    fn from(value: IdlParseError) -> Self {
        Self::IdlParse(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalHashMeta {
    #[serde(default)]
    pub algo: String,
    #[serde(default)]
    pub domain: String,
}

/// Canonical JSON document containing services and types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalDocument {
    #[serde(default = "default_schema")]
    pub canon_schema: String,
    /// Canonicalization version identifier.
    #[serde(default = "default_version", alias = "version")]
    pub canon_version: String,
    #[serde(default = "default_hash_meta")]
    pub hash: CanonicalHashMeta,
    #[serde(default)]
    pub services: BTreeMap<String, CanonicalService>,
    #[serde(default)]
    pub types: BTreeMap<String, CanonicalType>,
}

/// Canonical representation of a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalService {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub extends: Vec<CanonicalExtendedInterface>,
    #[serde(default)]
    pub functions: Vec<CanonicalFunction>,
    #[serde(default)]
    pub events: Vec<CanonicalEvent>,
}

/// Canonical description of a command/query inside a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalFunction {
    pub kind: FunctionKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(default)]
    pub params: Vec<CanonicalParam>,
    pub returns: CanonicalType,
}

/// Function mutability.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FunctionKind {
    Command,
    Query,
}

/// Canonical representation of a function parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalParam {
    #[serde(skip_serializing, default)]
    pub name: String,
    #[serde(rename = "type")]
    pub ty: CanonicalType,
}

/// Canonical representation of an event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalEvent {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<CanonicalType>,
}

/// Canonical description of a parent interface implemented by a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalExtendedInterface {
    pub name: String,
    pub interface_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Box<CanonicalService>>,
}

/// Canonical representation of a type expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanonicalType {
    Primitive {
        name: String,
    },
    Named {
        name: String,
    },
    Optional {
        item: Box<CanonicalType>,
    },
    Vector {
        item: Box<CanonicalType>,
    },
    Array {
        item: Box<CanonicalType>,
        len: u32,
    },
    Map {
        key: Box<CanonicalType>,
        value: Box<CanonicalType>,
    },
    Result {
        ok: Box<CanonicalType>,
        err: Box<CanonicalType>,
    },
    Tuple {
        items: Vec<CanonicalType>,
    },
    Struct {
        fields: Vec<CanonicalStructField>,
    },
    Enum {
        variants: Vec<CanonicalEnumVariant>,
    },
    Unit,
}

/// Canonical representation of a struct field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalStructField {
    #[serde(skip_serializing, default)]
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub ty: CanonicalType,
}

/// Canonical representation of an enum variant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalEnumVariant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<CanonicalType>,
}

/// Canonicalizes a raw IDL payload and returns the resulting JSON bytes.
pub fn canonicalize_to_bytes(input: &str) -> Result<Vec<u8>, CanonicalizationError> {
    canonicalize(input)
}

/// Canonicalizes either JSON or textual IDL input, returning canonical JSON bytes.
pub fn canonicalize(input: &str) -> Result<Vec<u8>, CanonicalizationError> {
    match CanonicalDocument::from_json_str(input) {
        Ok(doc) => doc.to_bytes(),
        Err(CanonicalizationError::InvalidJson(_)) => {
            CanonicalDocument::from_text_idl(input)?.to_bytes()
        }
        Err(err) => Err(err),
    }
}

impl CanonicalDocument {
    /// Canonicalizes a raw JSON string.
    pub fn from_json_str(input: &str) -> Result<Self, CanonicalizationError> {
        let value: Value = serde_json::from_str(input)?;
        Self::from_value(value)
    }

    /// Canonicalizes a JSON value.
    pub fn from_value(value: Value) -> Result<Self, CanonicalizationError> {
        let root = value
            .as_object()
            .cloned()
            .ok_or_else(|| serde_json::Error::custom("expected object at IDL root"))
            .map_err(CanonicalizationError::InvalidJson)?;

        let canon_schema = root
            .get("canon_schema")
            .and_then(Value::as_str)
            .unwrap_or(CANONICAL_SCHEMA)
            .to_owned();

        let canon_version = root
            .get("canon_version")
            .or_else(|| root.get("version"))
            .and_then(Value::as_str)
            .unwrap_or(CANONICAL_VERSION)
            .to_owned();

        let hash = root
            .get("hash")
            .cloned()
            .map(|value| {
                serde_json::from_value::<CanonicalHashMeta>(value)
                    .map_err(CanonicalizationError::InvalidJson)
            })
            .unwrap_or_else(|| Ok(default_hash_meta()))?;

        let services_value = root
            .get("services")
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new()));
        let services = parse_services(services_value)?;

        let types_value = root
            .get("types")
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new()));
        let types = parse_types(types_value)?;

        Ok(Self {
            canon_schema,
            canon_version,
            hash,
            services,
            types,
        }
        .normalized())
    }

    /// Serializes the canonical representation into canonical JSON bytes (RFC 8785).
    pub fn to_bytes(&self) -> Result<Vec<u8>, CanonicalizationError> {
        let value = serde_json::to_value(self).map_err(CanonicalizationError::Serialization)?;
        to_canonical_vec(&value).map_err(CanonicalizationError::Serialization)
    }

    /// Parses a human-readable Sails IDL document and produces its canonical representation.
    pub fn from_text_idl(input: &str) -> Result<Self, CanonicalizationError> {
        let program = parse_idl(input)?;
        let extends = collect_service_metadata(input);

        let mut services = BTreeMap::new();
        let mut types = BTreeMap::new();
        for ty in program.types() {
            let canonical_ty = type_def_to_canonical(ty.def());
            types.insert(ty.name().to_owned(), canonical_ty);
        }

        for service in program.services() {
            let metadata = extends.get(service.name());
            let mut canonical_service = service_to_canonical(service, metadata);
            canonical_service.normalize();
            services.insert(service.name().to_owned(), canonical_service);
        }

        Ok(Self {
            canon_schema: default_schema(),
            canon_version: default_version(),
            hash: default_hash_meta(),
            services,
            types,
        })
    }

    fn normalized(mut self) -> Self {
        for service in self.services.values_mut() {
            service.normalize();
        }
        for ty in self.types.values_mut() {
            ty.normalize();
        }
        if self.canon_schema.is_empty() {
            self.canon_schema = default_schema();
        }
        if self.canon_version.is_empty() {
            self.canon_version = default_version();
        }
        if self.hash.algo.is_empty() {
            self.hash.algo = CANONICAL_HASH_ALGO.to_owned();
        }
        if self.hash.domain.is_empty() {
            self.hash.domain = crate::INTERFACE_HASH_DOMAIN_STR.to_owned();
        }
        self
    }
}

impl CanonicalService {
    fn normalize(&mut self) {
        for extended in &mut self.extends {
            if let Some(service) = &mut extended.service {
                service.normalize();
            }
        }
        self.extends.sort_by(|a, b| a.name.cmp(&b.name));
        for function in &mut self.functions {
            function.normalize();
        }
        // Sort by name only (lexicographic), tie-break by canonical signature
        self.functions.sort_by(|a, b| {
            a.name.cmp(&b.name).then_with(|| {
                // Build canonical signature for tie-breaking
                let sig_a = canonical_function_signature(a);
                let sig_b = canonical_function_signature(b);
                sig_a.cmp(&sig_b)
            })
        });
        for event in &mut self.events {
            event.normalize();
        }
        self.events.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Build a canonical signature string for a function (used for tie-breaking when names match).
fn canonical_function_signature(func: &CanonicalFunction) -> String {
    let kind_str = match func.kind {
        FunctionKind::Command => "command",
        FunctionKind::Query => "query",
    };
    let params = func
        .params
        .iter()
        .map(|p| p.ty.to_signature_string())
        .collect::<Vec<_>>()
        .join(", ");
    let returns = func.returns.to_signature_string();
    format!("{kind_str} {}({params}) -> {returns}", func.name)
}

impl CanonicalFunction {
    fn normalize(&mut self) {
        if let Some(route) = &self.route {
            if route == &self.name {
                self.route = None;
            }
        }
        for param in &mut self.params {
            param.normalize();
        }
        self.returns.normalize();
    }
}

impl CanonicalParam {
    fn normalize(&mut self) {
        self.ty.normalize();
    }
}

impl CanonicalEvent {
    fn normalize(&mut self) {
        if let Some(payload) = &mut self.payload {
            payload.normalize();
        }
    }
}

impl CanonicalType {
    fn normalize(&mut self) {
        match self {
            CanonicalType::Optional { item } | CanonicalType::Vector { item } => item.normalize(),
            CanonicalType::Array { item, .. } => item.normalize(),
            CanonicalType::Map { key, value } => {
                key.normalize();
                value.normalize();
            }
            CanonicalType::Tuple { items } => {
                for item in items {
                    item.normalize();
                }
            }
            CanonicalType::Result { ok, err } => {
                ok.normalize();
                err.normalize();
            }
            CanonicalType::Struct { fields } => {
                for field in fields.iter_mut() {
                    field.normalize();
                }
                if fields.iter().all(|field| field.name.is_some()) {
                    fields.sort_by(|a, b| a.name.as_ref().unwrap().cmp(b.name.as_ref().unwrap()));
                }
            }
            CanonicalType::Enum { variants } => {
                for variant in variants.iter_mut() {
                    variant.normalize();
                }
                variants.sort_by(|a, b| a.name.cmp(&b.name));
            }
            CanonicalType::Primitive { .. } | CanonicalType::Named { .. } | CanonicalType::Unit => {
            }
        }
    }

    /// Produces a canonical textual form suitable for hashing and signatures.
    pub fn to_signature_string(&self) -> String {
        match self {
            CanonicalType::Primitive { name } => name.clone(),
            CanonicalType::Named { name } => name.clone(),
            CanonicalType::Optional { item } => format!("opt {}", item.to_signature_string()),
            CanonicalType::Vector { item } => format!("vec {}", item.to_signature_string()),
            CanonicalType::Array { item, len } => {
                format!("[{};{}]", item.to_signature_string(), len)
            }
            CanonicalType::Map { key, value } => format!(
                "map({}, {})",
                key.to_signature_string(),
                value.to_signature_string()
            ),
            CanonicalType::Result { ok, err } => format!(
                "result ({}, {})",
                ok.to_signature_string(),
                err.to_signature_string()
            ),
            CanonicalType::Tuple { items } => {
                let inner = items
                    .iter()
                    .map(|item| item.to_signature_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({inner})")
            }
            CanonicalType::Struct { fields } => {
                let inner = fields
                    .iter()
                    .map(|field| field.ty.to_signature_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("struct {{{inner}}}")
            }
            CanonicalType::Enum { variants } => {
                let inner = variants
                    .iter()
                    .map(|variant| {
                        if let Some(payload) = &variant.payload {
                            format!("{}: {}", variant.name, payload.to_signature_string())
                        } else {
                            variant.name.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("enum {{{inner}}}")
            }
            CanonicalType::Unit => "null".to_owned(),
        }
    }
}

impl CanonicalStructField {
    fn normalize(&mut self) {
        self.ty.normalize();
    }
}

impl CanonicalEnumVariant {
    fn normalize(&mut self) {
        if let Some(payload) = &mut self.payload {
            payload.normalize();
        }
    }
}

fn default_version() -> String {
    CANONICAL_VERSION.to_owned()
}

fn default_schema() -> String {
    CANONICAL_SCHEMA.to_owned()
}

fn default_hash_meta() -> CanonicalHashMeta {
    CanonicalHashMeta {
        algo: CANONICAL_HASH_ALGO.to_owned(),
        domain: crate::INTERFACE_HASH_DOMAIN_STR.to_owned(),
    }
}

fn parse_services(
    value: Value,
) -> Result<BTreeMap<String, CanonicalService>, CanonicalizationError> {
    let services_map = match value {
        Value::Null => Map::new(),
        Value::Object(map) => map,
        other => {
            return Err(CanonicalizationError::InvalidJson(
                serde_json::Error::custom(format!(
                    "expected services object or null, found {other:?}"
                )),
            ));
        }
    };

    let mut services = BTreeMap::new();
    for (name, service_value) in services_map {
        let mut service: CanonicalService =
            serde_json::from_value(service_value).map_err(CanonicalizationError::InvalidJson)?;
        if service.name.is_empty() {
            service.name = name.clone();
        }
        service.normalize();
        services.insert(name, service);
    }
    Ok(services)
}

fn parse_types(value: Value) -> Result<BTreeMap<String, CanonicalType>, CanonicalizationError> {
    let types_map = match value {
        Value::Null => Map::new(),
        Value::Object(map) => map,
        other => {
            return Err(CanonicalizationError::InvalidJson(
                serde_json::Error::custom(format!(
                    "expected types object or null, found {other:?}"
                )),
            ));
        }
    };

    let mut types = BTreeMap::new();
    for (name, ty_value) in types_map {
        let mut ty: CanonicalType =
            serde_json::from_value(ty_value).map_err(CanonicalizationError::InvalidJson)?;
        ty.normalize();
        types.insert(name, ty);
    }
    Ok(types)
}

fn service_to_canonical(
    service: &Service,
    metadata: Option<&ServiceDocMetadata>,
) -> CanonicalService {
    let functions = service
        .funcs()
        .iter()
        .map(|func| CanonicalFunction {
            kind: if func.is_query() {
                FunctionKind::Query
            } else {
                FunctionKind::Command
            },
            name: func.name().to_owned(),
            route: None,
            params: func
                .params()
                .iter()
                .map(|param| CanonicalParam {
                    name: param.name().to_owned(),
                    ty: type_decl_to_canonical(param.type_decl()),
                })
                .collect(),
            returns: type_decl_to_canonical(func.output()),
        })
        .collect();

    let events = service
        .events()
        .iter()
        .map(|event| CanonicalEvent {
            name: event.name().to_owned(),
            payload: event.type_decl().map(type_decl_to_canonical),
        })
        .collect();

    CanonicalService {
        name: service.name().to_owned(),
        extends: metadata.map(|m| m.extends.clone()).unwrap_or_default(),
        functions,
        events,
    }
}

fn type_decl_to_canonical(ty: &TypeDecl) -> CanonicalType {
    match ty {
        TypeDecl::Vector(inner) => CanonicalType::Vector {
            item: Box::new(type_decl_to_canonical(inner)),
        },
        TypeDecl::Array { item, len } => CanonicalType::Array {
            item: Box::new(type_decl_to_canonical(item)),
            len: *len,
        },
        TypeDecl::Map { key, value } => CanonicalType::Map {
            key: Box::new(type_decl_to_canonical(key)),
            value: Box::new(type_decl_to_canonical(value)),
        },
        TypeDecl::Optional(inner) => CanonicalType::Optional {
            item: Box::new(type_decl_to_canonical(inner)),
        },
        TypeDecl::Result { ok, err } => CanonicalType::Result {
            ok: Box::new(type_decl_to_canonical(ok)),
            err: Box::new(type_decl_to_canonical(err)),
        },
        TypeDecl::Id(id) => match id {
            sails_idl_parser::ast::TypeId::Primitive(p) => CanonicalType::Primitive {
                name: primitive_name(*p).to_owned(),
            },
            sails_idl_parser::ast::TypeId::UserDefined(name) => {
                CanonicalType::Named { name: name.clone() }
            }
        },
        TypeDecl::Def(def) => type_def_to_canonical(def),
    }
}

#[allow(dead_code)]
fn type_def_to_canonical(def: &TypeDef) -> CanonicalType {
    match def {
        TypeDef::Struct(struct_def) => {
            let mut fields = struct_def
                .fields()
                .iter()
                .map(|field| struct_field_to_canonical(field))
                .collect::<Vec<_>>();
            if fields.iter().all(|field| field.name.is_some()) {
                fields.sort_by(|a, b| a.name.as_ref().unwrap().cmp(b.name.as_ref().unwrap()));
            }
            CanonicalType::Struct { fields }
        }
        TypeDef::Enum(enum_def) => {
            let mut variants = enum_def
                .variants()
                .iter()
                .map(|variant| enum_variant_to_canonical(variant))
                .collect::<Vec<_>>();
            variants.sort_by(|a, b| a.name.cmp(&b.name));
            CanonicalType::Enum { variants }
        }
    }
}

#[allow(dead_code)]
fn struct_field_to_canonical(field: &StructField) -> CanonicalStructField {
    CanonicalStructField {
        name: field.name().map(|name| name.to_owned()),
        ty: type_decl_to_canonical(field.type_decl()),
    }
}

#[allow(dead_code)]
fn enum_variant_to_canonical(variant: &EnumVariant) -> CanonicalEnumVariant {
    CanonicalEnumVariant {
        name: variant.name().to_owned(),
        payload: variant.type_decl().map(|ty| type_decl_to_canonical(ty)),
    }
}

fn primitive_name(p: sails_idl_parser::ast::PrimitiveType) -> &'static str {
    use sails_idl_parser::ast::PrimitiveType as P;
    match p {
        P::Null => "null",
        P::Bool => "bool",
        P::Char => "char",
        P::Str => "str",
        P::U8 => "u8",
        P::U16 => "u16",
        P::U32 => "u32",
        P::U64 => "u64",
        P::U128 => "u128",
        P::I8 => "i8",
        P::I16 => "i16",
        P::I32 => "i32",
        P::I64 => "i64",
        P::I128 => "i128",
        P::ActorId => "actor_id",
        P::CodeId => "code_id",
        P::MessageId => "message_id",
        P::H256 => "h256",
        P::U256 => "u256",
        P::H160 => "h160",
        P::NonZeroU8 => "nat8",
        P::NonZeroU16 => "nat16",
        P::NonZeroU32 => "nat32",
        P::NonZeroU64 => "nat64",
        P::NonZeroU128 => "nat128",
        P::NonZeroU256 => "nat256",
    }
}

#[derive(Debug, Default, Clone)]
struct ServiceDocMetadata {
    extends: Vec<CanonicalExtendedInterface>,
}

fn collect_service_metadata(input: &str) -> BTreeMap<String, ServiceDocMetadata> {
    let mut metadata = BTreeMap::<String, ServiceDocMetadata>::new();
    let mut current_service: Option<String> = None;
    let mut brace_depth: i32 = 0;
    let mut in_extends_block = false;

    for line in input.lines() {
        let trimmed = line.trim();

        if current_service.is_none() {
            if let Some(rest) = trimmed.strip_prefix("service ") {
                let raw_name = rest.split_whitespace().next().unwrap_or_default();
                let name = raw_name.trim_end_matches('{').trim_end_matches(';');
                if !name.is_empty() {
                    current_service = Some(name.to_owned());
                    brace_depth = count_brace_delta(trimmed);
                }
            }
            continue;
        }

        let service_name = current_service.clone().unwrap();

        if trimmed.starts_with("///") {
            let comment = trimmed.trim_start_matches('/').trim();

            if comment.starts_with("!@extends") {
                in_extends_block = true;
                metadata.entry(service_name.clone()).or_default();
                continue;
            }

            if in_extends_block {
                if comment.is_empty() || comment.starts_with("!@") {
                    in_extends_block = false;
                    continue;
                }

                if let Some(entry) = parse_extends_entry(comment) {
                    metadata
                        .entry(service_name.clone())
                        .or_default()
                        .extends
                        .push(entry);
                } else {
                    in_extends_block = false;
                }
            }
        } else {
            in_extends_block = false;
        }

        brace_depth += count_brace_delta(trimmed);
        if brace_depth <= 0 {
            if let Some(entry) = metadata.get_mut(&service_name) {
                entry.extends.sort_by(|a, b| a.name.cmp(&b.name));
            }
            current_service = None;
            brace_depth = 0;
            in_extends_block = false;
        }
    }

    metadata
}

fn parse_extends_entry(comment: &str) -> Option<CanonicalExtendedInterface> {
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut name = trimmed;
    let mut interface_id: Option<u64> = None;

    if let Some(start) = trimmed.find('(') {
        name = trimmed[..start].trim();
        if let Some(end) = trimmed[start + 1..].rfind(')') {
            let inner = &trimmed[start + 1..start + 1 + end];
            for part in inner.split(',') {
                let part = part.trim();
                if let Some(value) = part.strip_prefix("interface_id=") {
                    interface_id = parse_u64(value).or(interface_id);
                }
            }
        }
    }

    Some(CanonicalExtendedInterface {
        name: name.to_owned(),
        interface_id: interface_id.unwrap_or(0),
        service: None,
    })
}

#[allow(dead_code)]
fn parse_u32(value: &str) -> Option<u32> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u32>().ok()
    }
}

#[allow(dead_code)]
fn parse_u64(value: &str) -> Option<u64> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u64>().ok()
    }
}

fn count_brace_delta(line: &str) -> i32 {
    let opens = line.chars().filter(|&c| c == '{').count() as i32;
    let closes = line.chars().filter(|&c| c == '}').count() as i32;
    opens - closes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn canonicalize(input: &str) -> CanonicalDocument {
        CanonicalDocument::from_json_str(input).expect("canonicalization succeeds")
    }

    #[test]
    fn ignores_unknown_root_fields() {
        let doc = canonicalize(
            r#"{
                "version": "sails-idl-v1-jcs",
                "services": {
                    "b": {"name":"b","extends":[],"functions":[{"kind":"command","name":"Beta","params":[],"returns":{"kind":"unit"}}],"events":[]},
                    "a": {"name":"a","extends":[],"functions":[{"kind":"command","name":"Alpha","params":[],"returns":{"kind":"unit"}}],"events":[]}
                },
                "metadata": {"version": 1},
                "types": {}
            }"#,
        );

        let canonical_services = doc
            .services
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(canonical_services, vec!["a", "b"]);
        assert!(doc.types.is_empty());
    }

    #[test]
    fn canonicalizes_textual_idl() {
        let idl = r#"
            service Example {
                /// !@entry_id = 0x0001
                DoSomething : () -> bool;
                /// !@entry_id = 0x0002
                query GetValue : () -> u32;
            };
        "#;

        let doc = CanonicalDocument::from_text_idl(idl).expect("textual IDL should be parsed");
        assert_eq!(doc.canon_schema, CANONICAL_SCHEMA);
        assert_eq!(doc.canon_version, CANONICAL_VERSION);
        assert_eq!(doc.hash.algo, CANONICAL_HASH_ALGO);
        assert_eq!(doc.hash.domain, crate::INTERFACE_HASH_DOMAIN_STR);
        let service = doc.services.get("Example").expect("service exists");
        assert_eq!(service.functions.len(), 2);
        assert_eq!(service.functions[0].kind, FunctionKind::Command);
        assert_eq!(service.functions[0].name, "DoSomething");
        assert_eq!(
            service.functions[0].returns,
            CanonicalType::Primitive {
                name: "bool".to_owned()
            }
        );
        assert_eq!(service.functions[1].kind, FunctionKind::Query);
        assert_eq!(
            service.functions[1].returns,
            CanonicalType::Primitive {
                name: "u32".to_owned()
            }
        );

        let bytes = doc.to_bytes().expect("serialization");
        let value: Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(
            value,
            json!({
                "canon_schema": CANONICAL_SCHEMA,
                "canon_version": CANONICAL_VERSION,
                "hash": {
                    "algo": CANONICAL_HASH_ALGO,
                    "domain": crate::INTERFACE_HASH_DOMAIN_STR,
                },
                "services": {
                    "Example": {
                        "events": [],
                        "extends": [],
                        "functions": [
                            {
                                "kind": "command",
                                "name": "DoSomething",
                                "params": [],
                                "returns": {"kind": "primitive", "name": "bool"}
                            },
                            {
                                "kind": "query",
                                "name": "GetValue",
                                "params": [],
                                "returns": {"kind": "primitive", "name": "u32"}
                            }
                        ],
                        "name": "Example"
                    }
                },
                "types": {}
            })
        );
    }

    #[test]
    fn parameter_names_do_not_affect_hash() {
        // Two services with identical signatures but different parameter names
        let service1 = json!({
            "canon_schema": CANONICAL_SCHEMA,
            "canon_version": CANONICAL_VERSION,
            "hash": {
                "algo": CANONICAL_HASH_ALGO,
                "domain": crate::INTERFACE_HASH_DOMAIN_STR,
            },
            "services": {
                "TestService": {
                    "name": "TestService",
                    "extends": [],
                    "functions": [{
                        "kind": "command",
                        "name": "Add",
                        "params": [{
                            "name": "value",
                            "type": {"kind": "primitive", "name": "u32"}
                        }],
                        "returns": {"kind": "unit"}
                    }],
                    "events": []
                }
            },
            "types": {}
        });

        let service2 = json!({
            "canon_schema": CANONICAL_SCHEMA,
            "canon_version": CANONICAL_VERSION,
            "hash": {
                "algo": CANONICAL_HASH_ALGO,
                "domain": crate::INTERFACE_HASH_DOMAIN_STR,
            },
            "services": {
                "TestService": {
                    "name": "TestService",
                    "extends": [],
                    "functions": [{
                        "kind": "command",
                        "name": "Add",
                        "params": [{
                            "name": "num",
                            "type": {"kind": "primitive", "name": "u32"}
                        }],
                        "returns": {"kind": "unit"}
                    }],
                    "events": []
                }
            },
            "types": {}
        });

        let doc1 = CanonicalDocument::from_value(service1).expect("valid document");
        let doc2 = CanonicalDocument::from_value(service2).expect("valid document");

        let bytes1 = doc1.to_bytes().expect("serialization");
        let bytes2 = doc2.to_bytes().expect("serialization");

        // The canonical bytes should be identical despite different parameter names
        assert_eq!(
            bytes1, bytes2,
            "parameter names should not affect canonical hash"
        );

        // Verify that parameter names are NOT in the serialized JSON
        let json_str = String::from_utf8(bytes1).expect("valid utf8");
        assert!(
            !json_str.contains("\"value\""),
            "parameter name 'value' should not be in canonical JSON"
        );
        assert!(
            !json_str.contains("\"num\""),
            "parameter name 'num' should not be in canonical JSON"
        );
    }

    #[test]
    fn functions_tie_break_by_signature_when_names_match() {
        // When function names are identical, tie-break by canonical signature
        let service = json!({
            "canon_schema": CANONICAL_SCHEMA,
            "canon_version": CANONICAL_VERSION,
            "hash": {
                "algo": CANONICAL_HASH_ALGO,
                "domain": crate::INTERFACE_HASH_DOMAIN_STR,
            },
            "services": {
                "TestService": {
                    "name": "TestService",
                    "extends": [],
                    "functions": [
                        {
                            "kind": "command",
                            "name": "Process",
                            "params": [{"type": {"kind": "primitive", "name": "u64"}}],
                            "returns": {"kind": "unit"}
                        },
                        {
                            "kind": "query",
                            "name": "Process",
                            "params": [{"type": {"kind": "primitive", "name": "u32"}}],
                            "returns": {"kind": "unit"}
                        }
                    ],
                    "events": []
                }
            },
            "types": {}
        });

        let doc = CanonicalDocument::from_value(service).expect("valid document");
        let functions = &doc.services["TestService"].functions;

        // Both named "Process" - should be sorted by signature
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "Process");
        assert_eq!(functions[1].name, "Process");

        // Verify they're sorted deterministically (by signature)
        // The exact order depends on signature comparison, but should be stable
        let sig0 = canonical_function_signature(&functions[0]);
        let sig1 = canonical_function_signature(&functions[1]);
        assert!(
            sig0 < sig1,
            "Functions with same name should be ordered by signature"
        );
    }

    #[test]
    fn struct_field_names_do_not_affect_hash() {
        // Struct field names should be excluded like parameter names
        let service1 = json!({
            "canon_schema": CANONICAL_SCHEMA,
            "canon_version": CANONICAL_VERSION,
            "hash": {
                "algo": CANONICAL_HASH_ALGO,
                "domain": crate::INTERFACE_HASH_DOMAIN_STR,
            },
            "services": {
                "TestService": {
                    "name": "TestService",
                    "extends": [],
                    "functions": [{
                        "kind": "command",
                        "name": "Process",
                        "params": [],
                        "returns": {
                            "kind": "struct",
                            "fields": [
                                {
                                    "name": "x",
                                    "type": {"kind": "primitive", "name": "u32"}
                                },
                                {
                                    "name": "y",
                                    "type": {"kind": "primitive", "name": "u64"}
                                }
                            ]
                        }
                    }],
                    "events": []
                }
            },
            "types": {}
        });

        let service2 = json!({
            "canon_schema": CANONICAL_SCHEMA,
            "canon_version": CANONICAL_VERSION,
            "hash": {
                "algo": CANONICAL_HASH_ALGO,
                "domain": crate::INTERFACE_HASH_DOMAIN_STR,
            },
            "services": {
                "TestService": {
                    "name": "TestService",
                    "extends": [],
                    "functions": [{
                        "kind": "command",
                        "name": "Process",
                        "params": [],
                        "returns": {
                            "kind": "struct",
                            "fields": [
                                {
                                    "name": "latitude",
                                    "type": {"kind": "primitive", "name": "u32"}
                                },
                                {
                                    "name": "longitude",
                                    "type": {"kind": "primitive", "name": "u64"}
                                }
                            ]
                        }
                    }],
                    "events": []
                }
            },
            "types": {}
        });

        let doc1 = CanonicalDocument::from_value(service1).expect("valid document");
        let doc2 = CanonicalDocument::from_value(service2).expect("valid document");

        let bytes1 = doc1.to_bytes().expect("serialization");
        let bytes2 = doc2.to_bytes().expect("serialization");

        // The canonical bytes should be identical despite different field names
        assert_eq!(
            bytes1, bytes2,
            "struct field names should not affect canonical hash"
        );

        // Verify that field names are NOT in the serialized JSON
        let json_str = String::from_utf8(bytes1).expect("valid utf8");
        assert!(
            !json_str.contains("\"x\""),
            "field name 'x' should not be in canonical JSON"
        );
        assert!(
            !json_str.contains("\"y\""),
            "field name 'y' should not be in canonical JSON"
        );
        assert!(
            !json_str.contains("\"latitude\""),
            "field name 'latitude' should not be in canonical JSON"
        );
        assert!(
            !json_str.contains("\"longitude\""),
            "field name 'longitude' should not be in canonical JSON"
        );
    }

    #[test]
    fn struct_signature_excludes_field_names() {
        // Struct signature should only include types, not field names
        let struct_type = CanonicalType::Struct {
            fields: vec![
                CanonicalStructField {
                    name: Some("user_id".to_string()),
                    ty: CanonicalType::Primitive {
                        name: "u32".to_string(),
                    },
                },
                CanonicalStructField {
                    name: Some("user_name".to_string()),
                    ty: CanonicalType::Primitive {
                        name: "str".to_string(),
                    },
                },
            ],
        };

        let signature = struct_type.to_signature_string();

        // Should not contain field names
        assert!(
            !signature.contains("user_id"),
            "signature should not contain field names"
        );
        assert!(
            !signature.contains("user_name"),
            "signature should not contain field names"
        );

        // Should contain types
        assert!(signature.contains("u32"), "signature should contain types");
        assert!(signature.contains("str"), "signature should contain types");

        // Expected format: "struct {u32, str}"
        assert_eq!(signature, "struct {u32, str}");
    }
}
