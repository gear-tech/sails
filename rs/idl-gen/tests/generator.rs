use gprimitives::*;
use meta_params::*;
use sails_idl_gen::{GenMetaInfoBuilder, program2, service2};
use sails_idl_meta::{
    AnyServiceMeta, AnyServiceMetaFn, ProgramMeta, ServiceMeta as RtlServiceMeta,
};
use scale_info::{StaticTypeInfo, TypeInfo};
use std::{collections::BTreeMap, result::Result as StdResult};

#[allow(dead_code)]
mod types {
    use super::*;

    /// GenericStruct docs
    #[derive(TypeInfo)]
    pub struct GenericStruct<T> {
        /// GenericStruct field `p1`
        pub p1: T,
    }

    /// GenericConstStruct docs
    #[derive(TypeInfo)]
    pub struct GenericConstStruct<const N: usize> {
        /// GenericStruct field `field`
        field: [u8; N],
    }

    /// GenericEnum docs
    /// with two lines
    #[derive(TypeInfo)]
    pub enum GenericEnum<T1, T2> {
        /// GenericEnum `Variant1` of type 'T1'
        Variant1(T1),
        /// GenericEnum `Variant2` of type 'T2'
        Variant2(T2),
    }

    /// TupleStruct docs
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
        Nine(TupleVariantsDocs),
    }

    #[derive(TypeInfo)]
    pub enum TupleVariantsDocs {
        /// Docs for no tuple docs 1
        NoTupleDocs1(u32, String),
        NoTupleDocs2(CodeId, Vec<u8>),
        /// Docs for tuple docs 1
        TupleDocs1(
            u32,
            /// This is the second field
            String,
        ),
        TupleDocs2(
            /// This is the first field
            u32,
            /// This is the second field
            String,
        ),
        /// Docs for struct docs
        StructDocs {
            /// This is field `a`
            a: u32,
            /// This is field `b`
            b: String,
        },
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
        p7: GenericConstStruct<8>,
        p8: GenericConstStruct<32>,
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
    /// Some description
    DoThis(DoThisParams, String),
    /// Some multiline description
    /// Second line
    /// Third line
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
    /// This is a query
    This(ThisParams, StdResult<(String, u32), String>),
    /// This is a second query
    /// This is a second line
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
    /// `This` Done
    ThisDone(
        /// This is unnamed field, comments ignored
        u32,
    ),
    ThisDoneTwice(
        /// This is the first unnamed field
        u32,
        /// This is the second unnamed field
        u32,
    ),
    /// `That` Done too
    ThatDone {
        /// This is `p1` field
        p1: String,
    },
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
    ThisDone(u32), // Conflicts with `EventsMeta::ThisDone` even it has the same signature
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
    type CommandsMeta = C;
    type QueriesMeta = Q;
    type EventsMeta = E;
    const BASE_SERVICES: &'static [AnyServiceMetaFn] = &[];
    const ASYNC: bool = false;
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
    type CommandsMeta = C;
    type QueriesMeta = Q;
    type EventsMeta = E;
    const BASE_SERVICES: &'static [AnyServiceMetaFn] = &[AnyServiceMeta::new::<B>];
    const ASYNC: bool = false;
}

type TestServiceMeta = ServiceMeta<CommandsMeta, QueriesMeta, EventsMeta>;

#[allow(dead_code)]
#[derive(TypeInfo)]
enum EmptyCtorsMeta {}

struct TestProgramWithEmptyCtorsMeta;

impl ProgramMeta for TestProgramWithEmptyCtorsMeta {
    type ConstructorsMeta = EmptyCtorsMeta;

    const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] =
        &[("", AnyServiceMeta::new::<TestServiceMeta>)];

    const ASYNC: bool = false;
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum NonEmptyCtorsMeta {
    /// This is New constructor
    New(NoParams),
    /// This is FromStr constructor
    /// with second line
    FromStr(SingleParams<String>),
}

struct TestProgramWithNonEmptyCtorsMeta;

impl ProgramMeta for TestProgramWithNonEmptyCtorsMeta {
    type ConstructorsMeta = NonEmptyCtorsMeta;

    const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] =
        &[("TestServiceMeta", AnyServiceMeta::new::<TestServiceMeta>)];

    const ASYNC: bool = false;
}

struct TestProgramWithMultipleServicesMeta;

impl ProgramMeta for TestProgramWithMultipleServicesMeta {
    type ConstructorsMeta = EmptyCtorsMeta;

    const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
        ("TestServiceMeta1", AnyServiceMeta::new::<TestServiceMeta>),
        ("TestServiceMeta2", AnyServiceMeta::new::<TestServiceMeta>),
    ];

    const ASYNC: bool = false;
}

