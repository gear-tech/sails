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

//! Functionality for generating IDL files describing some service based on its Rust code.

use errors::Result;
use handlebars::{handlebars_helper, Handlebars};
use meta::ExpandedProgramMeta;
use scale_info::{form::PortableForm, Field, PortableType, Variant};
use serde::Serialize;
use std::io::Write;

mod errors;
mod meta;
mod type_names;

const IDL_TEMPLATE: &str = include_str!("../hbs/idl.hbs");
const COMPOSITE_TEMPLATE: &str = include_str!("../hbs/composite.hbs");
const VARIANT_TEMPLATE: &str = include_str!("../hbs/variant.hbs");

pub mod program {
    use super::*;
    use sails_rtl::meta::ProgramMeta;

    pub fn generate_idl<P: ProgramMeta>(idl_writer: impl Write) -> Result<()> {
        render_idl(
            &ExpandedProgramMeta::new(Some(P::constructors()), P::services())?,
            idl_writer,
        )
    }
}

pub mod service {
    use super::*;
    use sails_rtl::meta::{AnyServiceMeta, ServiceMeta};

    pub fn generate_idl<S: ServiceMeta>(idl_writer: impl Write) -> Result<()> {
        render_idl(
            &ExpandedProgramMeta::new(None, vec![("", AnyServiceMeta::new::<S>())].into_iter())?,
            idl_writer,
        )
    }
}

fn render_idl(program_meta: &ExpandedProgramMeta, idl_writer: impl Write) -> Result<()> {
    let program_idl_data = ProgramIdlData {
        type_names: program_meta.type_names()?.collect(),
        types: program_meta.types().collect(),
        ctors: program_meta.ctors().collect(),
        services: program_meta
            .services()
            .map(|s| ServiceIdlData {
                name: s.name(),
                commands: s.commands().collect(),
                queries: s.queries().collect(),
                events: s.events().collect(),
            })
            .collect(),
    };

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("idl", IDL_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_template_string("composite", COMPOSITE_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_template_string("variant", VARIANT_TEMPLATE)
        .map_err(Box::new)?;
    handlebars.register_helper("deref", Box::new(deref));

    handlebars
        .render_to_write("idl", &program_idl_data, idl_writer)
        .map_err(Box::new)?;

    Ok(())
}

#[derive(Serialize)]
struct ProgramIdlData<'a> {
    type_names: Vec<String>,
    types: Vec<&'a PortableType>,
    ctors: Vec<(&'a str, &'a Vec<Field<PortableForm>>)>,
    services: Vec<ServiceIdlData<'a>>,
}

#[derive(Serialize)]
struct ServiceIdlData<'a> {
    name: &'a str,
    commands: Vec<(&'a str, &'a Vec<Field<PortableForm>>, u32)>,
    queries: Vec<(&'a str, &'a Vec<Field<PortableForm>>, u32)>,
    events: Vec<&'a Variant<PortableForm>>,
}

handlebars_helper!(deref: |v: String| { v });
