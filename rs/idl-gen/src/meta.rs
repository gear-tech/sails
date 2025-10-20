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

use std::collections::HashSet;

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

#[derive(serde::Serialize)]
pub(crate) struct ExpandedProgramMeta2 {
    program: ProgramIdlSection,
    services: Vec<ServiceSection>,
}

impl ExpandedProgramMeta2 {
    pub fn new(name: String, ctors: MetaType, services: impl Iterator<Item = (&'static str, AnyServiceMeta)>,) -> Result<Self> {
        // Create registry
        let mut ctor_registry = Registry::new();
        let ctor_registry_id = ctor_registry.register_type(&ctors).id;

        let services_registry_ids = services.map(|(service_name, meta)| {
            let mut registry = Registry::new();
            let commands = meta.commands();
            let commands_registry_id = registry.register_type(&commands).id;

            let queries = meta.queries();
            let queries_registry_id = registry.register_type(&queries).id;

            let events = meta.events();
            let events_registry_id = registry.register_type(&events).id;

            // todo [sab] base services and their events
            (registry, service_name, commands_registry_id, queries_registry_id, events_registry_id,)
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
        services_registry_ids.for_each(|(registry, name, commands_id, queries_id, events_id)| {
            // Handle commands
            let mut generated_types = HashSet::new();
            generated_types.insert(commands_id);
            generated_types.insert(queries_id);
            generated_types.insert(events_id);

            let service_portable_registry = PortableRegistry::from(registry);
            let mut commands = Vec::new();
            let commands_type = service_portable_registry
                .resolve(commands_id)
                .expect("command type was added previously; qed.");

            let TypeDef::Variant(ref variants) = commands_type.type_def else {
                todo!("return error - invalid meta impl for commands")
            };

            for variant in &variants.variants {
                if variant.fields.len() != 2 {
                    todo!("return error - invalid meta impl for command")
                }

                // Add to generated types __*Params type of service's variant in `CommandsMeta`
                generated_types.insert(variant.fields[0].ty.id);

                // Take args (fields of __*Params type)
                let args_type = service_portable_registry
                    .resolve(variant.fields[0].ty.id)
                    .expect("args type was added previously; qed.");
                let TypeDef::Composite(args_type) = &args_type.type_def else {
                    todo!("return error - invalid meta impl for command args")
                };
                let args = args_type.fields.iter().map(|f| {
                    let name = f.name
                        .as_ref()
                        .expect("command arg must have a name")
                        .to_string();
                    FuncArgIdl2 {
                        name,
                        ty: f.ty.id,
                    }
                }).collect::<Vec<_>>();

                // Take result type
                let result_ty = variant.fields[1].ty.id;

                let command = FunctionIdl2Data {
                    name: variant.name.to_string(),
                    args,
                    result_ty: Some(result_ty),
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
                todo!("return error - invalid meta impl for queries")
            };

            for variant in &variants.variants {
                if variant.fields.len() != 2 {
                    todo!("return error - invalid meta impl for query")
                }

                // Add to generated types __*Params type of service's variant in `QueriesMeta`
                generated_types.insert(variant.fields[0].ty.id);

                // Take args (fields of __*Params type)
                let args_type = service_portable_registry
                    .resolve(variant.fields[0].ty.id)
                    .expect("args type was added previously; qed.");
                let TypeDef::Composite(args_type) = &args_type.type_def else {
                    todo!("return error - invalid meta impl for query args")
                };
                let args = args_type.fields.iter().map(|f| {
                    let name = f.name
                        .as_ref()
                        .expect("query arg must have a name")
                        .to_string();
                    FuncArgIdl2 {
                        name,
                        ty: f.ty.id,
                    }
                }).collect::<Vec<_>>();

                // Take result type
                let result_ty = variant.fields[1].ty.id;

                let query = FunctionIdl2Data {
                    name: variant.name.to_string(),
                    args,
                    result_ty: Some(result_ty),
                    docs: variant.docs.iter().map(|s| s.to_string()).collect(),
                };

                queries.push(query);
            }

            // Handle events
            let events_type = service_portable_registry
                .resolve(events_id)
                .expect("event type was added previously; qed.");

            let TypeDef::Variant(ref variants) = events_type.type_def else {
                todo!("return error - invalid meta impl for events")
            };
            let events = variants.variants.clone();

            let types = service_portable_registry
                .types
                .iter()
                .filter(|ty| {
                    // todo [sab] test that skips primitive types and Result/Option + builtin types
                    !ty.ty.path.namespace().is_empty() && !generated_types.contains(&ty.id)
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
        });

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
                        let mut args = Vec::with_capacity(params_type.fields.len());
                        for f in &params_type.fields {
                            let name = f.name
                                .map(|s| s.to_string())
                                .expect("constructor argument must have a name");
                            args.push(FuncArgIdl2 {
                                name: name,
                                ty: f.ty.id,
                            });
                        }
                        Ok(FunctionIdl2Data {
                            name: c.name.to_string(),
                            args,
                            result_ty: None,
                            docs: c.docs.iter().map(|s| s.to_string()).collect(),
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
//     use std::any::TypeId;

//     use scale_info::TypeInfo;
//     use serde::de;

    use super::*;

    #[test]
    fn test_meta() {
        use sails_idl_meta::ProgramMeta;
        use demo::DemoProgram; // todo [sab] not indexed
        let res = ExpandedProgramMeta2::new(
            "Demo".to_string(),
            DemoProgram::constructors(),
            DemoProgram::services(),
        ).unwrap();

        println!("DemoProgram meta: {}", serde_json::to_string_pretty(&res).unwrap());

//         let type_id = TypeId::of::<[u8; 32]>();
//         let meta = ExpandedProgramMeta::new(Some(DemoProgram::constructors()), DemoProgram::services()).unwrap();

//         meta.type_names();
    }

//     fn print_json_string<T: TypeInfo + 'static>(msg: &str) {
//         let meta_type = MetaType::new::<T>();
//         println!("{msg} info: {:#?}\n", meta_type.type_info());
//         let mut registry = Registry::new();
//         registry.register_type(&meta_type);

//     }

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
