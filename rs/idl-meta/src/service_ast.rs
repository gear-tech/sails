//! Utilities for reconstructing `ast::ServiceUnit` from `scale-info` metadata emitted by
//! `ServiceMeta` implementors. This gives proc-macros and build tools a direct path from the
//! metadata they already emit to the canonical AST used by the hashing pipeline.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use scale_info::{
    Field, MetaType, Path, PortableRegistry, Registry, Type, TypeDef, TypeDefArray,
    TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
    form::PortableForm,
};

use crate::ast::{
    EnumDef, EnumVariant, FuncParam, PrimitiveType, ServiceFunc, ServiceUnit, StructDef,
    StructField, Type as AstType, TypeDecl, TypeDef as AstTypeDef,
    TypeParameter as AstTypeParameter,
};
use crate::{AnyServiceMeta, ServiceMeta};

#[derive(Debug, thiserror::Error)]
pub enum ServiceAstError {
    #[error("missing type information for id {0}")]
    MissingType(u32),
    #[error("invalid function metadata: {0}")]
    InvalidFunction(&'static str),
    #[error("invalid events metadata: {0}")]
    InvalidEvent(&'static str),
}

pub fn service_unit_from_meta<S: ServiceMeta>(
    service_name: &str,
) -> Result<ServiceUnit, ServiceAstError> {
    let meta = AnyServiceMeta::new::<S>();
    // `ServiceMeta` exposes `scale_info::MetaType` descriptors for commands, queries, and events.
    // We register those types into a temporary `PortableRegistry`, then let `TypeConverter`
    // translate the portable representations into our AST equivalents.
    build_service_unit(service_name, &meta)
}

pub fn build_service_unit(
    name: &str,
    meta: &AnyServiceMeta,
) -> Result<ServiceUnit, ServiceAstError> {
    let mut registry = Registry::new();
    let commands_id = register_meta_type(meta.commands(), &mut registry);
    let queries_id = register_meta_type(meta.queries(), &mut registry);
    let events_id = register_meta_type(meta.events(), &mut registry);

    let portable = PortableRegistry::from(registry);
    let mut converter = TypeConverter::new(portable);

    let commands = collect_service_funcs(&mut converter, commands_id, false)?;
    let queries = collect_service_funcs(&mut converter, queries_id, true)?;
    // Event metadata is also represented as a SCALE variant, so we convert it the same way.
    let events = collect_events(&mut converter, events_id)?;

    let types = converter.into_types();
    let extends = meta
        .base_services()
        .map(|base| short_type_name(base.type_name()).to_string())
        .collect();

    Ok(ServiceUnit {
        name: name.to_string(),
        extends,
        funcs: commands.into_iter().chain(queries.into_iter()).collect(),
        events,
        types,
        docs: Vec::new(),
        annotations: Vec::new(),
    })
}

fn register_meta_type(meta: &MetaType, registry: &mut Registry) -> u32 {
    registry.register_type(meta).id
}

fn collect_service_funcs(
    converter: &mut TypeConverter,
    type_id: u32,
    is_query: bool,
) -> Result<Vec<ServiceFunc>, ServiceAstError> {
    // `TypeConverter::resolve_variant` returns the portable variants for the SCALE enum that
    // `ServiceMeta` emitted. Each variant corresponds to a service entry, with the first field
    // describing the parameters composite and the second describing the return/throws bundle.
    let variants = converter.resolve_variant(type_id)?;
    let mut funcs = Vec::new();

    for variant in variants {
        if variant.fields.len() != 2 {
            return Err(ServiceAstError::InvalidFunction("unexpected fields length"));
        }
        let params_fields = converter.resolve_composite_fields(variant.fields[0].ty.id)?;
        let params = params_fields
            .iter()
            .enumerate()
            .map(|(idx, field)| {
                let name = field
                    .name
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| format!("arg_{idx}"));
                Ok(FuncParam {
                    name,
                    type_decl: converter.type_decl(field.ty.id)?,
                })
            })
            .collect::<Result<Vec<_>, ServiceAstError>>()?;

        let mut output = converter.type_decl(variant.fields[1].ty.id)?;
        let mut throws = None;
        if let TypeDecl::Result { ok, err } = output {
            output = *ok;
            throws = Some(*err);
        }

        funcs.push(ServiceFunc {
            name: variant.name.to_string(),
            params,
            output,
            throws,
            is_query,
            docs: convert_docs(&variant.docs),
            annotations: Vec::new(),
        });
    }

    Ok(funcs)
}

fn collect_events(
    converter: &mut TypeConverter,
    type_id: u32,
) -> Result<Vec<crate::ast::ServiceEvent>, ServiceAstError> {
    let variants = converter.resolve_variant(type_id)?;
    let mut events = Vec::new();
    for variant in variants {
        let payload = StructDef {
            fields: variant
                .fields
                .iter()
                .map(|field| {
                    Ok(StructField {
                        name: field.name.as_ref().map(|n| n.to_string()),
                        type_decl: converter.type_decl(field.ty.id)?,
                        docs: convert_docs(&field.docs),
                        annotations: Vec::new(),
                    })
                })
                .collect::<Result<Vec<_>, ServiceAstError>>()?,
        };
        events.push(EnumVariant {
            name: variant.name.to_string(),
            def: payload,
            docs: convert_docs(&variant.docs),
            annotations: Vec::new(),
        });
    }
    Ok(events)
}

struct TypeConverter {
    registry: PortableRegistry,
    types: BTreeMap<String, AstType>,
    resolving: BTreeSet<u32>,
}

impl TypeConverter {
    fn new(registry: PortableRegistry) -> Self {
        Self {
            registry,
            types: BTreeMap::new(),
            resolving: BTreeSet::new(),
        }
    }

