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

//! Procedural macros for the `gprogram-framework`.

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use sails_macros_core::{command_handlers_core, gservice_core, query_handlers_core};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn command_handlers(_attrs: TokenStream, mod_tokens: TokenStream) -> TokenStream {
    command_handlers_core(mod_tokens.into()).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn query_handlers(_attrs: TokenStream, mod_tokens: TokenStream) -> TokenStream {
    query_handlers_core(mod_tokens.into()).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn gservice(_attrs: TokenStream, impl_tokens: TokenStream) -> TokenStream {
    gservice_core(impl_tokens.into()).into()
}
