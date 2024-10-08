use std::collections::HashSet;
use thiserror::Error;

use crate::{
    grammar::ProgramParser,
    lexer::{Lexer, LexicalError},
};

pub mod visitor;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error(transparent)]
    Lexical(#[from] LexicalError),
    #[error("duplicate type `{0}`")]
    DuplicateType(String),
    #[error("duplicate ctor `{0}`")]
    DuplicateCtor(String),
    #[error("duplicate service `{0}`")]
    DuplicateService(String),
    #[error("duplicate service method `{method}` in service `{service}`")]
    DuplicateServiceMethod { method: String, service: String },
    #[error("duplicate variant `{0}`")]
    DuplicateEnumVariant(String),
    #[error("duplicate field `{0}`")]
    DuplicateStructField(String),
    #[error("struct has mixed named and unnamed fields")]
    StructMixedFields,
    #[error("parse error: `{0}`")]
    Other(String),
}

pub fn parse_idl(idl: &str) -> Result<Program, ParseError> {
    let lexer = Lexer::new(idl);
    let parser = ProgramParser::new();

    let program = parser.parse(lexer).map_err(|e| match e {
        lalrpop_util::ParseError::User { error } => error,
        _ => ParseError::Other(e.to_string()),
    })?;

    Ok(program)
}

type ParseResult<T> = Result<T, ParseError>;

/// A structure describing program
#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    ctor: Option<Ctor>,
    services: Vec<Service>,
    types: Vec<Type>,
}

impl Program {
    pub(crate) fn new(
        ctor: Option<Ctor>,
        services: Vec<Service>,
        types: Vec<Type>,
    ) -> ParseResult<Self> {
        let mut seen_types = HashSet::new();
        for t in &types {
            if !seen_types.insert(t.name().to_lowercase()) {
                return Err(ParseError::DuplicateType(t.name().to_string()));
            }
        }

        let mut seen_services = HashSet::new();
        for s in &services {
            if !seen_services.insert(s.name().to_lowercase()) {
                return Err(ParseError::DuplicateService(s.name().to_string()));
            }
        }

        Ok(Self {
            ctor,
            services,
            types,
        })
    }

    pub fn ctor(&self) -> Option<&Ctor> {
        self.ctor.as_ref()
    }

    pub fn services(&self) -> &[Service] {
        &self.services
    }

    pub fn types(&self) -> &[Type] {
        &self.types
    }
}

/// A structure describing program constructor
#[derive(Debug, PartialEq, Clone)]
pub struct Ctor {
    funcs: Vec<CtorFunc>,
}

impl Ctor {
    pub(crate) fn new(funcs: Vec<CtorFunc>) -> ParseResult<Self> {
        let mut seen = HashSet::new();
        for f in &funcs {
            if !seen.insert(f.name().to_lowercase()) {
                return Err(ParseError::DuplicateCtor(f.name().to_string()));
            }
        }
        Ok(Self { funcs })
    }

    pub fn funcs(&self) -> &[CtorFunc] {
        &self.funcs
    }
}

/// A structure describing one of program constructor functions
#[derive(Debug, PartialEq, Clone)]
pub struct CtorFunc {
    name: String,
    params: Vec<FuncParam>,
    docs: Vec<String>,
}

