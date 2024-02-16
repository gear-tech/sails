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

use sails_rtl::{ActorId, CodeId, MessageId};
use sails_service_meta::ServiceMeta;
use scale_info::{MetaType, PortableRegistry, PortableType, Registry};
use std::{marker::PhantomData, vec};

pub(crate) struct ServiceTypes<S> {
    type_registry: PortableRegistry,
    commands_type_id: u32,
    queries_type_id: u32,
    builtin_type_ids: Vec<u32>,
    _service: PhantomData<S>,
}

impl<S: ServiceMeta> ServiceTypes<S> {
    pub fn new() -> Self {
        // TODO: Validate HandlerTypes - both C and Q must be enums with variants having the same names in the same order
        let mut type_registry = Registry::new();
        let commands_type_id = type_registry
            .register_type(&MetaType::new::<S::Commands>())
            .id;
        let queries_type_id = type_registry
            .register_type(&MetaType::new::<S::Queries>())
            .id;
        let builtin_type_ids = type_registry
            .register_types(vec![
                MetaType::new::<ActorId>(),
                MetaType::new::<CodeId>(),
                MetaType::new::<MessageId>(),
            ])
            .iter()
            .map(|t| t.id)
            .collect::<Vec<_>>();
        let type_registry = PortableRegistry::from(type_registry);
        Self {
            type_registry,
            commands_type_id,
            queries_type_id,
            builtin_type_ids,
            _service: PhantomData,
        }
    }

    pub fn complex_types(&self) -> impl Iterator<Item = &PortableType> {
        self.type_registry.types.iter().filter(|ty| {
            !ty.ty.path.namespace().is_empty()
                && ty.id != self.commands_type_id
                && ty.id != self.queries_type_id
                && !self.command_params_type_ids().any(|id| id == ty.id)
                && !self.query_params_type_ids().any(|id| id == ty.id)
                && !self.builtin_type_ids.contains(&ty.id)
        })
    }

    pub fn commands_type(&self) -> &PortableType {
        self.type_registry
            .types
            .iter()
            .find(|ty| ty.id == self.commands_type_id)
            .unwrap_or_else(|| {
                panic!(
                    "type with id {} not found while it was registered previously",
                    self.commands_type_id
                )
            })
    }

    pub fn queries_type(&self) -> &PortableType {
        self.type_registry
            .types
            .iter()
            .find(|ty| ty.id == self.queries_type_id)
            .unwrap_or_else(|| {
                panic!(
                    "type with id {} not found while it was registered previously",
                    self.queries_type_id
                )
            })
    }

    pub fn all_types_registry(&self) -> &PortableRegistry {
        &self.type_registry
    }

    fn command_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        match &self.commands_type().ty.type_def {
            scale_info::TypeDef::Variant(variant) => {
                variant.variants.iter().map(|v| v.fields[0].ty.id)
            }
            _ => panic!("Commands type is not a variant"),
        }
    }

    fn query_params_type_ids(&self) -> impl Iterator<Item = u32> + '_ {
        match &self.queries_type().ty.type_def {
            scale_info::TypeDef::Variant(variant) => {
                variant.variants.iter().map(|v| v.fields[0].ty.id)
            }
            _ => panic!("Queries type is not a variant"),
        }
    }
}