    fn into_types(self) -> Vec<AstType> {
        self.types.into_values().collect()
    }

    fn resolve_variant(
        &self,
        type_id: u32,
    ) -> Result<Vec<scale_info::Variant<PortableForm>>, ServiceAstError> {
        // Portable types are stored by numeric ID. We resolve the ID first and only clone the
        // `TypeDef` data we need afterward so multiple lookups can share the same registry.
        let ty = self
            .registry
            .resolve(type_id)
            .ok_or(ServiceAstError::MissingType(type_id))?;
        if let TypeDef::Variant(variant) = &ty.type_def {
            Ok(variant.variants.clone())
        } else {
            Err(ServiceAstError::InvalidFunction("expected variant type"))
        }
    }

    fn resolve_composite_fields(
        &self,
        type_id: u32,
    ) -> Result<Vec<Field<PortableForm>>, ServiceAstError> {
        let ty = self
            .registry
            .resolve(type_id)
            .ok_or(ServiceAstError::MissingType(type_id))?;
        if let TypeDef::Composite(comp) = &ty.type_def {
            Ok(comp.fields.clone())
        } else {
            Err(ServiceAstError::InvalidFunction("expected composite type"))
        }
    }

    fn type_decl(&mut self, type_id: u32) -> Result<TypeDecl, ServiceAstError> {
        // To avoid infinite recursion on self-referential types, keep track of IDs currently
        // under conversion and fall back to a placeholder if we re-enter the same ID.
        if self.resolving.contains(&type_id) {
            return Ok(TypeDecl::UserDefined {
                name: format!("type_{type_id}"),
                generics: Vec::new(),
            });
        }
        self.resolving.insert(type_id);

        let ty = self
            .registry
            .resolve(type_id)
            .ok_or(ServiceAstError::MissingType(type_id))?
            .clone();
        let path = ty.path.clone();
        let type_params = ty.type_params.clone();
        let type_def = ty.type_def.clone();
        let decl = match type_def {
            TypeDef::Primitive(primitive) => match primitive {
                TypeDefPrimitive::Bool => TypeDecl::Primitive(PrimitiveType::Bool),
                TypeDefPrimitive::Char => TypeDecl::Primitive(PrimitiveType::Char),
                TypeDefPrimitive::Str => TypeDecl::Primitive(PrimitiveType::String),
                TypeDefPrimitive::U8 => TypeDecl::Primitive(PrimitiveType::U8),
                TypeDefPrimitive::U16 => TypeDecl::Primitive(PrimitiveType::U16),
                TypeDefPrimitive::U32 => TypeDecl::Primitive(PrimitiveType::U32),
                TypeDefPrimitive::U64 => TypeDecl::Primitive(PrimitiveType::U64),
                TypeDefPrimitive::U128 => TypeDecl::Primitive(PrimitiveType::U128),
                TypeDefPrimitive::U256 => TypeDecl::Primitive(PrimitiveType::U256),
                TypeDefPrimitive::I8 => TypeDecl::Primitive(PrimitiveType::I8),
                TypeDefPrimitive::I16 => TypeDecl::Primitive(PrimitiveType::I16),
                TypeDefPrimitive::I32 => TypeDecl::Primitive(PrimitiveType::I32),
                TypeDefPrimitive::I64 => TypeDecl::Primitive(PrimitiveType::I64),
                TypeDefPrimitive::I128 => TypeDecl::Primitive(PrimitiveType::I128),
                other => TypeDecl::UserDefined {
                    name: format!("primitive::{other:?}"),
                    generics: Vec::new(),
                },
            },
            TypeDef::Sequence(TypeDefSequence { type_param, .. }) => {
                TypeDecl::Slice(Box::new(self.type_decl(type_param.id)?))
            }
            TypeDef::Array(TypeDefArray { len, type_param }) => {
                let item = self.type_decl(type_param.id)?;
                TypeDecl::Array {
                    item: Box::new(item),
                    len: len as u32,
                }
            }
            TypeDef::Tuple(TypeDefTuple { fields }) => {
                let items = fields
                    .into_iter()
                    .map(|field| self.type_decl(field.id))
                    .collect::<Result<Vec<_>, _>>()?;
                TypeDecl::Tuple(items)
            }
            TypeDef::Variant(variant) if is_option_type(&path) => {
                let inner = variant
                    .variants
                    .into_iter()
                    .flat_map(|v| v.fields)
                    .next()
                    .ok_or(ServiceAstError::InvalidFunction("option missing field"))?;
                TypeDecl::Option(Box::new(self.type_decl(inner.ty.id)?))
            }
            TypeDef::Variant(variant) if is_result_type(&path) => {
                let variants = variant.variants;
                let ok = variants
                    .get(0)
                    .and_then(|v| v.fields.get(0))
                    .ok_or(ServiceAstError::InvalidFunction("result missing ok"))?
                    .ty
                    .id;
                let err = variants
                    .get(1)
                    .and_then(|v| v.fields.get(0))
                    .ok_or(ServiceAstError::InvalidFunction("result missing err"))?
                    .ty
                    .id;
                TypeDecl::Result {
                    ok: Box::new(self.type_decl(ok)?),
                    err: Box::new(self.type_decl(err)?),
                }
            }
            TypeDef::Compact(compact) => self.type_decl(compact.type_param.id)?,
            TypeDef::BitSequence(_) => {
                TypeDecl::Slice(Box::new(TypeDecl::Primitive(PrimitiveType::U8)))
            }
            TypeDef::Composite(composite) =>
                self.user_defined_or_inline_struct(type_id, &path, &type_params, composite)?,
            TypeDef::Variant(variant) =>
                self.user_defined_or_inline_enum(type_id, &path, &type_params, variant)?,
        };

        self.resolving.remove(&type_id);
        Ok(decl)
    }

