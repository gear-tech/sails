use crate::{grammar::ProgramParser, lexer::Lexer};

pub mod visitor;

pub fn parse_idl(idl: &str) -> Result<Program, String> {
    let lexer = Lexer::new(idl);
    let parser = ProgramParser::new();
    let program = parser.parse(lexer).map_err(|e| format!("{:?}", e))?;
    Ok(program)
}

/// A structure describing program
#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    ctor: Option<Ctor>,
    services: Vec<Service>,
    types: Vec<Type>,
}

impl Program {
    pub(crate) fn new(ctor: Option<Ctor>, services: Vec<Service>, types: Vec<Type>) -> Self {
        Self {
            ctor,
            services,
            types,
        }
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
    pub(crate) fn new(funcs: Vec<CtorFunc>) -> Self {
        Self { funcs }
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
}

impl CtorFunc {
    pub(crate) fn new(name: String, params: Vec<FuncParam>) -> Self {
        Self { name, params }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[FuncParam] {
        &self.params
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
    pub(crate) fn new(name: String, funcs: Vec<ServiceFunc>, events: Vec<ServiceEvent>) -> Self {
        Self {
            name,
            funcs,
            events,
        }
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
}

impl ServiceFunc {
    pub(crate) fn new(
        name: String,
        params: Vec<FuncParam>,
        output: TypeDecl,
        is_query: bool,
    ) -> Self {
        Self {
            name,
            params,
            output,
            is_query,
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
}

impl Type {
    pub(crate) fn new(name: String, def: TypeDef) -> Self {
        Self { name, def }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn def(&self) -> &TypeDef {
        &self.def
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
            "h256" => Some(PrimitiveType::H256),
            "u256" => Some(PrimitiveType::U256),
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
    pub(crate) fn new(fields: Vec<StructField>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    name: Option<String>,
    type_decl: TypeDecl,
}

impl StructField {
    pub(crate) fn new(name: Option<String>, type_decl: TypeDecl) -> Self {
        Self { name, type_decl }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn type_decl(&self) -> &TypeDecl {
        &self.type_decl
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    variants: Vec<EnumVariant>,
}

impl EnumDef {
    pub(crate) fn new(variants: Vec<EnumVariant>) -> Self {
        Self { variants }
    }

    pub fn variants(&self) -> &[EnumVariant] {
        &self.variants
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    name: String,
    type_decl: Option<TypeDecl>,
}

impl EnumVariant {
    pub(crate) fn new(name: String, type_decl: Option<TypeDecl>) -> Self {
        Self { name, type_decl }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_decl(&self) -> Option<&TypeDecl> {
        self.type_decl.as_ref()
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
                DoThis : (p1: actor_id, p2: code_id, p3: message_id, p4: h256, p5: u256) -> null;
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
                _ => panic!("unexpected type"),
            });
    }

    #[test]
    fn parser_recognizes_methods_without_return_value() {
        let program_idl = r"
            service {
                Noop : (p1: u32);
            }
        ";

        let program = parse_idl(program_idl).unwrap();

        assert_eq!(program.services().len(), 1);
        let output = program.services()[0].funcs().first().unwrap().output();

        assert_eq!(
            *output,
            TypeDecl::Id(TypeId::Primitive(PrimitiveType::Null))
        );
    }

    #[test]
    fn parser_rejects_missing_return_value() {
        // "->"" should be either present with returned type or missing alltogether
        let program_idl = r"
            service {
                Noop : (p1: u32) ->;
            }
        ";

        let err = parse_idl(program_idl).unwrap_err();
        assert!(err.contains("InvalidReturn"));
    }
}
