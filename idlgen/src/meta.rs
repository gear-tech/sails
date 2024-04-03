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
use sails_rtl::{ActorId, CodeId, MessageId};
use scale_info::{
    form::PortableForm, Field, MetaType, PortableRegistry, PortableType, Registry, TypeDef, Variant,
};

struct CtorFuncMeta(String, u32, Vec<Field<PortableForm>>);

struct ServiceFuncMeta(String, u32, Vec<Field<PortableForm>>, u32);

pub(crate) struct ExpandedProgramMeta {
    registry: PortableRegistry,
    ctors_type_id: Option<u32>,
    ctors: Vec<CtorFuncMeta>,
    commands_type_id: u32,
    commands: Vec<ServiceFuncMeta>,
    queries_type_id: u32,
    queries: Vec<ServiceFuncMeta>,
    events_type_id: u32,
    events: Vec<Variant<PortableForm>>,
    builtin_type_ids: Vec<u32>,
}

impl ExpandedProgramMeta {
    pub fn new(
        ctors: Option<&MetaType>,
        commands: &MetaType,
        queries: &MetaType,
        events: &MetaType,
    ) -> Result<Self> {
        let mut registry = Registry::new();
        let ctors_type_id = ctors.map(|ctors| registry.register_type(ctors).id);
        let commands_type_id = registry.register_type(commands).id;
        let queries_type_id = registry.register_type(queries).id;
        let events_type_id = registry.register_type(events).id;
        let builtin_type_ids = registry
            .register_types([
                MetaType::new::<ActorId>(),
                MetaType::new::<CodeId>(),
                MetaType::new::<MessageId>(),
            ])
            .iter()
            .map(|t| t.id)
            .collect::<Vec<_>>();
        let registry = PortableRegistry::from(registry);
        let ctors = Self::ctor_funcs(&registry, ctors_type_id)?;
        let commands = Self::service_funcs(&registry, commands_type_id)?;
        let queries = Self::service_funcs(&registry, queries_type_id)?;
        let events = Self::event_variants(&registry, events_type_id)?;
        Ok(Self {
            registry,
            ctors_type_id,
            ctors,
            commands_type_id,
            commands,
            queries_type_id,
            queries,
            events_type_id,
            events,
            builtin_type_ids,
        })
    }

    /// Returns complex types introduced by program only
    pub fn types(&self) -> impl Iterator<Item = &PortableType> {
        self.registry.types.iter().filter(|ty| {
            !ty.ty.path.namespace().is_empty()
                && !self.ctors_type_id.is_some_and(|id| id == ty.id)
                && ty.id != self.commands_type_id
                && ty.id != self.queries_type_id
                && ty.id != self.events_type_id
                && !self.ctor_params_type_ids().any(|id| id == ty.id)
                && !self.command_params_type_ids().any(|id| id == ty.id)
                && !self.query_params_type_ids().any(|id| id == ty.id)
                && !self.builtin_type_ids.contains(&ty.id)
        })
    }

    pub fn ctors(&self) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>)> {
        self.ctors.iter().map(|c| (c.0.as_str(), &c.2))
    }

    pub fn commands(&self) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>, u32)> {
        self.commands.iter().map(|c| (c.0.as_str(), &c.2, c.3))
    }

    pub fn queries(&self) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>, u32)> {
        self.queries.iter().map(|c| (c.0.as_str(), &c.2, c.3))
    }

    pub fn events(&self) -> impl Iterator<Item = &Variant<PortableForm>> {
        self.events.iter()
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
        Self::any_funcs(registry, func_type_id)?
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

    fn service_funcs(
        registry: &PortableRegistry,
        func_type_id: u32,
    ) -> Result<Vec<ServiceFuncMeta>> {
        Self::any_funcs(registry, func_type_id)?
            .map(|f| {
                if f.fields.len() != 2 {
                    Err(Error::FuncMetaIsInvalid(format!(
                        "func `{}` has invalid number of fields",
                        f.name
                    )))
                } else {
                    let params_type = registry.resolve(f.fields[0].ty.id).unwrap_or_else(|| {
                        panic!(
                            "func params type id {} not found while it was registered previously",
                            f.fields[0].ty.id
                        )
                    });
                    if let TypeDef::Composite(params_type) = &params_type.type_def {
                        Ok(ServiceFuncMeta(
                            f.name.to_string(),
                            f.fields[0].ty.id,
                            params_type.fields.to_vec(),
                            f.fields[1].ty.id,
                        ))
                    } else {
                        Err(Error::FuncMetaIsInvalid(format!(
                            "func `{}` params type is not a composite",
                            f.name
                        )))
                    }
                }
            })
            .collect()
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

    fn ctor_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.ctors.iter().map(|v| v.1)
    }

    fn command_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.commands.iter().map(|v| v.1)
    }

    fn query_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.queries.iter().map(|v| v.1)
    }

    fn event_variants(
        registry: &PortableRegistry,
        events_type_id: u32,
    ) -> Result<Vec<Variant<PortableForm>>> {
        let events = registry.resolve(events_type_id).unwrap_or_else(|| {
            panic!(
                "events type id {} not found while it was registered previously",
                events_type_id
            )
        });
        if let TypeDef::Variant(variant) = &events.type_def {
            Ok(variant.variants.to_vec())
        } else {
            Err(Error::EventMetaIsInvalid(format!(
                "events type id {} references a type that is not a variant",
                events_type_id
            )))
        }
    }
}
