use super::*;
use crate::type_resolver::TypeResolver;
use scale_info::*;
use std::collections::HashSet;

pub struct ProgramBuilder {
    registry: PortableRegistry,
    ctors_type_id: u32, // ctor_fns: Vec<CtorFunc>,
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
                assert_eq!(1, c.fields.len());
                let params_type = self.registry.resolve(c.fields[0].ty.id).unwrap();
                if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                    Ok(CtorFunc {
                        name: c.name.to_string(),
                        params: params_type
                            .fields
                            .iter()
                            .map(|f| FuncParam {
                                name: f.name.unwrap().to_string(),
                                type_decl: resolver.get(f.ty.id).unwrap().clone(),
                            })
                            .collect(),
                        docs: c.docs.iter().map(|s| s.to_string()).collect(),
                        annotations: vec![],
                    })
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    fn services_expo(&self) -> &Vec<ServiceExpo> {
        &self.services_expo
    }

    pub fn build(self, name: String) -> ProgramUnit {
        let mut exclude = HashSet::new();
        exclude.insert(self.ctors_type_id);
        exclude.extend(any_funcs_ids(&self.registry, self.ctors_type_id));
        let resolver = TypeResolver::from(&self.registry, exclude);
        let ctors = self.ctor_funcs(&resolver).unwrap();
        let services = self.services_expo().clone();
        let types = resolver.into_types();

        ProgramUnit {
            name,
            ctors,
            services,
            types,
            docs: vec![],
            annotations: vec![],
        }
    }
}

fn any_funcs(
    registry: &PortableRegistry,
    func_type_id: u32,
) -> Result<impl Iterator<Item = &Variant<PortableForm>>> {
    let funcs = registry.resolve(func_type_id).unwrap_or_else(|| {
        panic!("func type id {func_type_id} not found while it was registered previously")
    });
    if let scale_info::TypeDef::Variant(variant) = &funcs.type_def {
        Ok(variant.variants.iter())
    } else {
        Err(Error::FuncMetaIsInvalid(format!(
            "func type id {func_type_id} references a type that is not a variant"
        )))
    }
}

fn any_funcs_ids(registry: &PortableRegistry, func_type_id: u32) -> impl Iterator<Item = u32> {
    let funcs = registry.resolve(func_type_id).unwrap();
    if let scale_info::TypeDef::Variant(variant) = &funcs.type_def {
        variant.variants.iter().map(|v| v.fields[0].ty.id)
    } else {
        unreachable!()
    }
}

fn flat_meta(
    service_meta: &AnyServiceMeta,
    meta: fn(&AnyServiceMeta) -> &MetaType,
) -> Vec<&MetaType> {
    let mut metas = vec![meta(service_meta)];
    for base_service_meta in service_meta.base_services() {
        metas.extend(flat_meta(base_service_meta, meta));
    }
    metas
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
        let commands_type_id = registry.register_type(&meta.commands()).id;
        let queries_type_id = registry.register_type(&meta.queries()).id;
        let events_type_id = registry.register_type(&meta.events()).id;
        let registry = PortableRegistry::from(registry);
        Self {
            name,
            registry,
            commands_type_id,
            queries_type_id,
            events_type_id,
        }
    }

    pub fn build(self) -> ServiceUnit {
        let mut exclude = HashSet::new();
        exclude.insert(self.commands_type_id);
        exclude.extend(any_funcs_ids(&self.registry, self.commands_type_id));
        exclude.insert(self.queries_type_id);
        exclude.extend(any_funcs_ids(&self.registry, self.queries_type_id));
        exclude.insert(self.events_type_id);
        let resolver = TypeResolver::from(&self.registry, exclude);
        let commands = self.commands(&resolver).unwrap();
        let queries = self.queries(&resolver).unwrap();
        let events = self.events(&resolver).unwrap();
        // let extends = self.extends();
        // let services = self.services_expo().clone();
        let types = resolver.into_types();

        ServiceUnit {
            name: self.name.to_string(),
            extends: vec![],
            funcs: [commands, queries].concat(),
            events,
            types,
            docs: vec![],
            annotations: vec![],
        }
    }

    fn commands(&self, resolver: &TypeResolver) -> Result<Vec<ServiceFunc>> {
        any_funcs(&self.registry, self.commands_type_id)?
            .map(|c| {
                assert_eq!(2, c.fields.len());
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
                                name: f.name.unwrap().to_string(),
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
                    unreachable!()
                }
            })
            .collect()
    }

    fn queries(&self, resolver: &TypeResolver) -> Result<Vec<ServiceFunc>> {
        any_funcs(&self.registry, self.queries_type_id)?
            .map(|c| {
                assert_eq!(2, c.fields.len());
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
                                name: f.name.unwrap().to_string(),
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
                    unreachable!()
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
                            name: field.name.map(|s| s.to_string()),
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