#[test]
fn program_idl_works_with_empty_ctors() {
    let mut idl = Vec::new();

    let meta_info = GenMetaInfoBuilder::new().program_name("EmptyCtorsProgram".to_string());
    program2::generate_idl::<TestProgramWithEmptyCtorsMeta>(meta_info, &mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();

    insta::assert_snapshot!(generated_idl);

    // let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);
    // let generated_idl_program = generated_idl_program.unwrap();
    // assert!(generated_idl_program.ctor().is_none());
    // assert_eq!(generated_idl_program.services().len(), 1);
    // assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    // assert_eq!(generated_idl_program.types().len(), 10);
}

#[test]
fn program_idl_works_with_non_empty_ctors() {
    let mut idl = Vec::new();

    let meta_info = GenMetaInfoBuilder::new().program_name("NonEmptyCtorsProgram".to_string());
    program2::generate_idl::<TestProgramWithNonEmptyCtorsMeta>(meta_info, &mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();

    insta::assert_snapshot!(generated_idl);

    // let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);
    // let generated_idl_program = generated_idl_program.unwrap();
    // assert_eq!(generated_idl_program.ctor().unwrap().funcs().len(), 2);
    // assert_eq!(generated_idl_program.services().len(), 1);
    // assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    // assert_eq!(generated_idl_program.types().len(), 10);
}

#[test]
fn program_idl_works_with_multiple_services() {
    let mut idl = Vec::new();

    let meta_info =
        GenMetaInfoBuilder::new().program_name("MultipleServicesNoCtorsProgram".to_string());
    program2::generate_idl::<TestProgramWithMultipleServicesMeta>(meta_info, &mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();

    insta::assert_snapshot!(generated_idl);

    // let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);
    // let generated_idl_program = generated_idl_program.unwrap();
    // assert!(generated_idl_program.ctor().is_none());
    // assert_eq!(generated_idl_program.services().len(), 2);
    // assert_eq!(generated_idl_program.services()[0].name(), "");
    // assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    // assert_eq!(generated_idl_program.services()[1].name(), "SomeService");
    // assert_eq!(generated_idl_program.services()[1].funcs().len(), 4);
    // assert_eq!(generated_idl_program.types().len(), 10);
}

#[test]
fn service_idl_works_with_basics() {
    let mut idl = Vec::new();

    let meta_info = GenMetaInfoBuilder::new()
        .author("Developer".to_string())
        .major_version(1);
    service2::generate_idl::<TestServiceMeta>(meta_info, &mut idl).unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();

    insta::assert_snapshot!(generated_idl);

    // let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);
    // let generated_idl_program = generated_idl_program.unwrap();
    // assert!(generated_idl_program.ctor().is_none());
    // assert_eq!(generated_idl_program.services().len(), 1);
    // assert_eq!(generated_idl_program.services()[0].funcs().len(), 4);
    // assert_eq!(generated_idl_program.types().len(), 10);
}

#[test]
fn service_idl_works_with_base_services() {
    let mut idl = Vec::new();

    let meta_info = GenMetaInfoBuilder::new()
        .author("Developer".to_string())
        .major_version(3)
        .minor_version(10);
    service2::generate_idl::<
        ServiceMetaWithBase<
            CommandsMeta,
            QueriesMeta,
            EventsMeta,
            ServiceMeta<BaseCommandsMeta, BaseQueriesMeta, BaseEventsMeta>,
        >,
    >(meta_info, &mut idl)
    .unwrap();
    let generated_idl = String::from_utf8(idl).unwrap();

    insta::assert_snapshot!(generated_idl);

    // let generated_idl_program = sails_idl_parser::ast::parse_idl(&generated_idl);
    // let generated_idl_program = generated_idl_program.unwrap();
    // assert!(generated_idl_program.ctor().is_none());
    // assert_eq!(generated_idl_program.services().len(), 1);
    // assert_eq!(generated_idl_program.services()[0].funcs().len(), 6);
    // assert_eq!(generated_idl_program.types().len(), 10);
}

#[test]
fn service_idl_fails_with_base_services_and_ambiguous_events() {
    let mut idl = Vec::new();
    let result = service2::generate_idl::<
        ServiceMetaWithBase<
            CommandsMeta,
            QueriesMeta,
            EventsMeta,
            ServiceMeta<BaseCommandsMeta, BaseQueriesMeta, AmbiguousBaseEventsMeta>,
        >,
    >(GenMetaInfoBuilder::new(), &mut idl);

    assert!(matches!(
        result,
        Err(sails_idl_gen::Error::EventMetaIsAmbiguous(_))
    ));
}
