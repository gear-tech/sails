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

//! Traits and structs for a simple service comprised of command and query dispatchers.

#![no_std]

use scale_info::MetaType;

pub trait ServiceMeta {
    fn commands() -> MetaType;
    fn queries() -> MetaType;
    fn events() -> MetaType;
}

pub struct AnyServiceMeta {
    commands: MetaType,
    queries: MetaType,
    events: MetaType,
}

impl AnyServiceMeta {
    pub fn new<S: ServiceMeta>() -> Self {
        Self {
            commands: S::commands(),
            queries: S::queries(),
            events: S::events(),
        }
    }

    pub fn commands(&self) -> &MetaType {
        &self.commands
    }

    pub fn queries(&self) -> &MetaType {
        &self.queries
    }

    pub fn events(&self) -> &MetaType {
        &self.events
    }
}

pub trait ProgramMeta {
    fn constructors() -> MetaType;
    fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)>;
}
