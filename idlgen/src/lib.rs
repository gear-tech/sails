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

use handlebars::{handlebars_helper, Handlebars};
use sails_service_meta::ServiceMeta;
use scale_info::PortableType;
use serde::Serialize;
use service_types::ServiceTypes;
use std::io;

mod errors;
mod service_types;
mod type_names;

const IDL_TEMPLATE: &str = include_str!("../hbs/idl.hbs");
const COMPOSITE_TEMPLATE: &str = include_str!("../hbs/composite.hbs");
const VARIANT_TEMPLATE: &str = include_str!("../hbs/variant.hbs");

/// Generates IDL for the given service meta and writes it to the given writer.
pub fn generate_serivce_idl<S: ServiceMeta>(idl_writer: impl io::Write) -> errors::Result<()> {
    let service_info = ServiceTypes::<S>::new();

    let service_all_type_names = type_names::resolve_type_names(service_info.all_types_registry())?;

    let service_idl_data = ServiceIdlDataEx {
        type_names: service_all_type_names.values().collect(),
        all_types: service_info.all_types_registry().types.iter().collect(),
        complex_types: service_info.complex_types().collect(),
        commands: service_info.commands_type(),
        queries: service_info.queries_type(),
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
        .render_to_write("idl", &service_idl_data, idl_writer)
        .map_err(Box::new)?;

    Ok(())
}

#[derive(serde::Serialize)]
struct ServiceIdlData<'a> {
    complex_types: Vec<&'a PortableType>,
    commands: &'a PortableType,
    #[serde(rename = "commandResponses")]
    command_responses: &'a PortableType,
    queries: &'a PortableType,
    #[serde(rename = "queryResponses")]
    query_responses: &'a PortableType,
    type_names: Vec<&'a String>,
}

#[derive(Serialize)]
struct ServiceIdlDataEx<'a> {
    type_names: Vec<&'a String>,
    all_types: Vec<&'a PortableType>,
    complex_types: Vec<&'a PortableType>,
    commands: &'a PortableType,
    queries: &'a PortableType,
}

handlebars_helper!(deref: |v: String| { v });

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::TypeInfo;
    use std::result::Result as StdResult;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    pub struct GenericStruct<T> {
        pub p1: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    pub enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    pub struct DoThatParam {
        pub p1: u32,
        pub p2: String,
        pub p3: ManyVariants,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    pub struct ThatParam {
        pub p1: ManyVariants,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    pub struct TupleStruct(bool);

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    pub enum ManyVariants {
        One,
        Two(u32),
        Three(Option<Vec<u32>>),
        Four { a: u32, b: Option<u16> },
        Five(String, Vec<u8>),
        Six((u32,)),
        Seven(GenericEnum<u32, String>),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct DoThisParams {
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: TupleStruct,
        p5: GenericStruct<u32>,
        p6: GenericStruct<String>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct DoThatParams {
        par1: DoThatParam,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum CommandsMeta {
        DoThis(DoThisParams, String),
        DoThat(DoThatParams, StdResult<(String, u32), (String,)>),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ThisParams {
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: TupleStruct,
        p5: GenericEnum<bool, u32>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct ThatParams {
        pr1: ThatParam,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum QueriesMeta {
        This(ThisParams, StdResult<(String, u32), String>),
        That(ThatParams, String),
    }

    struct TestServiceMeta;

    impl ServiceMeta for TestServiceMeta {
        type Commands = CommandsMeta;
        type Queries = QueriesMeta;
    }

    #[test]
    fn idl_generation_works() {
        let mut idl = Vec::new();
        generate_serivce_idl::<TestServiceMeta>(&mut idl).unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();
        let _generated_idl_program = sails_idlparser::types::parse_idl(&generated_idl);

        const EXPECTED_IDL: &str = r"type SailsIdlgenTestsTupleStruct = struct {
  bool,
};

type SailsIdlgenTestsGenericStructForU32 = struct {
  p1: u32,
};

type SailsIdlgenTestsGenericStructForStr = struct {
  p1: str,
};

type SailsIdlgenTestsDoThatParam = struct {
  p1: u32,
  p2: str,
  p3: SailsIdlgenTestsManyVariants,
};

type SailsIdlgenTestsManyVariants = enum {
  One,
  Two: u32,
  Three: opt vec u32,
  Four: struct { a: u32, b: opt u16 },
  Five: struct { str, vec u8 },
  Six: struct { u32 },
  Seven: SailsIdlgenTestsGenericEnumForU32AndStr,
};

type SailsIdlgenTestsGenericEnumForU32AndStr = enum {
  Variant1: u32,
  Variant2: str,
};

type SailsIdlgenTestsGenericEnumForBoolAndU32 = enum {
  Variant1: bool,
  Variant2: u32,
};

type SailsIdlgenTestsThatParam = struct {
  p1: SailsIdlgenTestsManyVariants,
};

service {
  DoThis : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: SailsIdlgenTestsTupleStruct, p5: SailsIdlgenTestsGenericStructForU32, p6: SailsIdlgenTestsGenericStructForStr) -> str;
  DoThat : (par1: SailsIdlgenTestsDoThatParam) -> result (struct { str, u32 }, struct { str });
  query This : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: SailsIdlgenTestsTupleStruct, p5: SailsIdlgenTestsGenericEnumForBoolAndU32) -> result (struct { str, u32 }, str);
  query That : (pr1: SailsIdlgenTestsThatParam) -> str;
}
";

        assert_eq!(generated_idl, EXPECTED_IDL);
        // assert!(generated_idl_program.is_ok());
        // assert_eq!(generated_idl_program.unwrap().items.len(), 8);
    }
}
