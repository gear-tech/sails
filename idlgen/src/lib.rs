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

use errors::{Error, Result};
use handlebars::{handlebars_helper, Handlebars};
use meta::ExpandedProgramMeta;
use scale_info::{form::PortableForm, Field, PortableType};
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
    use sails_idl_meta::ProgramMeta;

    pub fn generate_idl<P: ProgramMeta>(idl_writer: impl Write) -> Result<()> {
        let services = P::services().collect::<Vec<_>>();

        if services.is_empty() {
            Err(Error::ServiceIsMissing)?;
        }

        if services.len() > 1 {
            todo!("multiple services are not supported yet");
        }

        let service = &services[0];

        if !service.0.is_empty() {
            todo!("service routes are not supported yet");
        }

        render_idl(
            &ExpandedProgramMeta::new(
                Some(&P::constructors()),
                service.1.commands(),
                service.1.queries(),
            )?,
            idl_writer,
        )
    }
}

pub mod service {
    use super::*;
    use sails_idl_meta::ServiceMeta;

    pub fn generate_idl<S: ServiceMeta>(idl_writer: impl Write) -> Result<()> {
        render_idl(
            &ExpandedProgramMeta::new(None, &S::commands(), &S::queries())?,
            idl_writer,
        )
    }
}

fn render_idl(program_meta: &ExpandedProgramMeta, idl_writer: impl Write) -> Result<()> {
    let program_idl_data = ProgramIdlData {
        type_names: program_meta.type_names()?.collect(),
        types: program_meta.types().collect(),
        ctors: program_meta.ctors().collect(),
        commands: program_meta.commands().collect(),
        queries: program_meta.queries().collect(),
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
    commands: Vec<(&'a str, &'a Vec<Field<PortableForm>>, u32)>,
    queries: Vec<(&'a str, &'a Vec<Field<PortableForm>>, u32)>,
}

handlebars_helper!(deref: |v: String| { v });

#[cfg(test)]
mod tests {
    use super::*;
    use sails_idl_meta::{AnyServiceMeta, ProgramMeta, ServiceMeta};
    use scale_info::{MetaType, TypeInfo};
    use std::{collections::BTreeMap, result::Result as StdResult};

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
        Eight([BTreeMap<u32, String>; 10]),
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
        fn commands() -> MetaType {
            scale_info::meta_type::<CommandsMeta>()
        }

        fn queries() -> MetaType {
            scale_info::meta_type::<QueriesMeta>()
        }
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum EmptyCtorsMeta {}

    struct TestProgramWithEmptyCtorsMeta;

    impl ProgramMeta for TestProgramWithEmptyCtorsMeta {
        fn constructors() -> MetaType {
            scale_info::meta_type::<EmptyCtorsMeta>()
        }

        fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
            vec![("", AnyServiceMeta::new::<TestServiceMeta>())].into_iter()
        }
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct NewParams;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct FromStrParams {
        s: String,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum NonEmptyCtorsMeta {
        New(NewParams),
        FromStr(FromStrParams),
    }

    struct TestProgramWithNonEmptyCtorsMeta;

    impl ProgramMeta for TestProgramWithNonEmptyCtorsMeta {
        fn constructors() -> MetaType {
            scale_info::meta_type::<NonEmptyCtorsMeta>()
        }

        fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
            vec![("", AnyServiceMeta::new::<TestServiceMeta>())].into_iter()
        }
    }

    #[test]
    fn generare_program_idl_works_with_empty_ctors() {
        let mut idl = Vec::new();
        program::generate_idl::<TestProgramWithEmptyCtorsMeta>(&mut idl).unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();
        let generated_idl_program = sails_idlparser::ast::parse_idl(&generated_idl);

        insta::assert_snapshot!(generated_idl);
        let generated_idl_program = generated_idl_program.unwrap();
        assert!(generated_idl_program.ctor().is_none());
        assert_eq!(generated_idl_program.service().funcs().len(), 4);
        assert_eq!(generated_idl_program.types().len(), 8);
    }

    #[test]
    fn generare_program_idl_works_with_non_empty_ctors() {
        let mut idl = Vec::new();
        program::generate_idl::<TestProgramWithNonEmptyCtorsMeta>(&mut idl).unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();
        let generated_idl_program = sails_idlparser::ast::parse_idl(&generated_idl);

        insta::assert_snapshot!(generated_idl);
        let generated_idl_program = generated_idl_program.unwrap();
        assert_eq!(generated_idl_program.ctor().unwrap().funcs().len(), 2);
        assert_eq!(generated_idl_program.service().funcs().len(), 4);
        assert_eq!(generated_idl_program.types().len(), 8);
    }

    #[test]
    fn generate_service_idl_works() {
        let mut idl = Vec::new();
        service::generate_idl::<TestServiceMeta>(&mut idl).unwrap();
        let generated_idl = String::from_utf8(idl).unwrap();
        let generated_idl_program = sails_idlparser::ast::parse_idl(&generated_idl);

        insta::assert_snapshot!(generated_idl);
        let generated_idl_program = generated_idl_program.unwrap();
        assert!(generated_idl_program.ctor().is_none());
        assert_eq!(generated_idl_program.service().funcs().len(), 4);
        assert_eq!(generated_idl_program.types().len(), 8);
    }
}
