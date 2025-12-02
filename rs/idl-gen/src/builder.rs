use super::*;
use crate::type_resolver::TypeResolver;
use scale_info::*;
use std::collections::HashSet;

pub struct ProgramBuilder {
    registry: PortableRegistry,
    ctors_type_id: u32,
    services_expo: Vec<ServiceExpo>,
}

impl ProgramBuilder {
    pub fn new<P: ProgramMeta>() -> Self {
        let ctors = P::constructors();
        let mut registry = Registry::new();
        let ctors_type_id = registry.register_type(&ctors).id;
        let services_expo = P::services()
            .map(|(name, _meta)| ServiceExpo {
                name: name.to_string(),
                route: None,
                docs: vec![],
                annotations: vec![],
            })
            .collect::<Vec<_>>();
        let registry = PortableRegistry::from(registry);
        Self {
            registry,
            ctors_type_id,
            services_expo,
        }
    }

    fn ctor_funcs(&self, resolver: &TypeResolver) -> Result<Vec<CtorFunc>> {
        any_funcs(&self.registry, self.ctors_type_id)?
            .map(|c| {
                if c.fields.len() != 1 {
                    Err(Error::FuncMetaIsInvalid(format!(
                        "ctor `{}` has invalid number of fields",
                        c.name
                    )))
                } else {
                    let params_type_id = c.fields[0].ty.id;
                    let params_type = &self
                        .registry
                        .resolve(params_type_id)
                        .ok_or(Error::TypeIdIsUnknown(params_type_id))?;
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        let params = params_type
                            .fields
                            .iter()
                            .map(|f| -> Result<_> {
                                let name = f.name.as_ref().ok_or_else(|| {
                                    Error::FuncMetaIsInvalid(format!(
                                        "ctor `{}` param is missing a name",
                                        c.name
                                    ))
                                })?;
                                let type_decl = resolver
                                    .get(f.ty.id)
                                    .cloned()
                                    .ok_or(Error::TypeIdIsUnknown(f.ty.id))?;
                                Ok(FuncParam {
                                    name: name.to_string(),
                                    type_decl,
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Ok(CtorFunc {
                            name: c.name.to_string(),
                            params,
                            docs: c.docs.iter().map(|s| s.to_string()).collect(),
                            annotations: vec![],
                        })
                    } else {
                        Err(Error::FuncMetaIsInvalid(format!(
                            "ctor `{}` params type is not a composite",
                            c.name
                        )))
                    }
                }
            })
            .collect()
    }

    pub fn build(self, name: String) -> Result<ProgramUnit> {
        let mut exclude = HashSet::new();
        exclude.insert(self.ctors_type_id);
        exclude.extend(any_funcs_ids(&self.registry, self.ctors_type_id)?);
        let resolver = TypeResolver::from(&self.registry, exclude);
        let ctors = self.ctor_funcs(&resolver)?;
        let services = self.services_expo;
        let types = resolver.into_types();

        Ok(ProgramUnit {
            name,
            ctors,
            services,
            types,
            docs: vec![],
            annotations: vec![],
        })
    }
}

fn any_funcs(
    registry: &PortableRegistry,
    func_type_id: u32,
) -> Result<impl Iterator<Item = &Variant<PortableForm>>> {
    let funcs = registry
        .resolve(func_type_id)
        .ok_or(Error::TypeIdIsUnknown(func_type_id))?;
    if let scale_info::TypeDef::Variant(variant) = &funcs.type_def {
        Ok(variant.variants.iter())
    } else {
        Err(Error::FuncMetaIsInvalid(format!(
            "func type id {func_type_id} references a type that is not a variant"
        )))
    }
}

fn any_funcs_ids(registry: &PortableRegistry, func_type_id: u32) -> Result<Vec<u32>> {
    any_funcs(registry, func_type_id)?
        .map(|variant| {
            variant
                .fields
                .first()
                .map(|field| field.ty.id)
                .ok_or_else(|| {
                    Error::FuncMetaIsInvalid(format!("func `{}` has no fields", variant.name))
                })
        })
        .collect::<Result<Vec<_>>>()
}

pub struct ServiceBuilder<'a> {
    name: &'a str,
    meta: &'a AnyServiceMeta,
    registry: PortableRegistry,
    commands_type_id: u32,
    queries_type_id: u32,
    events_type_id: u32,
}

impl<'a> ServiceBuilder<'a> {
    pub fn new(name: &'a str, meta: &'a AnyServiceMeta) -> Self {
        let mut registry = Registry::new();
        let commands_type_id = registry.register_type(meta.commands()).id;
        let queries_type_id = registry.register_type(meta.queries()).id;
        let events_type_id = registry.register_type(meta.events()).id;
        let registry = PortableRegistry::from(registry);
        Self {
            name,
            meta,
            registry,
            commands_type_id,
            queries_type_id,
            events_type_id,
        }
    }

    pub fn build(self) -> Result<Vec<ServiceUnit>> {
        let mut services = Vec::new();
        let mut extends = Vec::new();
        for meta in self.meta.base_services() {
            // TODO: add base service names to Meta trait
            let name = "TodoBaseName";
            extends.push(name.to_string());
            // TODO: dedup base services based on `interface_id`
            services.extend(ServiceBuilder::new(name, meta).build()?);
        }

        let exclude = HashSet::from_iter(self.exclude_type_ids()?);
        let resolver = TypeResolver::from(&self.registry, exclude);
        let commands = self.commands(&resolver)?;
        let queries = self.queries(&resolver)?;
        let events = self.events(&resolver)?;
        let types = resolver.into_types();

        services.push(ServiceUnit {
            name: self.name.to_string(),
            extends,
            funcs: [commands, queries].concat(),
            events,
            types,
            docs: vec![],
            annotations: vec![],
        });
        Ok(services)
    }

    fn exclude_type_ids(&self) -> Result<impl Iterator<Item = u32>> {
        let base = vec![
            self.commands_type_id,
            self.queries_type_id,
            self.events_type_id,
        ]
        .into_iter();
        let command_ids = any_funcs_ids(&self.registry, self.commands_type_id)?;
        let query_ids = any_funcs_ids(&self.registry, self.queries_type_id)?;
        Ok(base.chain(command_ids).chain(query_ids))
    }

    fn commands(&self, resolver: &TypeResolver) -> Result<Vec<ServiceFunc>> {
        any_funcs(&self.registry, self.commands_type_id)?
            .map(|c| {
                if c.fields.len() != 2 {
                    Err(Error::FuncMetaIsInvalid(format!(
                        "command `{}` has invalid number of fields",
                        c.name
                    )))
                } else {
                    let params_type_id = c.fields[0].ty.id;
                    let params_type = self
                        .registry
                        .resolve(params_type_id)
                        .ok_or(Error::TypeIdIsUnknown(params_type_id))?;
                    let output_type_id = c.fields[1].ty.id;
                    let mut output = resolver
                        .get(output_type_id)
                        .cloned()
                        .ok_or(Error::TypeIdIsUnknown(output_type_id))?;
                    let mut throws = None;
                    // TODO: unwrap result param
                    if let Some((ok, err)) = TypeDecl::result_type_decl(&output) {
                        output = ok;
                        throws = Some(err);
                    };
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        let params = params_type
                            .fields
                            .iter()
                            .map(|f| -> Result<_> {
                                let name = f.name.as_ref().ok_or_else(|| {
                                    Error::FuncMetaIsInvalid(format!(
                                        "command `{}` param is missing a name",
                                        c.name
                                    ))
                                })?;
                                let type_decl = resolver
                                    .get(f.ty.id)
                                    .cloned()
                                    .ok_or(Error::TypeIdIsUnknown(f.ty.id))?;
                                Ok(FuncParam {
                                    name: name.to_string(),
                                    type_decl,
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Ok(ServiceFunc {
                            name: c.name.to_string(),
                            params,
                            output,
                            throws,
                            kind: FunctionKind::Command,
                            docs: c.docs.iter().map(|s| s.to_string()).collect(),
                            annotations: vec![],
                        })
                    } else {
                        Err(Error::FuncMetaIsInvalid(format!(
                            "command `{}` params type is not a composite",
                            c.name
                        )))
                    }
                }
            })
            .collect()
    }

    fn queries(&self, resolver: &TypeResolver) -> Result<Vec<ServiceFunc>> {
        any_funcs(&self.registry, self.queries_type_id)?
            .map(|c| {
                if c.fields.len() != 2 {
                    Err(Error::FuncMetaIsInvalid(format!(
                        "query `{}` has invalid number of fields",
                        c.name
                    )))
                } else {
                    let params_type_id = c.fields[0].ty.id;
                    let params_type = self
                        .registry
                        .resolve(params_type_id)
                        .ok_or(Error::TypeIdIsUnknown(params_type_id))?;
                    let output_type_id = c.fields[1].ty.id;
                    let mut output = resolver
                        .get(output_type_id)
                        .cloned()
                        .ok_or(Error::TypeIdIsUnknown(output_type_id))?;
                    let mut throws = None;
                    // TODO: unwrap result param
                    if let Some((ok, err)) = TypeDecl::result_type_decl(&output) {
                        output = ok;
                        throws = Some(err);
                    };
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        let params = params_type
                            .fields
                            .iter()
                            .map(|f| -> Result<_> {
                                let name = f.name.as_ref().ok_or_else(|| {
                                    Error::FuncMetaIsInvalid(format!(
                                        "query `{}` param is missing a name",
                                        c.name
                                    ))
                                })?;
                                let type_decl = resolver
                                    .get(f.ty.id)
                                    .cloned()
                                    .ok_or(Error::TypeIdIsUnknown(f.ty.id))?;
                                Ok(FuncParam {
                                    name: name.to_string(),
                                    type_decl,
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Ok(ServiceFunc {
                            name: c.name.to_string(),
                            params,
                            output,
                            // TODO: Throws type
                            throws,
                            kind: FunctionKind::Query,
                            docs: c.docs.iter().map(|s| s.to_string()).collect(),
                            annotations: vec![("query".to_string(), None)],
                        })
                    } else {
                        Err(Error::FuncMetaIsInvalid(format!(
                            "query `{}` params type is not a composite",
                            c.name
                        )))
                    }
                }
            })
            .collect()
    }

    fn events(&self, resolver: &TypeResolver) -> Result<Vec<ServiceEvent>> {
        any_funcs(&self.registry, self.events_type_id)?
            .map(|v| {
                let fields = v
                    .fields
                    .iter()
                    .map(|field| -> Result<_> {
                        let type_decl = resolver
                            .get(field.ty.id)
                            .cloned()
                            .ok_or(Error::TypeIdIsUnknown(field.ty.id))?;
                        Ok(StructField {
                            name: field.name.as_ref().map(|s| s.to_string()),
                            type_decl,
                            docs: field.docs.iter().map(|d| d.to_string()).collect(),
                            annotations: vec![],
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(ServiceEvent {
                    name: v.name.to_string(),
                    def: StructDef { fields },
                    docs: v.docs.iter().map(|d| d.to_string()).collect(),
                    annotations: vec![],
                })
            })
            .collect()
    }
}
