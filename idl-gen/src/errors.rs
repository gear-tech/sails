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

//! Errors returned by IDL generator.

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("funcion meta is invalid: {0}")]
    FuncMetaIsInvalid(String),
    #[error("event meta is invalid: {0}")]
    EventMetaIsInvalid(String),
    #[error("event meta is ambiguous: {0}")]
    EventMetaIsAmbiguous(String),
    #[error("type id `{0}` is not found in the type registry")]
    TypeIdIsUnknown(u32),
    #[error("type `{0}` is not supported")]
    TypeIsUnsupported(String),
    #[error(transparent)]
    TemplateIsBroken(#[from] Box<handlebars::TemplateError>),
    #[error(transparent)]
    RenderingFailed(#[from] Box<handlebars::RenderError>),
}
