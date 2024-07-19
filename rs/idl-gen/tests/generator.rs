use meta_params::*;
use sails_idl_gen::{program, service};
use sails_rs::{
    meta::{AnyServiceMeta, ProgramMeta, ServiceMeta as RtlServiceMeta},
    H256, U256,
};
use scale_info::{MetaType, StaticTypeInfo, TypeInfo};
use std::{collections::BTreeMap, result::Result as StdResult};

#[allow(dead_code)]
mod types {
    use super::*;

    #[derive(TypeInfo)]
    pub struct GenericStruct<T> {
        pub p1: T,
    }

    #[derive(TypeInfo)]
    pub enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
    }

    #[derive(TypeInfo)]
    pub struct TupleStruct(bool);

    #[derive(TypeInfo)]
    pub enum ManyVariants {
        One,
        Two(u32),
        Three(Option<Vec<U256>>),
        Four { a: u32, b: Option<u16> },
        Five(String, Vec<u8>),
        Six((u32,)),
        Seven(GenericEnum<u32, String>),
        Eight([BTreeMap<u32, String>; 10]),
    }

    #[derive(TypeInfo)]
    pub struct DoThatParam {
        pub p1: u32,
        pub p2: String,
        pub p3: ManyVariants,
    }

    #[derive(TypeInfo)]
    pub struct ThatParam {
        pub p1: ManyVariants,
    }
}

#[allow(dead_code)]
mod meta_params {
    use super::{types::*, *};

    #[derive(TypeInfo)]
    pub struct DoThisParams {
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: TupleStruct,
        p5: GenericStruct<H256>,
        p6: GenericStruct<String>,
    }

    #[derive(TypeInfo)]
    pub struct DoThatParams {
        par1: DoThatParam,
    }

    #[derive(TypeInfo)]
    pub struct ThisParams {
        p1: u32,
        p2: String,
        p3: (Option<String>, u8),
        p4: TupleStruct,
        p5: GenericEnum<bool, u32>,
    }

    #[derive(TypeInfo)]
    pub struct ThatParams {
        pr1: ThatParam,
    }

    #[derive(TypeInfo)]
    pub struct SingleParams<T: TypeInfo> {
        pub p1: T,
    }

