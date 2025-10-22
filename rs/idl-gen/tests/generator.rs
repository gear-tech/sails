use gprimitives::*;
use meta_params::*;
use sails_idl_gen::{program, service};
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
    ThisDone(u32), // Conflicts with `EventsMeta::ThisDone` even it has the same signatur
    ThatDoneBase { p1: u16 },
}

trait CommandEntryIds {
    const IDS: &'static [u16];
}

trait QueryEntryIds {
    const IDS: &'static [u16];
}

trait EventEntryIds {
    fn ids() -> Vec<u16>;
}

impl CommandEntryIds for CommandsMeta {
    const IDS: &'static [u16] = &[1, 2];
}

impl CommandEntryIds for BaseCommandsMeta {
    const IDS: &'static [u16] = &[1, 2];
}

impl QueryEntryIds for QueriesMeta {
    const IDS: &'static [u16] = &[1, 2];
}

impl QueryEntryIds for BaseQueriesMeta {
    const IDS: &'static [u16] = &[1, 2];
}

impl EventEntryIds for EventsMeta {
    fn ids() -> Vec<u16> {
        vec![1, 2, 3]
    }
}

impl EventEntryIds for BaseEventsMeta {
    fn ids() -> Vec<u16> {
        vec![1, 2]
    }
}

impl EventEntryIds for AmbiguousBaseEventsMeta {
    fn ids() -> Vec<u16> {
        vec![1, 2]
    }
}

struct ServiceMeta<C, Q, E> {
    _commands: std::marker::PhantomData<C>,
    _queries: std::marker::PhantomData<Q>,
    _events: std::marker::PhantomData<E>,
}

impl<
    C: StaticTypeInfo + CommandEntryIds,
    Q: StaticTypeInfo + QueryEntryIds,
    E: StaticTypeInfo + EventEntryIds,
> RtlServiceMeta for ServiceMeta<C, Q, E>
{
    type CommandsMeta = C;
    type QueriesMeta = Q;
    type EventsMeta = E;
    // Test scaffolding does not rely on a real path; empty marker keeps the trait happy.
    const INTERFACE_PATH: &'static str = "";
    const BASE_SERVICES: &'static [AnyServiceMetaFn] = &[];
    const ASYNC: bool = false;

    fn command_entry_ids() -> Vec<u16> {
        C::IDS.to_vec()
    }

    fn local_command_entry_ids() -> &'static [u16] {
        C::IDS
    }

    fn query_entry_ids() -> Vec<u16> {
        Q::IDS.to_vec()
    }

    fn local_query_entry_ids() -> &'static [u16] {
        Q::IDS
    }

    fn event_entry_ids() -> Vec<u16> {
        E::ids()
    }

    fn local_event_entry_ids() -> Vec<u16> {
        E::ids()
    }
}

struct ServiceMetaWithBase<C, Q, E, B> {
    _commands: std::marker::PhantomData<C>,
    _queries: std::marker::PhantomData<Q>,
    _events: std::marker::PhantomData<E>,
    _base: std::marker::PhantomData<B>,
}

impl<
    C: StaticTypeInfo + CommandEntryIds,
    Q: StaticTypeInfo + QueryEntryIds,
    E: StaticTypeInfo + EventEntryIds,
    B: RtlServiceMeta,
> RtlServiceMeta for ServiceMetaWithBase<C, Q, E, B>
{
    type CommandsMeta = C;
    type QueriesMeta = Q;
    type EventsMeta = E;
    // Same as above: the mocked base service metadata does not need a path.
    const INTERFACE_PATH: &'static str = "";
    const BASE_SERVICES: &'static [AnyServiceMetaFn] = &[AnyServiceMeta::new::<B>];
    const ASYNC: bool = false;

    fn command_entry_ids() -> Vec<u16> {
        let mut ids = C::IDS.to_vec();
        ids.extend(B::command_entry_ids());
        ids
    }

    fn local_command_entry_ids() -> &'static [u16] {
        C::IDS
    }

    fn query_entry_ids() -> Vec<u16> {
        let mut ids = Q::IDS.to_vec();
        ids.extend(B::query_entry_ids());
        ids
    }

    fn local_query_entry_ids() -> &'static [u16] {
        Q::IDS
    }

    fn event_entry_ids() -> Vec<u16> {
        let mut ids = E::ids();
        ids.extend(B::event_entry_ids());
        ids
    }

    fn local_event_entry_ids() -> Vec<u16> {
        E::ids()
    }
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
        &[("", AnyServiceMeta::new::<TestServiceMeta>)];

    const ASYNC: bool = false;
}

struct TestProgramWithMultipleServicesMeta;

impl ProgramMeta for TestProgramWithMultipleServicesMeta {
    type ConstructorsMeta = EmptyCtorsMeta;

    const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
        ("", AnyServiceMeta::new::<TestServiceMeta>),
        ("SomeService", AnyServiceMeta::new::<TestServiceMeta>),
    ];

    const ASYNC: bool = false;
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
    assert_eq!(generated_idl_program.types().len(), 10);
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
    assert_eq!(generated_idl_program.types().len(), 10);
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
    assert_eq!(generated_idl_program.types().len(), 10);
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
    assert_eq!(generated_idl_program.types().len(), 10);
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
    assert_eq!(generated_idl_program.types().len(), 10);
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
