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
                        Ok(CtorFunc {
                            name: c.name.to_string(),
                            params: params_type
                                .fields
                                .iter()
                                .map(|f| FuncParam {
                                    name: f.name.as_ref().unwrap().to_string(),
                                    type_decl: resolver.get(f.ty.id).unwrap().clone(),
                                })
                                .collect(),
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

    fn services_expo(&self) -> &Vec<ServiceExpo> {
        &self.services_expo
    }

    pub fn build(self, name: String) -> Result<ProgramUnit> {
        let mut exclude = HashSet::new();
        exclude.insert(self.ctors_type_id);
        exclude.extend(any_funcs_ids(&self.registry, self.ctors_type_id)?);
        let resolver = TypeResolver::from(&self.registry, exclude);
        let ctors = self.ctor_funcs(&resolver)?;
        let services = self.services_expo().clone();
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

fn any_funcs_ids(
    registry: &PortableRegistry,
    func_type_id: u32,
) -> Result<impl Iterator<Item = u32>> {
    let any_funcs = any_funcs(registry, func_type_id)?;
    Ok(any_funcs.into_iter().map(|v| v.fields[0].ty.id))
}

pub struct ServiceBuilder {
    name: &'static str,
    registry: PortableRegistry,
    commands_type_id: u32,
    queries_type_id: u32,
    events_type_id: u32,
}

impl ServiceBuilder {
    pub fn new(name: &'static str, meta: AnyServiceMeta) -> Self {
        let mut registry = Registry::new();
        let commands_type_id = registry.register_type(meta.commands()).id;
        let queries_type_id = registry.register_type(meta.queries()).id;
        let events_type_id = registry.register_type(meta.events()).id;
        let registry = PortableRegistry::from(registry);
        Self {
            name,
            registry,
            commands_type_id,
            queries_type_id,
            events_type_id,
        }
    }

    pub fn build(self) -> Result<Vec<ServiceUnit>> {
        let exclude = HashSet::from_iter(self.exclude_type_ids()?);
        let resolver = TypeResolver::from(&self.registry, exclude);
        let commands = self.commands(&resolver)?;
        let queries = self.queries(&resolver)?;
        let events = self.events(&resolver)?;
        // let extends = self.extends();
        // let services = self.services_expo().clone();
        let types = resolver.into_types();

        Ok(vec![ServiceUnit {
            name: self.name.to_string(),
            extends: vec![],
            funcs: [commands, queries].concat(),
            events,
            types,
            docs: vec![],
            annotations: vec![],
        }])
    }

    fn exclude_type_ids(&self) -> Result<impl Iterator<Item = u32>> {
        Ok([
            self.commands_type_id,
            self.queries_type_id,
            self.events_type_id,
        ]
        .into_iter()
        .chain(any_funcs_ids(&self.registry, self.commands_type_id)?)
        .chain(any_funcs_ids(&self.registry, self.queries_type_id)?))
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
                    let params_type = self.registry.resolve(c.fields[0].ty.id).unwrap();
                    let mut output = resolver.get(c.fields[1].ty.id).unwrap().clone();
                    let mut throws = None;
                    // TODO: unwrap result param
                    if let Some((ok, err)) = TypeDecl::result_type_decl(&output) {
                        output = ok;
                        throws = Some(err);
                    };
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        Ok(ServiceFunc {
                            name: c.name.to_string(),
                            params: params_type
                                .fields
                                .iter()
                                .map(|f| FuncParam {
                                    name: f.name.as_ref().unwrap().to_string(),
                                    type_decl: resolver.get(f.ty.id).unwrap().clone(),
                                })
                                .collect(),
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
                    let params_type = self.registry.resolve(c.fields[0].ty.id).unwrap();
                    let mut output = resolver.get(c.fields[1].ty.id).unwrap().clone();
                    let mut throws = None;
                    // TODO: unwrap result param
                    if let Some((ok, err)) = TypeDecl::result_type_decl(&output) {
                        output = ok;
                        throws = Some(err);
                    };
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        Ok(ServiceFunc {
                            name: c.name.to_string(),
                            params: params_type
                                .fields
                                .iter()
                                .map(|f| FuncParam {
                                    name: f.name.as_ref().unwrap().to_string(),
                                    type_decl: resolver.get(f.ty.id).unwrap().clone(),
                                })
                                .collect(),
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
                    .map(|field| {
                        let type_decl = resolver.get(field.ty.id).unwrap().clone();
                        StructField {
                            name: field.name.as_ref().map(|s| s.to_string()),
                            type_decl,
                            docs: field.docs.iter().map(|d| d.to_string()).collect(),
                            annotations: vec![],
                        }
                    })
                    .collect();

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