    #[derive(TypeInfo)]
    pub struct NoParams;
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum CommandsMeta {
    DoThis(DoThisParams, String),
    DoThat(DoThatParams, StdResult<(String, u32), (String,)>),
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum BaseCommandsMeta {
    DoThis(SingleParams<u32>, u32),
    DoThatBase(SingleParams<String>, String),
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum QueriesMeta {
    This(ThisParams, StdResult<(String, u32), String>),
    That(ThatParams, String),
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum BaseQueriesMeta {
    ThisBase(SingleParams<u16>, u16),
    That(SingleParams<String>, String),
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum EventsMeta {
    ThisDone(u32),
    ThatDone { p1: String },
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum BaseEventsMeta {
    ThisDoneBase(u32),
    ThatDoneBase { p1: u16 },
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum AmbiguousBaseEventsMeta {
    ThisDone(u32), // Conflicts with `EventsMeta::ThisDone` even it has the same signatur
    ThatDoneBase { p1: u16 },
}

struct ServiceMeta<C, Q, E> {
    _commands: std::marker::PhantomData<C>,
    _queries: std::marker::PhantomData<Q>,
    _events: std::marker::PhantomData<E>,
}

impl<C: StaticTypeInfo, Q: StaticTypeInfo, E: StaticTypeInfo> RtlServiceMeta
    for ServiceMeta<C, Q, E>
{
    fn commands() -> MetaType {
        scale_info::meta_type::<C>()
    }

    fn queries() -> MetaType {
        scale_info::meta_type::<Q>()
    }

    fn events() -> MetaType {
        scale_info::meta_type::<E>()
    }

    fn base_services() -> impl Iterator<Item = AnyServiceMeta> {
        [].into_iter()
    }
}

struct ServiceMetaWithBase<C, Q, E, B> {
    _commands: std::marker::PhantomData<C>,
    _queries: std::marker::PhantomData<Q>,
    _events: std::marker::PhantomData<E>,
    _base: std::marker::PhantomData<B>,
}

impl<C: StaticTypeInfo, Q: StaticTypeInfo, E: StaticTypeInfo, B: RtlServiceMeta> RtlServiceMeta
    for ServiceMetaWithBase<C, Q, E, B>
{
    fn commands() -> MetaType {
        scale_info::meta_type::<C>()
    }

    fn queries() -> MetaType {
        scale_info::meta_type::<Q>()
    }

    fn events() -> MetaType {
        scale_info::meta_type::<E>()
    }

    fn base_services() -> impl Iterator<Item = AnyServiceMeta> {
        [AnyServiceMeta::new::<B>()].into_iter()
    }
}

type TestServiceMeta = ServiceMeta<CommandsMeta, QueriesMeta, EventsMeta>;

#[allow(dead_code)]
#[derive(TypeInfo)]
enum EmptyCtorsMeta {}

struct TestProgramWithEmptyCtorsMeta;

impl ProgramMeta for TestProgramWithEmptyCtorsMeta {
    fn constructors() -> MetaType {
        scale_info::meta_type::<EmptyCtorsMeta>()
    }

    fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        [("", AnyServiceMeta::new::<TestServiceMeta>())].into_iter()
    }
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum NonEmptyCtorsMeta {
    New(NoParams),
    FromStr(SingleParams<String>),
}

struct TestProgramWithNonEmptyCtorsMeta;

impl ProgramMeta for TestProgramWithNonEmptyCtorsMeta {
    fn constructors() -> MetaType {
        scale_info::meta_type::<NonEmptyCtorsMeta>()
    }

    fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        [("", AnyServiceMeta::new::<TestServiceMeta>())].into_iter()
    }
}

struct TestProgramWithMultipleServicesMeta;

impl ProgramMeta for TestProgramWithMultipleServicesMeta {
    fn constructors() -> MetaType {
        scale_info::meta_type::<EmptyCtorsMeta>()
    }

    fn services() -> impl Iterator<Item = (&'static str, AnyServiceMeta)> {
        [
            ("", AnyServiceMeta::new::<TestServiceMeta>()),
            ("SomeService", AnyServiceMeta::new::<TestServiceMeta>()),
        ]
        .into_iter()
    }
}

#[test]
fn program_idl_works_with_empty_ctors() {
    let mut idl = Vec::new();
    program::generate_idl::<TestProgramWithEmptyCtorsMeta>(&mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();
    let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);

    insta::assert_snapshot!(generated_idl);
    let generated_idl_program = generated_idl_program.unwrap();
    assert!(generated_idl_program.ctor().is_none());
    assert_eq!(generated_idl_program.services().len(), 1);
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    assert_eq!(generated_idl_program.types().len(), 8);
}

#[test]
fn program_idl_works_with_non_empty_ctors() {
    let mut idl = Vec::new();
    program::generate_idl::<TestProgramWithNonEmptyCtorsMeta>(&mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();
    let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);

    insta::assert_snapshot!(generated_idl);
    let generated_idl_program = generated_idl_program.unwrap();
    assert_eq!(generated_idl_program.ctor().unwrap().funcs().len(), 2);
    assert_eq!(generated_idl_program.services().len(), 1);
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    assert_eq!(generated_idl_program.types().len(), 8);
}

#[test]
fn program_idl_works_with_multiple_services() {
    let mut idl = Vec::new();
    program::generate_idl::<TestProgramWithMultipleServicesMeta>(&mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();
    let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);

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
fn service_idl_works_with_basics() {
    let mut idl = Vec::new();
    service::generate_idl::<TestServiceMeta>(&mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();
    let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);

    insta::assert_snapshot!(generated_idl);
    let generated_idl_program = generated_idl_program.unwrap();
    assert!(generated_idl_program.ctor().is_none());
    assert_eq!(generated_idl_program.services().len(), 1);
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    assert_eq!(generated_idl_program.types().len(), 8);
}

#[test]
fn service_idl_works_with_base_services() {
    let mut idl = Vec::new();
    service::generate_idl::<
        ServiceMetaWithBase<
            CommandsMeta,
            QueriesMeta,
            EventsMeta,
            ServiceMeta<BaseCommandsMeta, BaseQueriesMeta, BaseEventsMeta>,
        >,
    >(&mut idl)
    .unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();
    let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);

    insta::assert_snapshot!(generated_idl);
    let generated_idl_program = generated_idl_program.unwrap();
    assert!(generated_idl_program.ctor().is_none());
    assert_eq!(generated_idl_program.services().len(), 1);
    assert_eq!(generated_idl_program.services()[0].funcs().len(), 6);
    assert_eq!(generated_idl_program.types().len(), 8);
}

#[test]
fn service_idl_fails_with_base_services_and_ambiguous_events() {
    let mut idl = Vec::new();
    let result = service::generate_idl::<
        ServiceMetaWithBase<
            CommandsMeta,
            QueriesMeta,
            EventsMeta,
            ServiceMeta<BaseCommandsMeta, BaseQueriesMeta, AmbiguousBaseEventsMeta>,
        >,
    >(&mut idl);

    assert!(matches!(
        result,
        Err(sails_idl_gen::Error::EventMetaIsAmbiguous(_))
    ));
}
