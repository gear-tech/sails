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
use sails_service::{CommandProcessorMeta, QueryProcessorMeta};
use scale_info::PortableType;
use service_types::ServiceTypes;
use std::io;

mod errors;
mod service_types;
mod type_names;

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
    }

    #[allow(dead_code)]
    #[derive(TypeInfo, Decode)]
    enum Commands {
        DoThis(u32, String, (Option<String>, u8), TupleStruct),
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
        This(u32, String, (Option<String>, u8), TupleStruct),
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

        const EXPECTED_IDL: &str = r"type SailsIdlgenTestsTupleStruct = record {
  bool;
};

type SailsIdlgenTestsDoThatParam = record {
  p1: nat32;
  p2: text;
  p3: SailsIdlgenTestsManyVariants;
};

type SailsIdlgenTestsManyVariants = variant {
  One;
  Two: nat32;
  Three: opt vec nat32;
  Four: record { a: nat32; b: opt nat16 };
  Five: record { text; vec nat8 };
  Six: record { nat32 };
};

service {
  async DoThis : (nat32, text, record { opt text; nat8 }, SailsIdlgenTestsTupleStruct) -> (record { text; nat32 }, text);
  async DoThat : (SailsIdlgenTestsDoThatParam) -> (record { text; nat32 }, record { text });
  async Fail : (text) -> (null, text);
}
";
        assert_eq!(generated_idl, EXPECTED_IDL);
    }

    #[test]
    fn idl_generation_works_for_queries() {
        let mut idl = Vec::new();
        generate_serivce_idl::<(), TestQueryProcessorMeta>(None, &mut idl).unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();

        const EXPECTED_IDL: &str = r"type SailsIdlgenTestsTupleStruct = record {
  bool;
};

type SailsIdlgenTestsThatParam = record {
  p1: SailsIdlgenTestsManyVariants;
};

type SailsIdlgenTestsManyVariants = variant {
  One;
  Two: nat32;
  Three: opt vec nat32;
  Four: record { a: nat32; b: opt nat16 };
  Five: record { text; vec nat8 };
  Six: record { nat32 };
};

service {
  This : (nat32, text, record { opt text; nat8 }, SailsIdlgenTestsTupleStruct) -> (record { text; nat32 }, text) query;
  That : (SailsIdlgenTestsThatParam) -> (record { text; nat32 }, record { text }) query;
  Fail : (text) -> (null, text) query;
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

        const EXPECTED_IDL: &str = r"type SailsIdlgenTestsTupleStruct = record {
  bool;
};

type SailsIdlgenTestsDoThatParam = record {
  p1: nat32;
  p2: text;
  p3: SailsIdlgenTestsManyVariants;
};

type SailsIdlgenTestsManyVariants = variant {
  One;
  Two: nat32;
  Three: opt vec nat32;
  Four: record { a: nat32; b: opt nat16 };
  Five: record { text; vec nat8 };
  Six: record { nat32 };
};

type SailsIdlgenTestsThatParam = record {
  p1: SailsIdlgenTestsManyVariants;
};

service {
  async DoThis : (nat32, text, record { opt text; nat8 }, SailsIdlgenTestsTupleStruct) -> (record { text; nat32 }, text);
  async DoThat : (SailsIdlgenTestsDoThatParam) -> (record { text; nat32 }, record { text });
  async Fail : (text) -> (null, text);
  This : (nat32, text, record { opt text; nat8 }, SailsIdlgenTestsTupleStruct) -> (record { text; nat32 }, text) query;
  That : (SailsIdlgenTestsThatParam) -> (record { text; nat32 }, record { text }) query;
  Fail : (text) -> (null, text) query;
}
";
        assert_eq!(generated_idl, EXPECTED_IDL);
    }
}
