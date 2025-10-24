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
use sails_interface_id::{
    canonical::CanonicalDocument, runtime::build_canonical_document_from_meta,
};
use scale_info::{
    Field, MetaType, PortableRegistry, PortableType, Registry, TypeDef, Variant, form::PortableForm,
};
use std::collections::BTreeMap;

struct CtorFuncMeta(String, u32, Vec<Field<PortableForm>>, Vec<String>);

struct ServiceFuncMeta {
    name: String,
    params_type_id: u32,
    params_fields: Vec<Field<PortableForm>>,
    result_type_id: u32,
    entry_id: u16,
    docs: Vec<String>,
}

struct EventMeta {
    variant: Variant<PortableForm>,
    entry_id: u16,
}

pub(crate) struct ExtendedInterfaceMeta {
    pub name: String,
    pub interface_id: u64,
}

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
                let command_entry_ids = sm.command_entry_ids().to_vec();
                let query_entry_ids = sm.query_entry_ids().to_vec();
                let event_entry_ids = sm.event_entry_ids().to_vec();

                let service_name = sm.interface_path();
                let canonical_doc = build_canonical_document_from_meta(&sm).map_err(|err| {
                    Error::FuncMetaIsInvalid(format!(
                        "failed to build canonical document for `{service_name}`: {err}"
                    ))
                })?;
                let service_entry = canonical_doc.services.get(service_name).ok_or_else(|| {
                    Error::FuncMetaIsInvalid(format!(
                        "canonical document for `{service_name}` is missing service entry"
                    ))
                })?;
                let mut single_services = BTreeMap::new();
                single_services.insert(service_name.to_owned(), service_entry.clone());
                let single_doc = CanonicalDocument {
                    canon_schema: canonical_doc.canon_schema.clone(),
                    canon_version: canonical_doc.canon_version.clone(),
                    hash: canonical_doc.hash.clone(),
                    services: single_services,
                    types: canonical_doc.types.clone(),
                };
                let canonical_bytes = single_doc.to_bytes().map_err(|err| {
                    Error::FuncMetaIsInvalid(format!(
                        "failed to serialize canonical document for `{service_name}`: {err}"
                    ))
                })?;
                let extends = service_entry
                    .extends
                    .iter()
                    .map(|ext| ExtendedInterfaceMeta {
                        name: ext.name.clone(),
                        interface_id: ext.interface_id,
                    })
                    .collect::<Vec<_>>();

                Ok((
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
                    command_entry_ids,
                    query_entry_ids,
                    event_entry_ids,
                    canonical_bytes,
                    extends,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let registry = PortableRegistry::from(registry);
        let ctors = Self::ctor_funcs(&registry, ctors_type_id)?;
        let services = services_data
            .into_iter()
            .map(
                |(
                    sname,
                    ct_ids,
                    qt_ids,
                    et_ids,
                    command_entry_ids,
                    query_entry_ids,
                    event_entry_ids,
                    canonical,
                    extends,
                )| {
                    ExpandedServiceMeta::new(
                        &registry,
                        sname,
                        ct_ids,
                        qt_ids,
                        et_ids,
                        command_entry_ids,
                        query_entry_ids,
                        event_entry_ids,
                        canonical,
                        extends,
                    )
                },
            )
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
        self.services.iter().flat_map(|s| {
            s.commands
                .iter()
                .chain(&s.overriden_commands)
                .map(|v| v.params_type_id)
        })
    }

    fn query_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.services.iter().flat_map(|s| {
            s.queries
                .iter()
                .chain(&s.overriden_queries)
                .map(|v| v.params_type_id)
        })
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
    events: Vec<EventMeta>,
    canonical_bytes: Vec<u8>,
    extends: Vec<ExtendedInterfaceMeta>,
}

