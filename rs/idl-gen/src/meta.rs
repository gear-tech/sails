// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Struct describing the types of a service comprised of command and query handlers.

use std::{collections::{BTreeSet, HashSet}, num::{NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8}};

use crate::{
    errors::{Error, Result}, type_names, FuncArgIdl2, FunctionIdl2Data, FunctionsSection, ProgramIdlSection, ServiceSection
};
use gprimitives::*;
use sails_idl_meta::*;
use scale_info::{
    Field, MetaType, PortableRegistry, PortableType, Registry, TypeDef, Variant, form::PortableForm,
};

#[derive(Debug)]
struct CtorFuncMeta(String, u32, Vec<Field<PortableForm>>, Vec<String>);

#[derive(Debug)]
struct ServiceFuncMeta(String, u32, Vec<Field<PortableForm>>, u32, Vec<String>);

#[derive(Debug, serde::Serialize)]
pub(crate) struct ExpandedProgramMeta2 {
    program: ProgramIdlSection,
    services: Vec<ServiceSection>,
}

impl ExpandedProgramMeta2 {
    pub fn new(name: String, ctors: MetaType, services: impl Iterator<Item = (&'static str, AnyServiceMeta)>,) -> Result<Self> {
        // Create registry
        let mut ctor_registry = Registry::new();
        // todo [sab] add to ctors registry the builtin types and do type names resolution too
        let ctor_registry_id = ctor_registry.register_type(&ctors).id;

        let services_registry_ids = services.map(|(service_name, meta)| {
            let mut registry = Registry::new();
            let builtin_type_ids = registry.register_types([
                MetaType::new::<ActorId>(),
                MetaType::new::<CodeId>(),
                MetaType::new::<MessageId>(),
                MetaType::new::<H160>(),
                MetaType::new::<H256>(),
                MetaType::new::<U256>(),
                MetaType::new::<NonZeroU8>(),
                MetaType::new::<NonZeroU16>(),
                MetaType::new::<NonZeroU32>(),
                MetaType::new::<NonZeroU64>(),
                MetaType::new::<NonZeroU128>(),
                MetaType::new::<NonZeroU256>(),
            ]);
            let builtin_type_ids = builtin_type_ids
                .into_iter()
                .map(|t| t.id)
                .collect::<BTreeSet<_>>();

            let commands = meta.commands();
            let commands_registry_id = registry.register_type(&commands).id;

            let queries = meta.queries();
            let queries_registry_id = registry.register_type(&queries).id;

            let events = meta.events();
            let events_registry_id = registry.register_type(&events).id;

            // todo [sab] base services and their events
            // meta.base_services().for_each(|base_service_meta| {
            //     let base_commands = base_service_meta.commands();
            //     registry.register_type(&base_commands);
            //     let base_queries = base_service_meta.queries();
            //     registry.register_type(&base_queries);
            //     let base_events = base_service_meta.events();
            //     registry.register_type(&base_events);
            // });

            (registry, service_name, commands_registry_id, queries_registry_id, events_registry_id, builtin_type_ids)
        });

        // Process registered types to form ctors and services idl data
        let ctor_portable_registry = PortableRegistry::from(ctor_registry);
        let type_names = type_names::resolve(ctor_portable_registry.types.iter())?;
        let type_names = type_names
            .values()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        let ctors = Self::ctor_funcs(&ctor_portable_registry, ctor_registry_id)?;

        let program_section = ProgramIdlSection {
            name,
            type_names,
            ctors,
        };

        let mut services = Vec::new();
        for (mut registry, name, commands_id, queries_id, events_id, builtin_type_ids) in services_registry_ids {
            // Handle commands
            let mut generated_types = HashSet::new();
            generated_types.insert(commands_id);
            generated_types.insert(queries_id);
            generated_types.insert(events_id);

            let unit_ty_id = registry.register_type(&MetaType::new::<()>()).id;

            let service_portable_registry = PortableRegistry::from(registry);
            let mut commands = Vec::new();
            let commands_type = service_portable_registry
                .resolve(commands_id)
                .expect("command type was added previously; qed.");

            let TypeDef::Variant(ref variants) = commands_type.type_def else {
                return Err(Error::FuncMetaIsInvalid("Commands type is not a variant".to_string()));
            };

            for variant in &variants.variants {
                if variant.fields.len() != 2 {
                    return Err(Error::FuncMetaIsInvalid(format!(
                        "command `{}` has invalid number of fields, expected 2, got {}",
                        variant.name, variant.fields.len()
                    )));
                }

                // Add to generated types __*Params type of service's variant in `CommandsMeta`
                generated_types.insert(variant.fields[0].ty.id);

                // Take args (fields of __*Params type)
                let args_type = service_portable_registry
                    .resolve(variant.fields[0].ty.id)
                    .expect("args type was added previously; qed.");
                let TypeDef::Composite(args_type) = &args_type.type_def else {
                    return Err(Error::FuncMetaIsInvalid(format!(
                        "command `{}` args type is not a composite", variant.name
                    )));
                };
                let args = args_type.fields.iter().map(|f| -> Result<FuncArgIdl2, Error> {
                    let name = f.name
                        .as_ref()
                        .ok_or_else(|| Error::FuncMetaIsInvalid(format!("command `{}` arg must have a name", variant.name)))?
                        .to_string();
                    Ok(FuncArgIdl2 {
                        name,
                        ty: f.ty.id,
                    })
                }).collect::<Result<Vec<_>, _>>()?;

                // Take result type
                let result_ty = variant.fields[1].ty.id;

                let command = FunctionIdl2Data {
                    name: variant.name.to_string(),
                    args,
                    result_ty: (result_ty != unit_ty_id).then_some(result_ty),
                    docs: variant.docs.iter().map(|s| s.to_string()).collect(),
                };

                commands.push(command);
            }

            // Handle queries
            let mut queries = Vec::new();
            let queries_type = service_portable_registry
                .resolve(queries_id)
                .expect("query type was added previously; qed.");

            let TypeDef::Variant(ref variants) = queries_type.type_def else {
                return Err(Error::FuncMetaIsInvalid("Queries type is not a variant".to_string()));
            };

            for variant in &variants.variants {
                if variant.fields.len() != 2 {
                    return Err(Error::FuncMetaIsInvalid(format!(
                        "query `{}` has invalid number of fields, expected 2, got {}",
                        variant.name, variant.fields.len()
                    )));
                }

                // Add to generated types __*Params type of service's variant in `QueriesMeta`
                generated_types.insert(variant.fields[0].ty.id);

                // Take args (fields of __*Params type)
                let args_type = service_portable_registry
                    .resolve(variant.fields[0].ty.id)
                    .expect("args type was added previously; qed.");
                let TypeDef::Composite(args_type) = &args_type.type_def else {
                    return Err(Error::FuncMetaIsInvalid(format!(
                        "query `{}` args type is not a composite", variant.name
                    )));
                };
                let args = args_type.fields.iter().map(|f| -> Result<FuncArgIdl2, Error> {
                    let name = f.name
                        .as_ref()
                        .ok_or_else(|| Error::FuncMetaIsInvalid(format!("query `{}` arg must have a name", variant.name)))?
                        .to_string();
                    Ok(FuncArgIdl2 {
                        name,
                        ty: f.ty.id,
                    })
                }).collect::<Result<Vec<_>, _>>()?;

                // Take result type
                let result_ty = variant.fields[1].ty.id;

                let query = FunctionIdl2Data {
                    name: variant.name.to_string(),
                    args,
                    result_ty: (result_ty != unit_ty_id).then_some(result_ty),
                    docs: variant.docs.iter().map(|s| s.to_string()).collect(),
                };

                queries.push(query);
            }

            // Handle events
            let events_type = service_portable_registry
                .resolve(events_id)
                .expect("event type was added previously; qed.");

            let TypeDef::Variant(ref variants) = events_type.type_def else {
                return Err(Error::EventMetaIsInvalid("events type is not a variant".to_string()));
            };
            let events = variants.variants.clone();

            let types = service_portable_registry
                .types
                .iter()
                .filter(|ty| {
                    // todo [sab] test that skips primitive types and Result/Option + builtin types
                    !ty.ty.path.namespace().is_empty() && !generated_types.contains(&ty.id) && !builtin_type_ids.contains(&ty.id)
                })
                .cloned()
                .collect::<Vec<_>>();

            let type_names = type_names::resolve(service_portable_registry.types.iter()).unwrap();
            let type_names = type_names
                .values()
                .map(|name| name.to_string())
                .collect::<Vec<_>>();
            let service = ServiceSection {
                name: name.to_string(),
                type_names,
                extends: Default::default(), // todo [sab]
                events,
                types,
                functions: FunctionsSection {
                    commands,
                    queries,
                },
            };

            services.push(service);
        }

        Ok(Self {
            program: program_section,
            services,
        })
    }

    fn ctor_funcs(
        registry: &PortableRegistry,
        func_type_id: u32,
    ) -> Result<Vec<FunctionIdl2Data>> {
        any_funcs(registry, func_type_id)?
            .map(|constructor_fn| {
                if constructor_fn.fields.len() != 1 {
                    Err(Error::FuncMetaIsInvalid(format!(
                        "ctor `{}` has invalid number of fields",
                        constructor_fn.name
                    )))
                } else {
                    let param_type_id = constructor_fn.fields[0].ty.id;
                    let params_type = registry.resolve(param_type_id).ok_or(
                        Error::TypeIdIsUnknown(param_type_id)
                    )?;

                    if let TypeDef::Composite(params_type) = &params_type.type_def {
                        let mut args = Vec::with_capacity(params_type.fields.len());
                        for f in &params_type.fields {
                            let name = f.name
                                .map(|s| s.to_string())
                                .ok_or(Error::FuncMetaIsInvalid(format!("ctor {} has nameless argument", constructor_fn.name)))?;
                            args.push(FuncArgIdl2 {
                                name: name,
                                ty: f.ty.id,
                            });
                        }
                        Ok(FunctionIdl2Data {
                            name: constructor_fn.name.to_string(),
                            args,
                            result_ty: None,
                            docs: constructor_fn.docs.iter().map(|s| s.to_string()).collect(),
                        })
                    } else {
                        Err(Error::FuncMetaIsInvalid(format!(
                            "ctor `{}` params type is not a composite",
                            constructor_fn.name
                        )))
                    }
                }
            })
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct ExpandedProgramMeta {
    registry: PortableRegistry,
    builtin_type_ids: Vec<u32>,
    ctors_type_id: Option<u32>,
    ctors: Vec<CtorFuncMeta>,
    services: Vec<ExpandedServiceMeta>,
}

impl ExpandedProgramMeta {
    pub fn new(
        ctors: Option<MetaType>,
        services: impl Iterator<Item = (&'static str, AnyServiceMeta)>,
    ) -> Result<Self> {
        let mut registry = Registry::new();
        let builtin_type_ids = registry
            .register_types([
                MetaType::new::<ActorId>(),
                MetaType::new::<CodeId>(),
                MetaType::new::<MessageId>(),
                MetaType::new::<H160>(),
                MetaType::new::<H256>(),
                MetaType::new::<U256>(),
                MetaType::new::<NonZeroU256>(),
            ])
            .iter()
            .map(|t| t.id)
            .collect::<Vec<_>>();
        let ctors_type_id = ctors.map(|ctors| registry.register_type(&ctors).id);
        let services_data = services
            .map(|(sname, sm)| {
                (
                    sname,
                    Self::flat_meta(&sm, |sm| sm.commands())
                        .into_iter()
                        .map(|mt| registry.register_type(mt).id)
                        .collect::<Vec<_>>(),
                    Self::flat_meta(&sm, |sm| sm.queries())
                        .into_iter()
                        .map(|mt| registry.register_type(mt).id)
                        .collect::<Vec<_>>(),
                    Self::flat_meta(&sm, |sm| sm.events())
                        .into_iter()
                        .map(|mt| registry.register_type(mt).id)
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let registry = PortableRegistry::from(registry);
        let ctors = Self::ctor_funcs(&registry, ctors_type_id)?;
        let services = services_data
            .into_iter()
            .map(|(sname, ct_ids, qt_ids, et_ids)| {
                ExpandedServiceMeta::new(&registry, sname, ct_ids, qt_ids, et_ids)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            registry,
            builtin_type_ids,
            ctors_type_id,
            ctors,
            services,
        })
    }

    /// Returns complex types introduced by program only
    pub fn types(&self) -> impl Iterator<Item = &PortableType> {
        self.registry.types.iter().filter(|ty| {
            !ty.ty.path.namespace().is_empty()
                && self.ctors_type_id.is_none_or(|id| id != ty.id)
                && !self.commands_type_ids().any(|id| id == ty.id)
                && !self.queries_type_ids().any(|id| id == ty.id)
                && !self.events_type_ids().any(|id| id == ty.id)
                && !self.ctor_params_type_ids().any(|id| id == ty.id)
                && !self.command_params_type_ids().any(|id| id == ty.id)
                && !self.query_params_type_ids().any(|id| id == ty.id)
                && !self.builtin_type_ids.contains(&ty.id)
        })
    }

    pub fn ctors(&self) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>, &Vec<String>)> {
        self.ctors.iter().map(|c| (c.0.as_str(), &c.2, &c.3))
    }

    pub fn services(&self) -> impl Iterator<Item = &ExpandedServiceMeta> {
        self.services.iter()
    }

    /// Returns names for all types used by program including primitive, complex and "internal" ones.
    /// Each type name index corresponds to id of the type
    pub fn type_names(&self) -> Result<impl Iterator<Item = String>> {
        let names = type_names::resolve(self.registry.types.iter())?;
        Ok(names.into_iter().map(|i| i.1))
    }

    fn ctor_funcs(
        registry: &PortableRegistry,
        func_type_id: Option<u32>,
    ) -> Result<Vec<CtorFuncMeta>> {
        if func_type_id.is_none() {
            return Ok(Vec::new());
        }
        let func_type_id = func_type_id.unwrap();
        any_funcs(registry, func_type_id)?
            .map(|c| {
                if c.fields.len() != 1 {
                    Err(Error::FuncMetaIsInvalid(format!(
                        "ctor `{}` has invalid number of fields",
                        c.name
                    )))
                } else {
                    let params_type = registry.resolve(c.fields[0].ty.id).unwrap_or_else(|| {
                        panic!(
                            "ctor params type id {} not found while it was registered previously",
                            c.fields[0].ty.id
                        )
                    });
                    if let TypeDef::Composite(params_type) = &params_type.type_def {
                        Ok(CtorFuncMeta(
                            c.name.to_string(),
                            c.fields[0].ty.id,
                            params_type.fields.to_vec(),
                            c.docs.iter().map(|s| s.to_string()).collect(),
                        ))
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

    fn flat_meta(
        service_meta: &AnyServiceMeta,
        meta: fn(&AnyServiceMeta) -> &MetaType,
    ) -> Vec<&MetaType> {
        let mut metas = vec![meta(service_meta)];
        for base_service_meta in service_meta.base_services() {
            metas.extend(Self::flat_meta(base_service_meta, meta));
        }
        metas
    }

    fn commands_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.services
            .iter()
            .flat_map(|s| s.commands_type_ids.iter().copied())
    }

    fn queries_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.services
            .iter()
            .flat_map(|s| s.queries_type_ids.iter().copied())
    }

    fn events_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.services
            .iter()
            .flat_map(|s| s.events_type_ids.iter().copied())
    }

    fn ctor_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.ctors.iter().map(|v| v.1)
    }

    fn command_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.services
            .iter()
            .flat_map(|s| s.commands.iter().chain(&s.overriden_commands).map(|v| v.1))
    }

    fn query_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.services
            .iter()
            .flat_map(|s| s.queries.iter().chain(&s.overriden_queries).map(|v| v.1))
    }
}

#[derive(Debug)]
pub(crate) struct ExpandedServiceMeta {
    name: &'static str,
    commands_type_ids: Vec<u32>,
    commands: Vec<ServiceFuncMeta>,
    overriden_commands: Vec<ServiceFuncMeta>,
    queries_type_ids: Vec<u32>,
    queries: Vec<ServiceFuncMeta>,
    overriden_queries: Vec<ServiceFuncMeta>,
    events_type_ids: Vec<u32>,
    events: Vec<Variant<PortableForm>>,
}

impl ExpandedServiceMeta {
    fn new(
        registry: &PortableRegistry,
        name: &'static str,
        commands_type_ids: Vec<u32>,
        queries_type_ids: Vec<u32>,
        events_type_ids: Vec<u32>,
    ) -> Result<Self> {
        let (commands, overriden_commands) =
            Self::service_funcs(registry, commands_type_ids.iter().copied())?;
        let (queries, overriden_queries) =
            Self::service_funcs(registry, queries_type_ids.iter().copied())?;
        let events = Self::event_variants(registry, events_type_ids.iter().copied())?;
        Ok(Self {
            name,
            commands_type_ids,
            commands,
            overriden_commands,
            queries_type_ids,
            queries,
            overriden_queries,
            events_type_ids,
            events,
        })
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn commands(
        &self,
    ) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>, u32, &Vec<String>)> {
        self.commands
            .iter()
            .map(|c| (c.0.as_str(), &c.2, c.3, &c.4))
    }

    pub fn queries(
        &self,
    ) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>, u32, &Vec<String>)> {
        self.queries.iter().map(|c| (c.0.as_str(), &c.2, c.3, &c.4))
    }

    pub fn events(&self) -> impl Iterator<Item = &Variant<PortableForm>> {
        self.events.iter()
    }

    fn service_funcs(
        registry: &PortableRegistry,
        func_type_ids: impl Iterator<Item = u32>,
    ) -> Result<(Vec<ServiceFuncMeta>, Vec<ServiceFuncMeta>)> {
        let mut funcs_meta = Vec::new();
        let mut overriden_funcs_meta = Vec::new();
        for func_type_id in func_type_ids {
            for func_descr in any_funcs(registry, func_type_id)? {
                if func_descr.fields.len() != 2 {
                    return Err(Error::FuncMetaIsInvalid(format!(
                        "func `{}` has invalid number of fields",
                        func_descr.name
                    )));
                }
                let func_params_type = registry.resolve(func_descr.fields[0].ty.id).unwrap_or_else(
                    || {
                        panic!(
                            "func params type id {} not found while it was registered previously",
                            func_descr.fields[0].ty.id
                        )
                    },
                );
                if let TypeDef::Composite(func_params_type) = &func_params_type.type_def {
                    let func_meta = ServiceFuncMeta(
                        func_descr.name.to_string(),
                        func_descr.fields[0].ty.id,
                        func_params_type.fields.to_vec(),
                        func_descr.fields[1].ty.id,
                        func_descr.docs.iter().map(|s| s.to_string()).collect(),
                    );
                    // if base service had a func with the same name, it is considered overridden
                    // and is stored separately
                    if !funcs_meta
                        .iter()
                        .any(|fm: &ServiceFuncMeta| fm.0 == func_meta.0)
                    {
                        funcs_meta.push(func_meta);
                    } else {
                        overriden_funcs_meta.push(func_meta);
                    }
                } else {
                    return Err(Error::FuncMetaIsInvalid(format!(
                        "func `{}` params type is not a composite",
                        func_descr.name
                    )));
                }
            }
        }
        Ok((funcs_meta, overriden_funcs_meta))
    }

    fn event_variants(
        registry: &PortableRegistry,
        events_type_ids: impl Iterator<Item = u32>,
    ) -> Result<Vec<Variant<PortableForm>>> {
        let mut events_variants = Vec::new();
        for events_type_id in events_type_ids {
            let events = registry.resolve(events_type_id).unwrap_or_else(|| {
                panic!(
                    "events type id {events_type_id} not found while it was registered previously"
                )
            });
            if let TypeDef::Variant(variant) = &events.type_def {
                for event_variant in &variant.variants {
                    if events_variants
                        .iter()
                        .any(|ev: &Variant<PortableForm>| ev.name == event_variant.name)
                    {
                        return Err(Error::EventMetaIsAmbiguous(format!(
                            "events type id {} contains ambiguous event variant `{}`",
                            events_type_id, event_variant.name
                        )));
                    }
                    events_variants.push(event_variant.clone());
                }
            } else {
                return Err(Error::EventMetaIsInvalid(format!(
                    "events type id {events_type_id} references a type that is not a variant"
                )));
            }
        }
        Ok(events_variants)
    }
}

fn any_funcs(
    registry: &PortableRegistry,
    func_type_id: u32,
) -> Result<impl Iterator<Item = &Variant<PortableForm>>> {
    let funcs = registry.resolve(func_type_id).unwrap_or_else(|| {
        panic!("func type id {func_type_id} not found while it was registered previously")
    });
    if let TypeDef::Variant(variant) = &funcs.type_def {
        Ok(variant.variants.iter())
    } else {
        Err(Error::FuncMetaIsInvalid(format!(
            "func type id {func_type_id} references a type that is not a variant"
        )))
    }
}

/*
Special impls see in `scale-info::impls`
Type:
- path - path to type declaration relative crate root
- type params - generic type parameters
- type def - actual type definition
- docs - vector of documentation strings 

Field:
- name - field name (optional)
- ty - field type (TypeId)
- type_name - field type name (optional)
- docs - field documentation (optional)

1. u8-u128, i8-i128, bool, char, str, String -> Primitive
2. Structs -> Composite:
    - Unit struct -> Fields [], 
    - Tuple struct -> Fields are elements inside struct (fields in normal struct, or types in tuple struct)
                      Has multiple fields, hidden under type_name: Option<type>
    - Struct with named fields -> [ name (field name), ty (type id), type_name: Option<type>, docs (docs for the field) ]

3. Arrays - Array { len, type_param (TypeId) }
4. [T] - dynamic arrays -> Sequence { type_param (TypeId) }, TypeId should be stored somewhere
5. Tuples:
    - () -> Tuple type def with zero fields
    - (T1,) -> Tuple type def with 1 field [ TypeId - id of the type ] 
    - (T1, T2,) -> Tuple type def with 2 fields [ TypeId - id of the type, TypeId - id of the type ]
6. Enums -> Variant which is the Vector of Variant { name: str, fields: Field } 

Generics:


PhantomData fields are missed
lifetimes are missed
&Type is always Type for ScaleInfo
*/

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::TypeInfo;
    use std::{iter, collections::BTreeMap};

    mod utils {
        use super::*;

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum SimpleCtors {
            SimpleCtor(SimpleCtorParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) struct SimpleCtorParams {
            f1: u32,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) struct SimpleFunctionParams {
            f1: u32,
        }
            
        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum NoCommands {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum NoQueries {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum NoEvents {}
    }

    // #[test]
    // fn test_meta() {
    //     use sails_idl_meta::ProgramMeta;
    //     use demo::DemoProgram; // todo [sab] not indexed
    //     let res = ExpandedProgramMeta2::new(
    //         "Demo".to_string(),
    //         DemoProgram::constructors(),
    //         DemoProgram::services(),
    //     ).unwrap();

    //     println!("DemoProgram meta: {}", serde_json::to_string_pretty(&res).unwrap());
    // }

    /// Test various constructor validation errors
    #[test]
    fn ctor_validation_errors() {
        // Define all constructor error test types
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum NonCompositeArgsCtors {
            CtorWithInvalidArgTypes(u32), // u32 is not composite, should cause error
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum NamelessFieldsCtors {
            CtorWithNamelessArgs(NamelessFieldParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NamelessFieldParams(u32, String);

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum NoArgsCtors {
            CtorWithNoArgs,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum TooManyArgsCtors {
            CtorWithResult(ValidParams, String), // Should have exactly 1 field, not 2
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ValidParams {
            pub param1: u32,
        }

        // Helper function to test constructor validation errors
        fn test_ctor_error<T: TypeInfo + 'static>(expected_error_msg: &str) {
            let result = ExpandedProgramMeta2::new(
                "TestProgram".to_string(),
                MetaType::new::<T>(),
                iter::empty(),
            );

            assert!(result.is_err());
            let Err(Error::FuncMetaIsInvalid(msg)) = result else {
                panic!("Expected FuncMetaIsInvalid error, got {:?}", result);
            };
            assert_eq!(msg.as_str(), expected_error_msg);
        }

        // Test all error scenarios
        test_ctor_error::<NonCompositeArgsCtors>(
            "ctor `CtorWithInvalidArgTypes` params type is not a composite"
        );

        test_ctor_error::<NamelessFieldsCtors>(
            "ctor CtorWithNamelessArgs has nameless argument"
        );

        test_ctor_error::<NoArgsCtors>(
            "ctor `CtorWithNoArgs` has invalid number of fields"
        );

        test_ctor_error::<TooManyArgsCtors>(
            "ctor `CtorWithResult` has invalid number of fields"
        );
    }

    /// Test that returned program meta has result_ty == None for all constructors in program IDL section
    #[test]
    fn ctors_result_ty_none() {

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ValidConstructors {
            Zero(ZeroParams),
            One(OneParam),
            Three(ThreeParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ZeroParams {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct OneParam {
            pub actor: ActorId,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ThreeParams {
            pub code: CodeId,
            pub name: String,
            pub num: NonZeroU256,
        }

        let result = ExpandedProgramMeta2::new(
            "TestProgram".to_string(),
            MetaType::new::<ValidConstructors>(),
            iter::empty(),
        );

        assert!(result.is_ok());
        let meta = result.unwrap();

        // Check that all constructors have result_ty == None
        assert_eq!(meta.program.ctors.len(), 3);

        for ctor in &meta.program.ctors {
            assert!(ctor.result_ty.is_none(), 
                "Constructor '{}' should have result_ty == None, but got {:?}", 
                ctor.name, ctor.result_ty);
        }
    }

    /// Test successful creation with valid constructors and services
    #[test]
    fn ctor_simple_positive_test() {
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum Ctors {
            Ctor(InitParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct InitParams {
            pub initial_value: u32,
        }

        let result = ExpandedProgramMeta2::new(
            "TestProgram".to_string(),
            MetaType::new::<Ctors>(),
            std::iter::empty(),
        );

        assert!(result.is_ok());
        let meta = result.unwrap();

        let ctors_json = serde_json::to_value(&meta.program.ctors)
            .expect("Should serialize to JSON");
        assert_eq!(
            ctors_json,
            serde_json::json!([
                {
                    "name": "Ctor",
                    "args": [
                        {
                            "name": "initial_value",
                            "type_idx": meta.program.ctors[0].args[0].ty,
                        }
                    ],
                    "docs": [],
                }
            ])
        );
    }

    // #[test]
    // #[ignore]
    // fn test_json_serialization() {
    //     // todo [sab] features to test:
    //     // 2. base services
    //     // 3. events
    //     // 4. ctors: no args, multiple args
    //     // 5. services:
    //     // - unit res for extended/base services


    //     let meta = ExpandedProgramMeta2::new(
    //         "TestProgram".to_string(),
    //         MetaType::new::<utils::SimpleCtors>(),
    //         std::iter::empty(),
    //     ).unwrap();

    //     let json = serde_json::to_value(&meta).expect("Should serialize to JSON");

    //     // Check JSON structure
    //     assert!(json.get("program").is_some());
    //     assert!(json.get("services").is_some());

    //     let program = json.get("program").unwrap();
    //     assert_eq!(program.get("name").unwrap(), "TestProgram");
        
    //     let ctors = program.get("ctors").unwrap().as_array().unwrap();
    //     assert_eq!(ctors.len(), 1);
        
    //     let ctor = &ctors[0];
    //     assert_eq!(ctor.get("name").unwrap(), "TestCtor");
    //     assert!(ctor.get("result_ty").is_none()); // Should not be present in JSON when None
        
    //     let args = ctor.get("args").unwrap().as_array().unwrap();
    //     assert_eq!(args.len(), 1);
    //     assert_eq!(args[0].get("name").unwrap(), "test_field");
    //     assert!(args[0].get("type_idx").is_some());
    // }

    /// Test that services with only primitive/builtin types have empty types sections
    #[test] 
    fn service_non_user_defined_types_excluded() {
        struct Service1;
        impl sails_idl_meta::ServiceMeta for Service1 {
            type CommandsMeta = CommandsWithNonUserDefinedArgs;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service2;
        impl sails_idl_meta::ServiceMeta for Service2 {
            type CommandsMeta = CommandWithUserDefinedArgs;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service3;
        impl sails_idl_meta::ServiceMeta for Service3 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = CommandsWithNonUserDefinedArgs;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service4;
        impl sails_idl_meta::ServiceMeta for Service4 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = CommandWithUserDefinedArgs;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service5;
        impl sails_idl_meta::ServiceMeta for Service5 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventsWithNonUserDefinedArgs;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service6;
        impl sails_idl_meta::ServiceMeta for Service6 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventsWithUserDefinedArgs;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum CommandWithUserDefinedArgs {
            Cmd1(UserDefinedParams, String),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct UserDefinedParams {
            pub arg1: NonUserDefinedArgs,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum CommandsWithNonUserDefinedArgs {
            Cmd1(NonUserDefinedArgs, String),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum EventsWithUserDefinedArgs {
            Event1(NonUserDefinedArgs),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NonUserDefinedArgs {  
            pub number: u32,
            pub flag: bool,
            pub text: String,
            pub actor: ActorId,
            pub option_val: Option<u8>,
            pub result_val: Result<u16, String>,
            pub map: BTreeMap<String, u32>,
            pub code: CodeId,
            pub message: MessageId,
            pub h160: H160,
            pub h256: H256,
            pub u256: U256,
            pub non_zero_u8: NonZeroU8,
            pub non_zero_u16: NonZeroU16,
            pub non_zero_u32: NonZeroU32,
            pub non_zero_u64: NonZeroU64,
            pub non_zero_u128: NonZeroU128,
            pub non_zero_u256: NonZeroU256,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum EventsWithNonUserDefinedArgs {
            Event1 {
                number: u32,
                flag: bool,
                text: String,
                actor: ActorId,
                option_val: Option<u8>,
                result_val: Result<u16, String>,
                map: BTreeMap<String, u32>,
                code: CodeId,
                message: MessageId,
                h160: H160,
                h256: H256,
                u256: U256,
                non_zero_u8: NonZeroU8,
                non_zero_u16: NonZeroU16,
                non_zero_u32: NonZeroU32,
                non_zero_u64: NonZeroU64,
                non_zero_u128: NonZeroU128,
                non_zero_u256: NonZeroU256,
            },
        }

        let internal_check = |service1: AnyServiceMeta, service2: AnyServiceMeta| {
            let meta = ExpandedProgramMeta2::new(
                "TestProgram".to_string(),
                MetaType::new::<utils::SimpleCtors>(),
                vec![("Service1", service1), ("Service2", service2)].into_iter(),
            ).unwrap_or_else(|e| panic!("Failed to create expanded meta: {:?}", e));

            assert_eq!(meta.services.len(), 2);

            let service_1 = &meta.services[0];
            assert_eq!(service_1.name, "Service1");
            assert_eq!(service_1.types.len(), 0, "Service with only primitive/builtin types should have empty types section");

            let service_2 = &meta.services[1];
            assert_eq!(service_2.name, "Service2");
            assert_eq!(service_2.types.len(), 1, "Service with user-defined types should have non-empty types section");
            assert_eq!(service_2.types[0].ty.path.ident().unwrap(), "NonUserDefinedArgs");
        };

        internal_check(
            AnyServiceMeta::new::<Service1>(),
            AnyServiceMeta::new::<Service2>(),
        );

        internal_check(
            AnyServiceMeta::new::<Service3>(),
            AnyServiceMeta::new::<Service4>(),
        );

        internal_check(
            AnyServiceMeta::new::<Service5>(),
            AnyServiceMeta::new::<Service6>(),
        );

        // todo [sab] add same test but with base services
    }

    /// Test that service functions with () result have result_ty == None
    #[test]
    fn service_unit_result() {
        struct TestServiceMeta;
        impl sails_idl_meta::ServiceMeta for TestServiceMeta {
            type CommandsMeta = TestCommands;
            type QueriesMeta = TestQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum TestCommands {
            UnitCmd(utils::SimpleFunctionParams, ()), // Returns unit type - no result_ty
            NonUnitCmd(utils::SimpleFunctionParams, String), // Returns non-unit type
            WithUnitCmd(utils::SimpleFunctionParams, Result<(), u32>), // Returns value containing unit type
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum TestQueries {
            UnitQuery(utils::SimpleFunctionParams, ()), // Returns unit type - no result_ty
            NonUnitQuery(utils::SimpleFunctionParams, u32), // Returns non-unit type
            WithUnitQuery(utils::SimpleFunctionParams, Result<(), u32>), // Returns value containing unit type
        }

        let meta = ExpandedProgramMeta2::new(
            "TestProgram".to_string(),
            MetaType::new::<utils::SimpleCtors>(),
            vec![("TestService", AnyServiceMeta::new::<TestServiceMeta>())].into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {:?}", e));

        let service = &meta.services[0];

        // Check commands
        let check_fn_result_ty = |fns: &[FunctionIdl2Data]| {
            for f in fns {
                match f.name.as_str() {
                    "UnitCmd" | "UnitQuery" => {
                        assert!(f.result_ty.is_none(), "Command returning () should have result_ty == None");
                    }
                    "NonUnitCmd" | "NonUnitQuery" => {
                        assert!(f.result_ty.is_some(), "Command returning non-unit should have result_ty == Some");
                    }
                    "WithUnitCmd" | "WithUnitQuery" => {
                        assert!(f.result_ty.is_some(), "Command returning Result<(), T> should have result_ty == Some");
                    }
                    _ => unimplemented!("Unexpected function name: {}", f.name),
                }
            }
        };

        check_fn_result_ty(&service.functions.commands);
        check_fn_result_ty(&service.functions.queries);

        // Test JSON serialization - result_ty should not appear in JSON when None
        let json = serde_json::to_value(&service).expect("Should serialize to JSON");
        let (commands, queries) = json
            .get("functions")
            .and_then(|v| 
                v
                    .get("commands")
                    .and_then(|c| v.get("queries").map(|q| (c, q)))
            )
            .and_then(|(c, q)| q
                .as_array()
                .and_then(|q_arr| c.as_array().map(|c_arr| (c_arr, q_arr)))
            )
            .unwrap();

        let check_json_result_ty = |fns_json: &[serde_json::Value]| {
            for fn_json in fns_json {
                let name = fn_json.get("name").unwrap().as_str().unwrap();
                match name {
                    "UnitCmd" | "UnitQuery" => {
                        assert!(fn_json.get("result_ty").is_none(), "{} returning () should not have result_ty in JSON", name);
                    }
                    "NonUnitCmd" | "NonUnitQuery" => {
                        assert!(fn_json.get("result_ty").is_some(), "{} returning non-unit should have result_ty in JSON", name);
                    }
                    "WithUnitCmd" | "WithUnitQuery" => {
                        assert!(fn_json.get("result_ty").is_some(), "{} returning Result<(), T> should have result_ty in JSON", name);
                    }
                    _ => unimplemented!("Unexpected function name in JSON: {}", name),
                }
            }
        };

        check_json_result_ty(commands);
        check_json_result_ty(queries);
    }

    /// Test error when commands/queries are not variant types
    #[test]
    fn service_functions_non_variant_error() {
        struct NotVariantCommandsService;
        impl sails_idl_meta::ServiceMeta for NotVariantCommandsService {
            type CommandsMeta = NotVariantCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct NotVariantQueriesService;
        impl sails_idl_meta::ServiceMeta for NotVariantQueriesService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = NotVariantQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NotVariantCommands {
            pub field: u32,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NotVariantQueries(u32);

        let internal_check = |service: AnyServiceMeta, expected_msg: &str| {
            let result = ExpandedProgramMeta2::new(
                "TestProgram".to_string(),
                MetaType::new::<utils::SimpleCtors>(),
                vec![("TestService", service)].into_iter(),
            );
            assert!(result.is_err());
            let Err(Error::FuncMetaIsInvalid(msg)) = result else {
                panic!("Expected FuncMetaIsInvalid error, got {:?}", result);
            };
            assert_eq!(msg.as_str(), expected_msg);
        };

        internal_check(
            AnyServiceMeta::new::<NotVariantCommandsService>(),
            "Commands type is not a variant",
        );

        internal_check(
            AnyServiceMeta::new::<NotVariantQueriesService>(),
            "Queries type is not a variant",
        );
    }

    /// Test error when service variant doesn't have exactly 2 fields
    #[test]
    fn service_variant_field_count_error() {
        struct InvalidCommandsService1;
        impl sails_idl_meta::ServiceMeta for InvalidCommandsService1 {
            type CommandsMeta = BadCommands1;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidCommandsService2;
        impl sails_idl_meta::ServiceMeta for InvalidCommandsService2 {
            type CommandsMeta = BadCommands2;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }
        
        struct InvalidQueriesService1;
        impl sails_idl_meta::ServiceMeta for InvalidQueriesService1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = BadQueries1;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidQueriesService2;
        impl sails_idl_meta::ServiceMeta for InvalidQueriesService2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = BadQueries2;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        // Commands with wrong number of fields
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands1 {
            OneField(u32), // Should have 2 fields (params, result)
            ValidCmd(utils::SimpleFunctionParams, String), // Valid command for control
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands2 {
            ThreeFields(u32, String, bool), // Should have 2 fields (params, result)
            ValidCmd(utils::SimpleFunctionParams, String), // Valid command for control
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadQueries1 {
            OneField(u32), // Should have 2 fields (params, result)
            ValidQuery(utils::SimpleFunctionParams, String), // Valid query for control
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadQueries2 {
            ThreeFields(u32, String, bool), // Should have 2 fields (params, result)
            ValidQuery(utils::SimpleFunctionParams, String), // Valid query for control
        }

        let internal_check = |service: AnyServiceMeta, expected_msg: &str| {
            let result = ExpandedProgramMeta2::new(
                "TestProgram".to_string(),
                MetaType::new::<utils::SimpleCtors>(),
                vec![("TestService", service)].into_iter(),
            );

            assert!(result.is_err());
            let Err(Error::FuncMetaIsInvalid(msg)) = result else {
                panic!("Expected FuncMetaIsInvalid error, got {:?}", result);
            };
            assert_eq!(msg.as_str(), expected_msg);
        };

        internal_check(
            AnyServiceMeta::new::<InvalidCommandsService1>(),
            "command `OneField` has invalid number of fields, expected 2, got 1",
        );

        internal_check(
            AnyServiceMeta::new::<InvalidQueriesService1>(),
            "query `OneField` has invalid number of fields, expected 2, got 1",
        );

        internal_check(
            AnyServiceMeta::new::<InvalidCommandsService2>(),
            "command `ThreeFields` has invalid number of fields, expected 2, got 3",
        );

        internal_check(
            AnyServiceMeta::new::<InvalidQueriesService2>(),
            "query `ThreeFields` has invalid number of fields, expected 2, got 3",
        );
    }

    /// Test error when service method params are not composite
    #[test]
    fn service_params_non_composite_error() {
        struct TestServiceMeta;
        impl sails_idl_meta::ServiceMeta for TestServiceMeta {
            type CommandsMeta = BadCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        // Commands where the first field (params) is not composite
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands {
            BadCmd(u32, String), // First field should be composite (struct), not primitive
        }

        let result = ExpandedProgramMeta2::new(
            "TestProgram".to_string(),
            MetaType::new::<utils::SimpleCtors>(),
            vec![("TestService", AnyServiceMeta::new::<TestServiceMeta>())].into_iter(),
        );

        assert!(result.is_err());
        let Err(Error::FuncMetaIsInvalid(msg)) = result else {
            panic!("Expected FuncMetaIsInvalid error, got {:?}", result);
        };
        assert_eq!(msg.as_str(), "command `BadCmd` args type is not a composite");
    }

    /// Test error when service method params have nameless fields
    #[test]
    fn service_params_nameless_fields_error() {
        struct BadServiceMeta;
        impl sails_idl_meta::ServiceMeta for BadServiceMeta {
            type CommandsMeta = BadCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands {
            BadCmd(NamelessParams, String),
        }

        // Tuple struct with nameless fields for params
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NamelessParams(u32, String);

        let result = ExpandedProgramMeta2::new(
            "TestProgram".to_string(),
            MetaType::new::<utils::SimpleCtors>(),
            vec![("TestService", AnyServiceMeta::new::<BadServiceMeta>())].into_iter(),
        );

        assert!(result.is_err());
        let Err(Error::FuncMetaIsInvalid(msg)) = result else {
            panic!("Expected FuncMetaIsInvalid error, got {:?}", result);
        };
        assert_eq!(msg.as_str(), "command `BadCmd` arg must have a name");
    }

    #[test]
    fn service_function_variations_positive_test() {
        struct ServiceWithOneCommand;
        impl sails_idl_meta::ServiceMeta for ServiceWithOneCommand {
            type CommandsMeta = OneFunction;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceWithOneQuery;
        impl sails_idl_meta::ServiceMeta for ServiceWithOneQuery {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = OneFunction;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceWithNoFunctions;
        impl sails_idl_meta::ServiceMeta for ServiceWithNoFunctions {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum OneFunction {
            Fn1(utils::SimpleFunctionParams, String),
        }

        let internal_check = |service: AnyServiceMeta, expected_commands_count: usize, expected_queries_count: usize| {
            let meta = ExpandedProgramMeta2::new(
                "TestProgram".to_string(),
                MetaType::new::<utils::SimpleCtors>(),
                vec![("TestService", service)].into_iter(),
            ).unwrap_or_else(|e| panic!("Failed to create expanded meta: {:?}", e));

            let service_meta = &meta.services[0];
            assert_eq!(service_meta.functions.commands.len(), expected_commands_count, "Service should have {} command(s)", expected_commands_count);
            assert_eq!(service_meta.functions.queries.len(), expected_queries_count, "Service should have {} query(s)", expected_queries_count);

            if !service_meta.functions.commands.is_empty() {
                let cmd = &service_meta.functions.commands[0];
                assert_eq!(cmd.name, "Fn1", "Command name should be 'Fn1'");
            } else if !service_meta.functions.queries.is_empty() {
                let query = &service_meta.functions.queries[0];
                assert_eq!(query.name, "Fn1", "Query name should be 'Fn1'");
            }
        };

        internal_check(
            AnyServiceMeta::new::<ServiceWithOneCommand>(),
            1,
            0,
        );

        internal_check(
            AnyServiceMeta::new::<ServiceWithOneQuery>(),
            0,
            1,
        );

        internal_check(
            AnyServiceMeta::new::<ServiceWithNoFunctions>(),
            0,
            0,
        );

        struct Service;
        impl sails_idl_meta::ServiceMeta for Service {
            type CommandsMeta = ServiceCommands;
            type QueriesMeta = ServiceQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ServiceCommands {
            NoArgsCmd(NoArgs, String),
            OneArgCmd(OneArg, u32),
            MultiArgsCmd(MultiArgs, bool),
            NoResultCmd(OneArg, ()),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ServiceQueries {
            NoArgsQuery(NoArgs, String),
            OneArgQuery(OneArg, u32),
            MultiArgsQuery(MultiArgs, bool),
            NoResultQuery(OneArg, ()),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NoArgs;

        // One argument
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct OneArg {
            pub arg1: u32,
        }

        // Multiple arguments
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct MultiArgs {
            pub arg1: u32,
            pub arg2: String,
            pub arg3: bool,
        }

        let meta = ExpandedProgramMeta2::new(
            "TestProgram".to_string(),
            MetaType::new::<utils::SimpleCtors>(),
            vec![("TestService", AnyServiceMeta::new::<Service>())].into_iter(),
        ).unwrap_or_else(|e| panic!("Failed to create expanded meta: {:?}", e));

        assert_eq!(meta.services.len(), 1);
        let service = &meta.services[0];

        let internal_check = |fns: &[FunctionIdl2Data]| {
            for f in fns {
                match f.name.as_str() {
                    "NoArgsCmd" | "NoArgsQuery" => {
                        assert_eq!(f.args.len(), 0, "{} should have no arguments", f.name);
                    }
                    "OneArgCmd" | "OneArgQuery" => {
                        assert_eq!(f.args.len(), 1, "{} should have one argument", f.name);
                        assert_eq!(f.args[0].name, "arg1", "Argument name should be 'arg1'");
                    }
                    "MultiArgsCmd" | "MultiArgsQuery" => {
                        assert_eq!(f.args.len(), 3, "{} should have three arguments", f.name);
                        assert_eq!(f.args[0].name, "arg1", "First argument name should be 'arg1'");
                        assert_eq!(f.args[1].name, "arg2", "Second argument name should be 'arg2'");
                        assert_eq!(f.args[2].name, "arg3", "Third argument name should be 'arg3'");
                    }
                    "NoResultCmd" | "NoResultQuery" => {
                        assert!(f.result_ty.is_none(), "{} should have no result type", f.name);
                    }
                    _ => unimplemented!("Unexpected function name: {}", f.name),
                }
            }
        };

        internal_check(&service.functions.commands);
        internal_check(&service.functions.queries);
    }

//     #[test]
//     fn test_scale_info() {

//         /// Some docs here
//         #[derive(TypeInfo)]
//         struct TupleStruct(u8);
//         #[derive(TypeInfo)]
//         struct TupleStruct2(String);
//         #[derive(TypeInfo)]
//         struct TupleStruct3([u8; 7]);
//         #[derive(TypeInfo)]
//         struct TupleStruct4(H256);
//         /// And some docs here
//         #[derive(TypeInfo)]
//         struct TupleStruct5(Vec<u8>);
//         #[derive(TypeInfo)]
//         struct TupleStruct6(Vec<H256>);
//         #[derive(TypeInfo)]
//         struct UnitStruct;

//         #[derive(TypeInfo)]
//         struct StructWithFields {
//             field1: u8,
//             field2: String,
//             field3: [u8; 7],
//             field4: H256,
//             field5: Vec<u8>,
//             field6: Vec<H256>,
//             field7: TupleStruct,
//             field8: TupleStruct5,
//         }

//         #[derive(TypeInfo)]
//         struct TwoElementsTupleStruct(u8, String);

//         println!("i32 info: {:?}\n", <i32 as TypeInfo>::type_info());
//         println!("u8 info: {:?}\n", <u8 as TypeInfo>::type_info());
//         println!("H256 info: {:?}\n", <H256 as TypeInfo>::type_info());
//         println!("ActorId info: {:?}\n", <ActorId as TypeInfo>::type_info());
//         println!("CodeId info: {:?}\n", <CodeId as TypeInfo>::type_info());

//         let type_info = <[u8; 7] as TypeInfo>::type_info();
//         println!("\n\n Array info: {:?}\n", type_info);

//         println!("TupleStruct info: {:?}\n", <TupleStruct as TypeInfo>::type_info());
//         println!("TupleStruct2 info: {:?}\n", <TupleStruct2 as TypeInfo>::type_info());
//         println!("TupleStruct3 info: {:?}\n", <TupleStruct3 as TypeInfo>::type_info());
//         println!("TupleStruct4 info: {:?}\n", <TupleStruct4 as TypeInfo>::type_info());
//         println!("TupleStruct5 info: {:?}\n", <TupleStruct5 as TypeInfo>::type_info());
//         println!("TupleStruct6 info: {:?}\n", <TupleStruct6 as TypeInfo>::type_info());
//         println!("UnitStruct info: {:?}\n", <UnitStruct as TypeInfo>::type_info());

//         println!("TwoElementsTupleStruct info: {:#?}\n", <TwoElementsTupleStruct as TypeInfo>::type_info());

//         println!("StructWithFields info: {:#?}\n", <StructWithFields as TypeInfo>::type_info());

//         println!("\n\n\n");
//         println!("Vector info: {:?}\n", <Vec<String> as TypeInfo>::type_info());
//         println!("Array info: {:?}\n", <[H256; 4] as TypeInfo>::type_info());
//         println!("Map info: {:?}\n", <std::collections::BTreeMap<String, H256> as TypeInfo>::type_info());
//         println!("Set info: {:?}\n", <std::collections::BTreeSet<H256> as TypeInfo>::type_info());

//         println!("Tuple info: {:?}", <() as TypeInfo>::type_info());
//         println!("Tuple-1 info: {:?}", <(u8,) as TypeInfo>::type_info());
//         println!("Tuple-2 info: {:?}", <(u8, String) as TypeInfo>::type_info());
//         println!("Tuple-3 info: {:?}", <(u8, String, H256) as TypeInfo>::type_info());
//         println!("Tuple-4 info: {:?}", <(String, [u8; 9], H256, Vec<H256>) as TypeInfo>::type_info());

//         #[derive(TypeInfo)]
//         enum Enum1 {}
//         #[derive(TypeInfo)]
//         enum Enum2 { A }
//         #[derive(TypeInfo)]
//         enum Enum3 { A, B, C }
//         #[derive(TypeInfo)]
//         enum Enum4 { A(u8), B(Vec<H256>), C(H256), D([u8; 4]), E((String, ActorId)) }
//         #[derive(TypeInfo)]
//         enum Enum5 { A { f1: u8, f2: String }, B(Vec<u8>), C }

//         println!("Enum1 info: {:?}\n", <Enum1 as TypeInfo>::type_info());
//         println!("Enum2 info: {:?}\n", <Enum2 as TypeInfo>::type_info());
//         println!("Enum3 info: {:?}\n", <Enum3 as TypeInfo>::type_info());
//         println!("Enum4 info: {:?}\n", <Enum4 as TypeInfo>::type_info());
//         println!("Enum5 info: {:?}\n", <Enum5 as TypeInfo>::type_info());

//         println!("Option info: {:?}\n", <Option<H256> as TypeInfo>::type_info());
//         println!("Result info: {:?}\n", <Result<H256, String> as TypeInfo>::type_info());

//         #[derive(TypeInfo)]
//         struct WithGenerics1<T> {
//             f1: Enum1,
//             f2: T
//         }

//         #[derive(TypeInfo)]
//         struct WithGenericsBounded<T: std::fmt::Debug> {
//             f1: Enum2,
//             f2: T
//         }

//         #[derive(TypeInfo)]
//         struct WithMultipleGenerics<T1, T2, T3> {
//             f1: T1,
//             f2: T2,
//             f3: T3,
//         }

//         println!("\n\n\n");

//         println!("WithGenerics1 info: {:?}\n", <WithGenerics1<StructWithFields> as TypeInfo>::type_info());
//         println!("String info: {:?}\n", TypeId::of::<String>());
//         println!("WithGenerics1 ANOTHER info: {:?}\n", <WithGenerics1<String> as TypeInfo>::type_info());
//         println!("WithGenericsBounded info: {:?}\n", <WithGenericsBounded<String> as TypeInfo>::type_info());
//         println!("WithMultipleGenerics info: {:?}\n", <WithMultipleGenerics<StructWithFields, String, H256> as TypeInfo>::type_info());

//         // Pha
//         #[derive(TypeInfo)]
//         struct M<T> {
//             _marker1: std::marker::PhantomData<T>,
//         }

//         println!("M info: {:?}\n", <M<String> as TypeInfo>::type_info());

//         #[derive(TypeInfo)]
//         struct WithLifetime<'a, T> {
//             f1: &'a str,
//             f2: T,
//         }

//         println!("WithLifetime info: {:?}\n", <WithLifetime<'_, String> as TypeInfo>::type_info());
//     }
}