impl CtorFunc {
    pub(crate) fn new(name: String, params: Vec<FuncParam>, docs: Vec<String>) -> Self {
        Self { name, params, docs }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[FuncParam] {
        &self.params
    }

    pub fn docs(&self) -> &Vec<String> {
        &self.docs
    }
}

/// A structure describing one of program services
#[derive(Debug, PartialEq, Clone)]
pub struct Service {
    name: String,
    funcs: Vec<ServiceFunc>,
    events: Vec<ServiceEvent>,
}

impl Service {
    pub(crate) fn new(
        name: String,
        funcs: Vec<ServiceFunc>,
        events: Vec<ServiceEvent>,
    ) -> ParseResult<Self> {
        let mut seen = HashSet::new();
        for f in &funcs {
            if !seen.insert(f.name().to_lowercase()) {
                return Err(ParseError::DuplicateServiceMethod {
                    method: f.name().to_string(),
                    service: name.to_string(),
                });
            }
        }

        Ok(Self {
            name,
            funcs,
            events,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn funcs(&self) -> &[ServiceFunc] {
        &self.funcs
    }

    pub fn events(&self) -> &[ServiceEvent] {
        &self.events
    }
}

/// A structure describing one of service functions
#[derive(Debug, PartialEq, Clone)]
pub struct ServiceFunc {
    name: String,
    params: Vec<FuncParam>,
    output: TypeDecl,
    is_query: bool,
    docs: Vec<String>,
}

impl ServiceFunc {
    pub(crate) fn new(
        name: String,
        params: Vec<FuncParam>,
        output: TypeDecl,
        is_query: bool,
        docs: Vec<String>,
    ) -> Self {
        Self {
            name,
            params,
            output,
            is_query,
            docs,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[FuncParam] {
        &self.params
    }

    pub fn output(&self) -> &TypeDecl {
        &self.output
    }

    pub fn is_query(&self) -> bool {
        self.is_query
    }

    pub fn docs(&self) -> &Vec<String> {
        &self.docs
    }
}

/// A structure describing one of service events
pub type ServiceEvent = EnumVariant;

#[derive(Debug, PartialEq, Clone)]
pub struct FuncParam {
    name: String,
    type_decl: TypeDecl,
}

impl FuncParam {
    pub(crate) fn new(name: String, type_decl: TypeDecl) -> Self {
        Self { name, type_decl }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_decl(&self) -> &TypeDecl {
        &self.type_decl
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    name: String,
    def: TypeDef,
    docs: Vec<String>,
}

impl Type {
    pub(crate) fn new(name: String, def: TypeDef, docs: Vec<String>) -> Self {
        Self { name, def, docs }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn def(&self) -> &TypeDef {
        &self.def
    }

    pub fn docs(&self) -> &Vec<String> {
        &self.docs
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDecl {
    Vector(Box<TypeDecl>),
    Array {
        item: Box<TypeDecl>,
        len: u32,
    },
    Map {
        key: Box<TypeDecl>,
        value: Box<TypeDecl>,
    },
    Optional(Box<TypeDecl>),
    Result {
        ok: Box<TypeDecl>,
        err: Box<TypeDecl>,
    },
    Id(TypeId),
    Def(TypeDef),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeId {
    Primitive(PrimitiveType),
    UserDefined(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum PrimitiveType {
    Null,
    Bool,
    Char,
    Str,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    ActorId,
    CodeId,
    MessageId,
    H256,
    U256,
    H160,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroU256,
}

impl PrimitiveType {
    pub(crate) fn str_to_enum(str: &str) -> Option<Self> {
        match str {
            "bool" => Some(PrimitiveType::Bool),
            "char" => Some(PrimitiveType::Char),
            "str" => Some(PrimitiveType::Str),
            "u8" => Some(PrimitiveType::U8),
            "u16" => Some(PrimitiveType::U16),
            "u32" => Some(PrimitiveType::U32),
            "u64" => Some(PrimitiveType::U64),
            "u128" => Some(PrimitiveType::U128),
            "i8" => Some(PrimitiveType::I8),
            "i16" => Some(PrimitiveType::I16),
            "i32" => Some(PrimitiveType::I32),
            "i64" => Some(PrimitiveType::I64),
            "i128" => Some(PrimitiveType::I128),
            "h160" => Some(PrimitiveType::H160),
            "h256" => Some(PrimitiveType::H256),
            "u256" => Some(PrimitiveType::U256),
            "nat8" => Some(PrimitiveType::NonZeroU8),
            "nat16" => Some(PrimitiveType::NonZeroU16),
            "nat32" => Some(PrimitiveType::NonZeroU32),
            "nat64" => Some(PrimitiveType::NonZeroU64),
            "nat128" => Some(PrimitiveType::NonZeroU128),
            "nat256" => Some(PrimitiveType::NonZeroU256),
            "actor_id" => Some(PrimitiveType::ActorId),
            "code_id" => Some(PrimitiveType::CodeId),
            "message_id" => Some(PrimitiveType::MessageId),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDef {
    fields: Vec<StructField>,
}

impl StructDef {
    pub(crate) fn new(fields: Vec<StructField>) -> ParseResult<Self> {
        // check if all fields are named or unnamed
        let all_unnamed = fields.iter().all(|f| f.name().is_none());
        let all_named = fields.iter().all(|f| f.name().is_some());

        if !all_unnamed && !all_named {
            return Err(ParseError::StructMixedFields);
        }

        let mut seen = HashSet::new();

        if all_named {
            for f in &fields {
                let name = f.name().unwrap();
                if !seen.insert(name.to_lowercase()) {
                    return Err(ParseError::DuplicateStructField(name.to_string()));
                }
            }
        }

        Ok(Self { fields })
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    name: Option<String>,
    type_decl: TypeDecl,
    docs: Vec<String>,
}

impl StructField {
    pub(crate) fn new(name: Option<String>, type_decl: TypeDecl, docs: Vec<String>) -> Self {
        Self {
            name,
            type_decl,
            docs,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn type_decl(&self) -> &TypeDecl {
        &self.type_decl
    }

    pub fn docs(&self) -> &Vec<String> {
        &self.docs
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    variants: Vec<EnumVariant>,
}

impl EnumDef {
    pub(crate) fn new(variants: Vec<EnumVariant>) -> ParseResult<Self> {
        let mut seen = HashSet::new();
        for v in &variants {
            if !seen.insert(v.name().to_lowercase()) {
                return Err(ParseError::DuplicateEnumVariant(v.name().to_string()));
            }
        }
        Ok(Self { variants })
    }

    pub fn variants(&self) -> &[EnumVariant] {
        &self.variants
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    name: String,
    type_decl: Option<TypeDecl>,
    docs: Vec<String>,
}

impl EnumVariant {
    pub(crate) fn new(name: String, type_decl: Option<TypeDecl>, docs: Vec<String>) -> Self {
        Self {
            name,
            type_decl,
            docs,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_decl(&self) -> Option<&TypeDecl> {
        self.type_decl.as_ref()
    }

    pub fn docs(&self) -> &Vec<String> {
        &self.docs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_works() {
        let program_idl = r"
          type ThisThatSvcAppTupleStruct = struct {
            bool,
          };

          type ThisThatSvcAppDoThatParam = struct {
            p1: u32,
            p2: str,
            p3: ThisThatSvcAppManyVariants,
          };

          type ThisThatSvcAppManyVariants = enum {
            One,
            Two: u32,
            Three: opt u256,
            Four: struct { a: u32, b: opt u16 },
            Five: struct { str, h256 },
            Six: struct { u32 },
            Seven: [map (u32, str), 10],
            Eight: actor_id,
          };

          constructor {
            New : (p1: u32);
          };

          service {
            DoThis : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: ThisThatSvcAppTupleStruct) -> struct { str, u32 };
            DoThat : (param: ThisThatSvcAppDoThatParam) -> result (struct { str, u32 }, struct { str });
            query This : (v1: vec u16) -> u32;
            query That : (v1: null) -> result (str, str);

            events {
                ThisDone;
                ThatDone: u32;
                SomethingHappened: struct { str, u32 };
                SomethingDone: ThisThatSvcAppManyVariants;
            }
          };
        ";

        let program = parse_idl(program_idl).unwrap();

        assert_eq!(program.types().len(), 3);
        assert_eq!(program.ctor().unwrap().funcs().len(), 1);
        assert_eq!(program.services().len(), 1);
        assert_eq!(program.services()[0].funcs().len(), 4);
        assert_eq!(program.services()[0].events().len(), 4);

        //println!("ast: {:#?}", program);
    }

    #[test]
    fn parser_accepts_types_service() {
        let program_idl = r"
          type T = enum { One };
          service {}
        ";

        let program = parse_idl(program_idl).unwrap();

        assert_eq!(program.types().len(), 1);
        assert_eq!(program.services().len(), 1);
        assert_eq!(program.services()[0].funcs().len(), 0);
    }

    #[test]
    fn parser_accepts_ctor_service() {
        let program_idl = r"
          constructor {};
          service {}
        ";

        let program = parse_idl(program_idl).unwrap();

        assert_eq!(program.ctor().unwrap().funcs().len(), 0);
        assert_eq!(program.services().len(), 1);
        assert_eq!(program.services()[0].funcs().len(), 0);
    }

    #[test]
    fn parser_accepts_multiple_services() {
        let program_idl = r"
          service {};
          service SomeService {};
        ";

        let program = parse_idl(program_idl).unwrap();

        assert_eq!(program.services().len(), 2);
        assert_eq!(program.services()[0].name(), "");
        assert_eq!(program.services()[1].name(), "SomeService");
    }

    #[test]
    fn parser_accepts_types_ctor_service() {
        let program_idl = r"
          type T = enum { One };
          constructor {};
          service {}
        ";

        let program = parse_idl(program_idl).unwrap();

        assert_eq!(program.types().len(), 1);
        assert_eq!(program.ctor().unwrap().funcs().len(), 0);
        assert_eq!(program.services().len(), 1);
        assert_eq!(program.services()[0].funcs().len(), 0);
    }

    #[test]
    fn parser_requires_semicolon_between_types_and_service() {
        let program_idl = r"
          type T = enum { One }
          service {}
        ";

        let program = parse_idl(program_idl);

        assert!(program.is_err());
    }

    #[test]
    fn parser_recognizes_builtin_types_as_primitives() {
        let program_idl = r"
            service {
                DoThis : (p1: actor_id, p2: code_id, p3: message_id, p4: h256, p5: u256, p6: h160) -> null;
            }
        ";

        let program = parse_idl(program_idl).unwrap();

        assert_eq!(program.services().len(), 1);
        program.services()[0]
            .funcs()
            .first()
            .unwrap()
            .params()
            .iter()
            .for_each(|p| match p.type_decl() {
                TypeDecl::Id(TypeId::Primitive(PrimitiveType::ActorId)) => {
                    assert_eq!(p.name(), "p1");
                }
                TypeDecl::Id(TypeId::Primitive(PrimitiveType::CodeId)) => {
                    assert_eq!(p.name(), "p2");
                }
                TypeDecl::Id(TypeId::Primitive(PrimitiveType::MessageId)) => {
                    assert_eq!(p.name(), "p3");
                }
                TypeDecl::Id(TypeId::Primitive(PrimitiveType::H256)) => {
                    assert_eq!(p.name(), "p4");
                }
                TypeDecl::Id(TypeId::Primitive(PrimitiveType::U256)) => {
                    assert_eq!(p.name(), "p5");
                }
                TypeDecl::Id(TypeId::Primitive(PrimitiveType::H160)) => {
                    assert_eq!(p.name(), "p6");
                }
                _ => panic!("unexpected type"),
            });
    }

    #[test]
    fn parser_rejects_duplicate_names() {
        let program_idl = r"
          type A = enum { One };
          type A = enum { Two };
          service {};
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(err, ParseError::DuplicateType("A".to_owned()));
    }

    #[test]
    fn parser_rejects_duplicate_unnamed_services() {
        let program_idl = r"
          service {};
          service {};
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(err, ParseError::DuplicateService("".to_owned()));
    }

    #[test]
    fn parser_rejects_duplicate_named_services() {
        let program_idl = r"
          service A {};
          service B {};
          service A {};
          service {};
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(err, ParseError::DuplicateService("A".to_owned()));
    }

    #[test]
    fn parser_rejects_duplicate_service_methods() {
        let program_idl = r"
          service {
            DoTHIS : () -> null;
            DoThis : () -> null;
          };
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(
            err,
            ParseError::DuplicateServiceMethod {
                method: "DoThis".to_owned(),
                service: "".to_owned()
            }
        );
    }

    #[test]
    fn parser_rejects_duplicate_ctor_funcs() {
        let program_idl = r"
          constructor {
            New : ();
            new : ();
          };
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(err, ParseError::DuplicateCtor("new".to_owned()));
    }

    #[test]
    fn parser_rejects_duplicate_enum_variants() {
        let program_idl = r"
          type T = enum { One, One };
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(err, ParseError::DuplicateEnumVariant("One".to_owned()));
    }

    #[test]
    fn parser_rejects_duplicate_struct_fields() {
        let program_idl = r"
          type T = struct {
            a: u32,
            a: u32,
          };
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(err, ParseError::DuplicateStructField("a".to_owned()));
    }

    #[test]
    fn parser_rejects_mixed_named_unnamed_struct_fields() {
        let program_idl = r"
          type T = struct {
            a: u32,
            u32,
          };
        ";

        let err = parse_idl(program_idl).unwrap_err();

        assert_eq!(err, ParseError::StructMixedFields);
    }

    #[test]
    fn parser_accepts_struct_field_reserved_keywords() {
        const IDL: &str = r#"
        type MyStruct = struct {
            query: u8,
            result: u8,
        };
        "#;

        let expected = TypeDef::Struct(
            StructDef::new(vec![
                StructField::new(
                    Some("query".to_owned()),
                    TypeDecl::Id(TypeId::Primitive(PrimitiveType::U8)),
                    vec![],
                ),
                StructField::new(
                    Some("result".to_owned()),
                    TypeDecl::Id(TypeId::Primitive(PrimitiveType::U8)),
                    vec![],
                ),
            ])
            .unwrap(),
        );

        // act
        let program = parse_idl(IDL).unwrap();

        // assert
        let my_struct = program
            .types()
            .iter()
            .find(|t| t.name() == "MyStruct")
            .unwrap();
        assert_eq!(&expected, my_struct.def());
    }

    #[test]
    fn parser_accepts_func_param_reserved_keywords() {
        const IDL: &str = r#"
            service {
                /// DoThis comment
                DoThis : (constructor: u8, service: u8, events: vec u8) -> null;
            }
        "#;

        let expected = Service::new(
            "".to_owned(),
            vec![ServiceFunc::new(
                "DoThis".to_owned(),
                vec![
                    FuncParam::new(
                        "constructor".to_owned(),
                        TypeDecl::Id(TypeId::Primitive(PrimitiveType::U8)),
                    ),
                    FuncParam::new(
                        "service".to_owned(),
                        TypeDecl::Id(TypeId::Primitive(PrimitiveType::U8)),
                    ),
                    FuncParam::new(
                        "events".to_owned(),
                        TypeDecl::Vector(Box::new(TypeDecl::Id(TypeId::Primitive(
                            PrimitiveType::U8,
                        )))),
                    ),
                ],
                TypeDecl::Id(TypeId::Primitive(PrimitiveType::Null)),
                false,
                vec!["DoThis comment".to_owned()],
            )],
            vec![],
        )
        .unwrap();

        // act
        let program = parse_idl(IDL).unwrap();

        // assert
        let my_service = program.services().iter().find(|t| t.name() == "").unwrap();
        assert_eq!(&expected, my_service);
    }

    #[test]
    fn parser_accepts_nonzero_primitives() {
        const IDL: &str = r#"
        type MyStruct = struct {
            /// field `query`
            query: nat32,
            data: nat256,
            /// field `result`
            /// second line
            result: nat8
        };
        "#;

        let expected = TypeDef::Struct(
            StructDef::new(vec![
                StructField::new(
                    Some("query".to_owned()),
                    TypeDecl::Id(TypeId::Primitive(PrimitiveType::NonZeroU32)),
                    vec!["field `query`".into()],
                ),
                StructField::new(
                    Some("data".to_owned()),
                    TypeDecl::Id(TypeId::Primitive(PrimitiveType::NonZeroU256)),
                    vec![],
                ),
                StructField::new(
                    Some("result".to_owned()),
                    TypeDecl::Id(TypeId::Primitive(PrimitiveType::NonZeroU8)),
                    vec!["field `result`".into(), "second line".into()],
                ),
            ])
            .unwrap(),
        );

        // act
        let program = parse_idl(IDL).unwrap();

        // assert
        let my_struct = program
            .types()
            .iter()
            .find(|t| t.name() == "MyStruct")
            .unwrap();
        assert_eq!(&expected, my_struct.def());
    }
}