impl ExpandedServiceMeta {
    fn new(
        registry: &PortableRegistry,
        name: &'static str,
        commands_type_ids: Vec<u32>,
        queries_type_ids: Vec<u32>,
        events_type_ids: Vec<u32>,
        command_entry_ids: Vec<u16>,
        query_entry_ids: Vec<u16>,
        event_entry_ids: Vec<u16>,
        canonical_bytes: Vec<u8>,
        extends: Vec<ExtendedInterfaceMeta>,
    ) -> Result<Self> {
        let (commands, overriden_commands) = Self::service_funcs(
            registry,
            commands_type_ids.iter().copied(),
            command_entry_ids.into_iter(),
        )?;
        let (queries, overriden_queries) = Self::service_funcs(
            registry,
            queries_type_ids.iter().copied(),
            query_entry_ids.into_iter(),
        )?;
        let events = Self::event_variants(
            registry,
            events_type_ids.iter().copied(),
            event_entry_ids.into_iter(),
        )?;
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
            canonical_bytes,
            extends,
        })
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn commands(
        &self,
    ) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>, u32, u16, &Vec<String>)> {
        self.commands.iter().map(|c| {
            (
                c.name.as_str(),
                &c.params_fields,
                c.result_type_id,
                c.entry_id,
                &c.docs,
            )
        })
    }

    pub fn queries(
        &self,
    ) -> impl Iterator<Item = (&str, &Vec<Field<PortableForm>>, u32, u16, &Vec<String>)> {
        self.queries.iter().map(|c| {
            (
                c.name.as_str(),
                &c.params_fields,
                c.result_type_id,
                c.entry_id,
                &c.docs,
            )
        })
    }

    pub fn events(&self) -> impl Iterator<Item = (&Variant<PortableForm>, u16)> {
        self.events
            .iter()
            .map(|event| (&event.variant, event.entry_id))
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn extends(&self) -> &[ExtendedInterfaceMeta] {
        &self.extends
    }

    fn service_funcs(
        registry: &PortableRegistry,
        func_type_ids: impl Iterator<Item = u32>,
        entry_ids: impl Iterator<Item = u16>,
    ) -> Result<(Vec<ServiceFuncMeta>, Vec<ServiceFuncMeta>)> {
        let mut entry_ids = entry_ids.peekable();
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
                    let entry_id = entry_ids.next().ok_or_else(|| {
                        Error::FuncMetaIsInvalid(format!(
                            "missing entry_id metadata for function `{}`",
                            func_descr.name
                        ))
                    })?;
                    let func_meta = ServiceFuncMeta {
                        name: func_descr.name.to_string(),
                        params_type_id: func_descr.fields[0].ty.id,
                        params_fields: func_params_type.fields.to_vec(),
                        result_type_id: func_descr.fields[1].ty.id,
                        entry_id,
                        docs: func_descr.docs.iter().map(|s| s.to_string()).collect(),
                    };
                    if !funcs_meta
                        .iter()
                        .any(|fm: &ServiceFuncMeta| fm.name == func_meta.name)
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
        if entry_ids.next().is_some() {
            return Err(Error::FuncMetaIsInvalid(
                "entry metadata contains extra entries".into(),
            ));
        }
        Ok((funcs_meta, overriden_funcs_meta))
    }

    fn event_variants(
        registry: &PortableRegistry,
        events_type_ids: impl Iterator<Item = u32>,
        entry_ids: impl Iterator<Item = u16>,
    ) -> Result<Vec<EventMeta>> {
        let mut entry_ids = entry_ids.peekable();
        let mut events_variants = Vec::new();
        for events_type_id in events_type_ids {
            let events = registry.resolve(events_type_id).unwrap_or_else(|| {
                panic!(
                    "events type id {events_type_id} not found while it was registered previously"
                )
            });
            if let TypeDef::Variant(variant) = &events.type_def {
                for event_variant in &variant.variants {
                    let entry_id = entry_ids.next().ok_or_else(|| {
                        Error::EventMetaIsInvalid(format!(
                            "missing event entry metadata for `{}`",
                            event_variant.name
                        ))
                    })?;
                    if events_variants
                        .iter()
                        .any(|ev: &EventMeta| ev.variant.name == event_variant.name)
                    {
                        return Err(Error::EventMetaIsAmbiguous(format!(
                            "events type id {} contains ambiguous event variant `{}`",
                            events_type_id, event_variant.name
                        )));
                    }
                    events_variants.push(EventMeta {
                        variant: event_variant.clone(),
                        entry_id,
                    });
                }
            } else {
                return Err(Error::EventMetaIsInvalid(format!(
                    "events type id {events_type_id} references a type that is not a variant"
                )));
            }
        }
        if entry_ids.next().is_some() {
            return Err(Error::EventMetaIsInvalid(
                "event entry metadata contains extra entries".into(),
            ));
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
