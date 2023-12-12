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
use sails_service::{CommandProcessorMeta, QueryProcessorMeta, ServiceMeta};
use scale_info::PortableType;
use serde::Serialize;
use service_types::{ServiceTypes, ServiceTypesEx};
use std::io;

mod errors;
mod service_types;
mod type_names;

const IDL_EX_TEMPLATE: &str = include_str!("../hbs/idl_ex.hbs");
const IDL_TEMPLATE: &str = include_str!("../hbs/idl.hbs");
const COMPOSITE_TEMPLATE: &str = include_str!("../hbs/composite.hbs");
const VARIANT_TEMPLATE: &str = include_str!("../hbs/variant.hbs");

pub fn generate_serivce_idl<C, Q>(
    _service_name: Option<&str>,
    idl_writer: impl io::Write,
) -> errors::Result<()>
where
    C: CommandProcessorMeta,
    Q: QueryProcessorMeta,
{
    let service_info = ServiceTypes::<C, Q>::new();

    let service_all_type_names = type_names::resolve_type_names(service_info.all_types_registry())?;

    let service_idl_data = ServiceIdlData {
        complex_types: service_info.complex_types().collect(),
        commands: service_info.command_types().0,
        command_responses: service_info.command_types().1,
        queries: service_info.query_types().0,
        query_responses: service_info.query_types().1,
        type_names: service_all_type_names.values().collect(),
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

pub fn generate_serivce_idl_ex<S: ServiceMeta>(idl_writer: impl io::Write) -> errors::Result<()> {
    let service_info = ServiceTypesEx::<S>::new();

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
        .register_template_string("idl", IDL_EX_TEMPLATE)
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
    use parity_scale_codec::{Decode, Encode};
    use sails_service::BoxedFuture;
    use scale_info::TypeInfo;
    use std::result::Result as StdResult;

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
    pub struct GenericStruct<T> {
        pub p1: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
    pub enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
    pub struct DoThatParam {
        pub p1: u32,
        pub p2: String,
        pub p3: ManyVariants,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
    pub struct ThatParam {
        pub p1: ManyVariants,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
    pub struct TupleStruct(bool);

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
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
    #[derive(TypeInfo, Decode)]
    enum Commands {
        DoThis(
            u32,
            String,
            (Option<String>, u8),
            TupleStruct,
            GenericStruct<u32>,
            GenericStruct<String>,
        ),
        DoThat(DoThatParam),
        Fail(String),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Encode)]
    enum CommandResponses {
        DoThis(StdResult<(String, u32), String>),
        DoThat(StdResult<(String, u32), (String,)>),
        Fail(StdResult<(), String>),
    }

    struct TestCommandProcessorMeta;

    impl CommandProcessorMeta for TestCommandProcessorMeta {
        type Request = Commands;
        type Response = CommandResponses;
        type ProcessFn = fn(Self::Request) -> BoxedFuture<(Self::Response, bool)>;
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
    enum Queries {
        This(
            u32,
            String,
            (Option<String>, u8),
            TupleStruct,
            GenericEnum<bool, u32>,
        ),
        That(ThatParam),
        Fail(String),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Encode)]
    enum QueryResponses {
        This(StdResult<(String, u32), String>),
        That(StdResult<(String, u32), (String,)>),
        Fail(StdResult<(), String>),
    }

    struct TestQueryProcessorMeta;

    impl QueryProcessorMeta for TestQueryProcessorMeta {
        type Request = Queries;
        type Response = QueryResponses;
        type ProcessFn = fn(Self::Request) -> (Self::Response, bool);
    }

    #[test]
    fn idl_generation_works_for_commands() {
        let mut idl = Vec::new();
        generate_serivce_idl::<TestCommandProcessorMeta, ()>(None, &mut idl).unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();

        const EXPECTED_IDL: &str = r"type SailsIdlgenTestsTupleStruct = struct {
  bool,
};

type SailsIdlgenTestsGenericStruct<u32> = struct {
  p1: u32,
};

type SailsIdlgenTestsGenericStruct<str> = struct {
  p1: str,
};

type SailsIdlgenTestsDoThatParam = struct {
  p1: u32,
  p2: str,
  p3: SailsIdlgenTestsManyVariants,
};

type SailsIdlgenTestsManyVariants = variant {
  One,
  Two: u32,
  Three: opt vec u32,
  Four: struct { a: u32, b: opt u16 },
  Five: struct { str, vec u8 },
  Six: struct { u32 },
  Seven: SailsIdlgenTestsGenericEnum<u32, str>,
};

type SailsIdlgenTestsGenericEnum<u32, str> = variant {
  Variant1: u32,
  Variant2: str,
};

service {
  async DoThis : (u32, str, struct { opt str, u8 }, SailsIdlgenTestsTupleStruct, SailsIdlgenTestsGenericStruct<u32>, SailsIdlgenTestsGenericStruct<str>) -> result (struct { str, u32 }, str);
  async DoThat : (SailsIdlgenTestsDoThatParam) -> result (struct { str, u32 }, struct { str });
  async Fail : (str) -> result (null, str);
}
";
        assert_eq!(generated_idl, EXPECTED_IDL);
    }

    #[test]
    fn idl_generation_works_for_queries() {
        let mut idl = Vec::new();
        generate_serivce_idl::<(), TestQueryProcessorMeta>(None, &mut idl).unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();

        const EXPECTED_IDL: &str = r"type SailsIdlgenTestsTupleStruct = struct {
  bool,
};

type SailsIdlgenTestsGenericEnum<bool, u32> = variant {
  Variant1: bool,
  Variant2: u32,
};

type SailsIdlgenTestsThatParam = struct {
  p1: SailsIdlgenTestsManyVariants,
};

type SailsIdlgenTestsManyVariants = variant {
  One,
  Two: u32,
  Three: opt vec u32,
  Four: struct { a: u32, b: opt u16 },
  Five: struct { str, vec u8 },
  Six: struct { u32 },
  Seven: SailsIdlgenTestsGenericEnum<u32, str>,
};

type SailsIdlgenTestsGenericEnum<u32, str> = variant {
  Variant1: u32,
  Variant2: str,
};

service {
  This : (u32, str, struct { opt str, u8 }, SailsIdlgenTestsTupleStruct, SailsIdlgenTestsGenericEnum<bool, u32>) -> result (struct { str, u32 }, str) query;
  That : (SailsIdlgenTestsThatParam) -> result (struct { str, u32 }, struct { str }) query;
  Fail : (str) -> result (null, str) query;
}
";
        assert_eq!(generated_idl, EXPECTED_IDL);
    }

    #[test]
    fn idl_generation_works_for_commands_and_queries() {
        let mut idl = Vec::new();
        generate_serivce_idl::<TestCommandProcessorMeta, TestQueryProcessorMeta>(None, &mut idl)
            .unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();

        const EXPECTED_IDL: &str = r"type SailsIdlgenTestsTupleStruct = struct {
  bool,
};

type SailsIdlgenTestsGenericStruct<u32> = struct {
  p1: u32,
};

type SailsIdlgenTestsGenericStruct<str> = struct {
  p1: str,
};

type SailsIdlgenTestsDoThatParam = struct {
  p1: u32,
  p2: str,
  p3: SailsIdlgenTestsManyVariants,
};

type SailsIdlgenTestsManyVariants = variant {
  One,
  Two: u32,
  Three: opt vec u32,
  Four: struct { a: u32, b: opt u16 },
  Five: struct { str, vec u8 },
  Six: struct { u32 },
  Seven: SailsIdlgenTestsGenericEnum<u32, str>,
};

type SailsIdlgenTestsGenericEnum<u32, str> = variant {
  Variant1: u32,
  Variant2: str,
};

type SailsIdlgenTestsGenericEnum<bool, u32> = variant {
  Variant1: bool,
  Variant2: u32,
};

type SailsIdlgenTestsThatParam = struct {
  p1: SailsIdlgenTestsManyVariants,
};

service {
  async DoThis : (u32, str, struct { opt str, u8 }, SailsIdlgenTestsTupleStruct, SailsIdlgenTestsGenericStruct<u32>, SailsIdlgenTestsGenericStruct<str>) -> result (struct { str, u32 }, str);
  async DoThat : (SailsIdlgenTestsDoThatParam) -> result (struct { str, u32 }, struct { str });
  async Fail : (str) -> result (null, str);
  This : (u32, str, struct { opt str, u8 }, SailsIdlgenTestsTupleStruct, SailsIdlgenTestsGenericEnum<bool, u32>) -> result (struct { str, u32 }, str) query;
  That : (SailsIdlgenTestsThatParam) -> result (struct { str, u32 }, struct { str }) query;
  Fail : (str) -> result (null, str) query;
}
";
        assert_eq!(generated_idl, EXPECTED_IDL);
    }
}