    fn ensure_named_type(
        &mut self,
        type_id: u32,
        path: &Path<PortableForm>,
        type_params_def: &[scale_info::TypeParameter<PortableForm>],
        type_def: &TypeDef<PortableForm>,
    ) -> Result<(), ServiceAstError> {
        let name = type_path_string(path).unwrap_or_else(|| format!("type#{type_id}"));
        if self.types.contains_key(&name) {
            return Ok(());
        }
        let type_params = type_params_def
            .iter()
            .map(|param| AstTypeParameter {
                name: param.name.to_string(),
                ty: None,
            })
            .collect();
        let docs = Vec::new();
        let def = match type_def {
            TypeDef::Composite(TypeDefComposite { fields, .. }) => {
                let fields = fields
                    .iter()
                    .map(|field| {
                        Ok(StructField {
                            name: field.name.as_ref().map(|n| n.to_string()),
                            type_decl: self.type_decl(field.ty.id)?,
                            docs: convert_docs(&field.docs),
                            annotations: Vec::new(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                AstTypeDef::Struct(StructDef { fields })
            }
            TypeDef::Variant(TypeDefVariant { variants, .. }) => {
                let variants = variants
                    .into_iter()
                    .map(|variant| {
                        let fields = variant
                            .fields
                            .into_iter()
                            .map(|field| {
                                Ok(StructField {
                                    name: field.name.map(|n| n.to_string()),
                                    type_decl: self.type_decl(field.ty.id)?,
                                    docs: convert_docs(&field.docs),
                                    annotations: Vec::new(),
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(EnumVariant {
                            name: variant.name.to_string(),
                            def: StructDef { fields },
                            docs: convert_docs(&variant.docs),
                            annotations: Vec::new(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                AstTypeDef::Enum(EnumDef { variants })
            }
            _ => return Ok(()),
        };
        self.types.insert(
            name.clone(),
            AstType {
                name,
                type_params,
                def,
                docs,
                annotations: Vec::new(),
            },
        );
        Ok(())
    }

    fn user_defined_or_inline_struct(
        &mut self,
        type_id: u32,
        path: &Path<PortableForm>,
        type_params: &[scale_info::TypeParameter<PortableForm>],
        composite: TypeDefComposite<PortableForm>,
    ) -> Result<TypeDecl, ServiceAstError> {
        if let Some(name) = type_path_string(path) {
            self.ensure_named_type(type_id, path, type_params, &TypeDef::Composite(composite.clone()))?;
            Ok(TypeDecl::UserDefined {
                name,
                generics: Vec::new(),
            })
        } else {
            let items = composite
                .fields
                .into_iter()
                .map(|field| self.type_decl(field.ty.id))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TypeDecl::Tuple(items))
        }
    }

    fn user_defined_or_inline_enum(
        &mut self,
        type_id: u32,
        path: &Path<PortableForm>,
        type_params: &[scale_info::TypeParameter<PortableForm>],
        variant: TypeDefVariant<PortableForm>,
    ) -> Result<TypeDecl, ServiceAstError> {
        if let Some(name) = type_path_string(path) {
            self.ensure_named_type(type_id, path, type_params, &TypeDef::Variant(variant))?;
            Ok(TypeDecl::UserDefined {
                name,
                generics: Vec::new(),
            })
        } else {
            Ok(TypeDecl::Tuple(Vec::new()))
        }
    }
}

fn type_path_string(path: &Path<PortableForm>) -> Option<String> {
    if path.segments.is_empty() {
        None
    } else {
        Some(path.segments.join("::"))
    }
}

fn short_type_name(full: &str) -> &str {
    full.rsplit("::").next().unwrap_or(full)
}

fn is_option_type(path: &Path<PortableForm>) -> bool {
    path.segments
        .last()
        .map(|seg| seg.to_string() == "Option")
        .unwrap_or(false)
}

fn is_result_type(path: &Path<PortableForm>) -> bool {
    path.segments
        .last()
        .map(|seg| seg.to_string() == "Result")
        .unwrap_or(false)
}

fn convert_docs<T: AsRef<str>>(docs: &[T]) -> Vec<String> {
    docs.iter().map(|d| d.as_ref().to_string()).collect()
}
