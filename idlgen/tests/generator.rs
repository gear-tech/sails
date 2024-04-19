use sails_idl_meta::{AnyServiceMeta, ProgramMeta, ServiceMeta};
use sails_idlgen::{program, service};
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

#[allow(dead_code)]
#[derive(TypeInfo)]
enum EventsMeta {
    ThisDone(u32),
    ThatDone { p1: String },
}

struct TestServiceMeta;

impl ServiceMeta for TestServiceMeta {
    fn commands() -> MetaType {
        scale_info::meta_type::<CommandsMeta>()
    }

    fn queries() -> MetaType {
        scale_info::meta_type::<QueriesMeta>()
    }

    fn events() -> MetaType {
        scale_info::meta_type::<EventsMeta>()
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

struct TestProgramWithMultipleServicesMeta;

impl ProgramMeta for TestProgramWithMultipleServicesMeta {
    fn constructors() -> MetaType {
        scale_info::meta_type::<EmptyCtorsMeta>()
    }

    fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        vec![
            ("", AnyServiceMeta::new::<TestServiceMeta>()),
            ("SomeService", AnyServiceMeta::new::<TestServiceMeta>()),
        ]
        .into_iter()
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
    assert_eq!(generated_idl_program.services().len(), 1);
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
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
    assert_eq!(generated_idl_program.services().len(), 1);
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    assert_eq!(generated_idl_program.types().len(), 8);
}

#[test]
fn generate_program_idl_works_with_multiple_services() {
    let mut idl = Vec::new();
    program::generate_idl::<TestProgramWithMultipleServicesMeta>(&mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();
    let generated_idl_program = sails_idlparser::ast::parse_idl(&generated_idl);

    insta::assert_snapshot!(generated_idl);
    let generated_idl_program = generated_idl_program.unwrap();
    assert!(generated_idl_program.ctor().is_none());
    assert_eq!(generated_idl_program.services().len(), 2);
    assert_eq!(generated_idl_program.services()[0].name(), "");
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    assert_eq!(generated_idl_program.services()[1].name(), "SomeService");
    assert_eq!(generated_idl_program.services()[1].funcs().len(), 4);
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
    assert_eq!(generated_idl_program.services().len(), 1);
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    assert_eq!(generated_idl_program.types().len(), 8);
}
