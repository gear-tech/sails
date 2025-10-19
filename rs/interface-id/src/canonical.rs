use sails_idl_parser::ast::{
    EnumVariant, ParseError as IdlParseError, Service, StructField, TypeDecl, TypeDef, parse_idl,
};
use serde::{Deserialize, Serialize, de::Error as _};
use serde_json::{Map, Value};
use serde_json_canonicalizer::to_vec as to_canonical_vec;
use std::collections::BTreeMap;
use std::fmt;

/// Human-readable version identifier for the canonicalization scheme.
pub const CANONICAL_VERSION: &str = "sails-idl-v1-jcs";

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

/// Canonical JSON document containing services and types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalDocument {
    /// Canonicalization version identifier.
    #[serde(default = "default_version")]
    pub version: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_id_override: Option<u16>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_id_override: Option<u16>,
}

/// Canonical description of a parent interface implemented by a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalExtendedInterface {
    pub name: String,
    pub interface_id32: u32,
    pub interface_uid64: u64,
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
    #[serde(skip_serializing_if = "Option::is_none")]
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

        let version = root
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or(CANONICAL_VERSION)
            .to_owned();

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
            version,
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
        for service in program.services() {
            let metadata = extends.get(service.name());
            let mut canonical_service = service_to_canonical(service, metadata);
            canonical_service.normalize();
            services.insert(service.name().to_owned(), canonical_service);
        }

        Ok(Self {
            version: CANONICAL_VERSION.to_owned(),
            services,
            types: BTreeMap::new(),
        })
    }

    fn normalized(mut self) -> Self {
        for service in self.services.values_mut() {
            service.normalize();
        }
        for ty in self.types.values_mut() {
            ty.normalize();
        }
        if self.version.is_empty() {
            self.version = CANONICAL_VERSION.to_owned();
        }
        self
    }
}

impl CanonicalService {
    fn normalize(&mut self) {
        self.extends.sort_by(|a, b| a.name.cmp(&b.name));
        for function in &mut self.functions {
            function.normalize();
        }
        self.functions
            .sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.name.cmp(&b.name)));
        for event in &mut self.events {
            event.normalize();
        }
        self.events.sort_by(|a, b| a.name.cmp(&b.name));
    }
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
                    .map(|field| {
                        let ty = field.ty.to_signature_string();
                        match &field.name {
                            Some(name) => format!("{name}: {ty}"),
                            None => ty,
                        }
                    })
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
    CanonicalService {
        name: service.name().to_owned(),
        extends: metadata.map(|m| m.extends.clone()).unwrap_or_default(),
        functions: service
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
                        ty: CanonicalType::Unit,
                    })
                    .collect(),
                returns: CanonicalType::Unit,
                entry_id_override: extract_u16_marker(func.docs(), "!@entry_id"),
            })
            .collect(),
        events: Vec::new(),
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

    let name = trimmed
        .split('(')
        .next()
        .unwrap_or(trimmed)
        .trim()
        .to_owned();

    Some(CanonicalExtendedInterface {
        name,
        interface_id32: 0,
        interface_uid64: 0,
    })
}

fn extract_u16_marker(docs: &[String], marker: &str) -> Option<u16> {
    docs.iter().find_map(|doc| {
        let trimmed = doc.trim();
        if let Some(idx) = trimmed.find(marker) {
            let after = trimmed[idx + marker.len()..].trim();
            let value_str = after.strip_prefix('=').map(str::trim).unwrap_or(after);
            parse_u16(value_str)
        } else {
            None
        }
    })
}

fn parse_u16(value: &str) -> Option<u16> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u16>().ok()
    }
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
                    "b": {"name":"b","extends":[],"functions":[{"kind":"command","name":"Beta","params":[],"returns":{"kind":"unit"},"entry_id_override":1}],"events":[]},
                    "a": {"name":"a","extends":[],"functions":[{"kind":"command","name":"Alpha","params":[],"returns":{"kind":"unit"},"entry_id_override":1}],"events":[]}
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
        assert_eq!(doc.version, CANONICAL_VERSION);
        let service = doc.services.get("Example").expect("service exists");
        assert_eq!(service.functions.len(), 2);
        assert_eq!(service.functions[0].kind, FunctionKind::Command);
        assert_eq!(service.functions[0].name, "DoSomething");
        assert_eq!(service.functions[0].returns, CanonicalType::Unit);
        assert_eq!(service.functions[1].kind, FunctionKind::Query);
        assert_eq!(service.functions[1].returns, CanonicalType::Unit);

        let bytes = doc.to_bytes().expect("serialization");
        let value: Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(
            value,
            json!({
                "services": {
                    "Example": {
                        "events": [],
                        "extends": [],
                        "functions": [
                            {
                                "kind": "command",
                                "name": "DoSomething",
                                "params": [],
                                "returns": {"kind": "unit"},
                                "entry_id_override": 1
                            },
                            {
                                "kind": "query",
                                "name": "GetValue",
                                "params": [],
                                "returns": {"kind": "unit"},
                                "entry_id_override": 2
                            }
                        ],
                        "name": "Example"
                    }
                },
                "types": {},
                "version": CANONICAL_VERSION,
            })
        );
    }
}
