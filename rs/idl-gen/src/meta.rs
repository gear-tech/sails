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

use crate::{
    errors::{Error, Result},
    type_names,
};
use gprimitives::*;
use sails_idl_meta::*;
use scale_info::{
    form::PortableForm, Field, MetaType, PortableRegistry, PortableType, Registry, TypeDef, Variant,
};

struct CtorFuncMeta(String, u32, Vec<Field<PortableForm>>, Vec<String>);

struct ServiceFuncMeta(String, u32, Vec<Field<PortableForm>>, u32, Vec<String>);

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
                && !self.ctors_type_id.is_some_and(|id| id == ty.id)
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
                let func_params_type =
                    registry
                        .resolve(func_descr.fields[0].ty.id)
                        .unwrap_or_else(|| {
                            panic!(
                            "func params type id {} not found while it was registered previously",
                            func_descr.fields[0].ty.id
                        )
                        });
                if let TypeDef::Composite(func_params_type) = &func_params_type.type_def {
                    let func_meta = ServiceFuncMeta(
                        func_descr.name.to_string(),
                        func_descr.fields[0].ty.id,
                        func_params_type.fields.to_vec(),
                        func_descr.fields[1].ty.id,
                        func_descr.docs.iter().map(|s| s.to_string()).collect(),
                    );
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
                    "events type id {} not found while it was registered previously",
                    events_type_id
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
                    "events type id {} references a type that is not a variant",
                    events_type_id
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
        panic!(
            "func type id {} not found while it was registered previously",
            func_type_id
        )
    });
    if let TypeDef::Variant(variant) = &funcs.type_def {
        Ok(variant.variants.iter())
    } else {
        Err(Error::FuncMetaIsInvalid(format!(
            "func type id {} references a type that is not a variant",
            func_type_id
        )))
    }
}
