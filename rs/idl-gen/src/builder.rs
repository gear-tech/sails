use std::collections::HashSet;

use super::*;
use scale_info::*;

pub struct ProgramMetaBuilder {
    registry: PortableRegistry,
    ctors_type_id: u32, // ctor_fns: Vec<CtorFunc>,
}

impl ProgramMetaBuilder {
    pub fn new(
        ctors: MetaType,
        // services: impl Iterator<Item = (&'static str, AnyServiceMeta)>,
    ) -> Result<Self> {
        let mut registry = Registry::new();
        let ctors_type_id = registry.register_type(&ctors).id;
        // let services_data = services
        //     .map(|(sname, sm)| {
        //         (
        //             sname,
        //             Self::flat_meta(&sm, |sm| sm.commands())
        //                 .into_iter()
        //                 .map(|mt| registry.register_type(mt).id)
        //                 .collect::<Vec<_>>(),
        //             Self::flat_meta(&sm, |sm| sm.queries())
        //                 .into_iter()
        //                 .map(|mt| registry.register_type(mt).id)
        //                 .collect::<Vec<_>>(),
        //             Self::flat_meta(&sm, |sm| sm.events())
        //                 .into_iter()
        //                 .map(|mt| registry.register_type(mt).id)
        //                 .collect::<Vec<_>>(),
        //         )
        //     })
        //     .collect::<Vec<_>>();
        let registry = PortableRegistry::from(registry);
        // let ctor_fns = Self::ctor_funcs(&registry, ctors_type_id)?;
        // let services = services_data
        //     .into_iter()
        //     .map(|(sname, ct_ids, qt_ids, et_ids)| {
        //         ExpandedServiceMeta::new(&registry, sname, ct_ids, qt_ids, et_ids)
        //     })
        //     .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            registry,
            ctors_type_id,
        })
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

    pub fn build(self, name: String) -> ProgramUnit {
        let mut exclude = HashSet::new();
        exclude.insert(self.ctors_type_id);
        exclude.extend(any_funcs_ids(&self.registry, self.ctors_type_id));
        let resolver = TypeResolver::from(&self.registry, exclude);
        let ctors = self.ctor_funcs(&resolver).unwrap();
        let types = resolver.into_types();

        ProgramUnit {
            name,
            ctors,
            services: vec![],
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
