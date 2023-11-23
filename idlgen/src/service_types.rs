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

use sails_service::{CommandProcessorMeta, QueryProcessorMeta};
use scale_info::{MetaType, PortableRegistry, PortableType, Registry};
use std::marker::PhantomData;

pub(crate) struct ServiceTypes<C, Q> {
    type_registry: PortableRegistry,
    commands_type_id: u32,
    command_responses_type_id: u32,
    queries_type_id: u32,
    query_responses_type_id: u32,
    _commands: PhantomData<C>,
    _queries: PhantomData<Q>,
}

impl<C, Q> ServiceTypes<C, Q>
where
    C: CommandProcessorMeta,
    Q: QueryProcessorMeta,
{
    pub fn new() -> Self {
        // TODO: Validate HandlerTypes - both C and Q must be enums with variants having the same names in the same order
        let mut type_registry = Registry::new();
        let commands_type_id = type_registry
            .register_type(&MetaType::new::<C::Request>())
            .id;
        let command_responses_type_id = type_registry
            .register_type(&MetaType::new::<C::Response>())
            .id;
        let queries_type_id = type_registry
            .register_type(&MetaType::new::<Q::Request>())
            .id;
        let query_responses_type_id = type_registry
            .register_type(&MetaType::new::<Q::Response>())
            .id;
        let type_registry = PortableRegistry::from(type_registry);
        Self {
            type_registry,
            commands_type_id,
            command_responses_type_id,
            queries_type_id,
            query_responses_type_id,
            _commands: PhantomData,
            _queries: PhantomData,
        }
    }

    pub fn complex_types(&self) -> impl Iterator<Item = &PortableType> {
        self.type_registry.types.iter().filter(|ty| {
            !ty.ty.path.namespace().is_empty()
                && ty.id != self.commands_type_id
                && ty.id != self.command_responses_type_id
                && ty.id != self.queries_type_id
                && ty.id != self.query_responses_type_id
        })
    }

    pub fn command_types(&self) -> (&PortableType, &PortableType) {
        let commands_type = self.resolve_handler_type(self.commands_type_id);
        let command_responses_type = self.resolve_handler_type(self.command_responses_type_id);
        (commands_type, command_responses_type)
    }

    pub fn query_types(&self) -> (&PortableType, &PortableType) {
        let queries_type = self.resolve_handler_type(self.queries_type_id);
        let query_responses_type = self.resolve_handler_type(self.query_responses_type_id);
        (queries_type, query_responses_type)
    }

    pub fn all_types_registry(&self) -> &PortableRegistry {
        &self.type_registry
    }

    fn resolve_handler_type(&self, handler_type_id: u32) -> &PortableType {
        self.type_registry
            .types
            .iter()
            .find(|ty| ty.id == handler_type_id)
            .unwrap_or_else(|| {
                panic!(
                    "type with id {} not found while it was registered previously",
                    handler_type_id
                )
            })
    }
}
