use alloc::{boxed::Box, format, string::String, vec::Vec};
use askama::Template;
use core::fmt::{Display, Write};

// -------------------------------- Target model ---------------------------------

/// A structure describing program
#[derive(Debug, Default, PartialEq, Clone, Template)]
#[template(path = "idl.askama", escape = "none")]
pub struct IdlDoc {
    pub globals: Vec<(String, Option<String>)>,
    pub program: Option<ProgramUnit>,
    pub services: Vec<ServiceUnit>,
}

/// A structure describing program
#[derive(Debug, Default, PartialEq, Clone, Template)]
#[template(path = "program.askama", escape = "none")]
pub struct ProgramUnit {
    pub name: String,
    pub ctors: Vec<CtorFunc>,
    pub services: Vec<ProgramServiceItem>,
    pub types: Vec<Type>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// A structure describing program
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ProgramServiceItem {
    pub name: String,
    pub route: Option<String>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// A structure describing one of program constructor functions
#[derive(Debug, PartialEq, Clone)]
pub struct CtorFunc {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone, Template)]
#[template(path = "service.askama", escape = "none")]
pub struct ServiceUnit {
    pub name: String,
    pub extends: Vec<String>,
    pub funcs: Vec<ServiceFunc>,
    pub events: Vec<ServiceEvent>,
    pub types: Vec<Type>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ServiceFunc {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub output: TypeDecl,
    pub throws: Option<TypeDecl>,
    pub is_query: bool,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

impl ServiceFunc {
    pub fn is_return_void(&self) -> bool {
        use PrimitiveType::*;
        use TypeDecl::*;
        self.output == Primitive(Void)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncParam {
    pub name: String,
    pub type_decl: TypeDecl,
}

impl Display for FuncParam {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let FuncParam { name, type_decl } = self;
        write!(f, "{name}: {type_decl}")
    }
}

pub type ServiceEvent = EnumVariant;

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDecl {
    Slice(Box<TypeDecl>),
    Array {
        item: Box<TypeDecl>,
        len: u32,
    },
    Tuple(Vec<TypeDecl>),
    Option(Box<TypeDecl>),
    Result {
        ok: Box<TypeDecl>,
        err: Box<TypeDecl>,
    },
    Primitive(PrimitiveType),
    UserDefined {
        path: String,
        generics: Vec<TypeDecl>,
    },
}

impl Display for TypeDecl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TypeDecl::*;
        match self {
            Slice(type_decl) => write!(f, "[{type_decl}]"),
            Array { item, len } => write!(f, "[{item};{len}]"),
            Tuple(type_decls) => {
                f.write_char('(')?;
                for (i, ty) in type_decls.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                f.write_char(')')?;
                Ok(())
            }
            Option(type_decl) => write!(f, "Option<{type_decl}>"),
            Result { ok, err } => write!(f, "Result<{ok}, {err}>"),
            Primitive(primitive_type) => write!(f, "{primitive_type}"),
            UserDefined { path, generics } => {
                write!(f, "{path}")?;
                if !generics.is_empty() {
                    f.write_char('<')?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            f.write_str(", ")?;
                        }
                        write!(f, "{g}")?;
                    }
                    f.write_char('>')?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum PrimitiveType {
    Void,
    Bool,
    Char,
    String,
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
}

impl PrimitiveType {
    pub fn as_str(&self) -> &'static str {
        use PrimitiveType::*;
        match self {
            Void => "()",
            Bool => "bool",
            Char => "char",
            String => "string",
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
            U128 => "u128",
            I8 => "i8",
            I16 => "i16",
            I32 => "i32",
            I64 => "i64",
            I128 => "i128",
            ActorId => "ActorId",
            CodeId => "CodeId",
            MessageId => "MessageId",
            H256 => "H256",
            U256 => "U256",
            H160 => "H160",
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for PrimitiveType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PrimitiveType::*;
        match s {
            "()" => Ok(Void),
            "bool" => Ok(Bool),
            "char" => Ok(Char),
            "String" | "string" => Ok(String),
            "u8" => Ok(U8),
            "u16" => Ok(U16),
            "u32" => Ok(U32),
            "u64" => Ok(U64),
            "u128" => Ok(U128),
            "i8" => Ok(I8),
            "i16" => Ok(I16),
            "i32" => Ok(I32),
            "i64" => Ok(I64),
            "i128" => Ok(I128),

            "ActorId" | "actorid" | "actor_id" => Ok(ActorId),
            "CodeId" | "codeid" | "code_id" => Ok(CodeId),
            "MessageId" | "messageid" | "message_id" => Ok(MessageId),

            "H256" | "h256" => Ok(H256),
            "U256" | "u256" => Ok(U256),
            "H160" | "h160" => Ok(H160),

            other => Err(format!("Unknown primitive type: {other}")),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Template)]
#[template(path = "type.askama", escape = "none")]
pub struct Type {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub def: TypeDef,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParameter {
    /// The name of the generic type parameter e.g. "T".
    pub name: String,
    /// The concrete type for the type parameter.
    ///
    /// `None` if the type parameter is skipped.
    pub ty: Option<TypeDecl>,
}

impl Display for TypeParameter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let TypeParameter { name, ty } = self;
        if let Some(ty) = ty.as_ref() {
            write!(f, "{name} = {ty}")
        } else {
            write!(f, "{name}")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug, PartialEq, Clone, Template)]
#[template(path = "struct_def.askama", escape = "none")]
pub struct StructDef {
    pub fields: Vec<StructField>,
}

impl StructDef {
    pub fn is_unit(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn is_inline(&self) -> bool {
        self.fields
            .iter()
            .all(|f| f.name.is_none() && f.docs.is_empty() && f.annotations.is_empty())
    }

    pub fn is_tuple(&self) -> bool {
        self.fields.iter().all(|f| f.name.is_none())
    }
}

#[derive(Debug, PartialEq, Clone, Template)]
#[template(path = "field.askama", escape = "none")]
pub struct StructField {
    pub name: Option<String>,
    pub type_decl: TypeDecl,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, PartialEq, Clone, Template)]
#[template(path = "variant.askama", escape = "none")]
pub struct EnumVariant {
    pub name: String,
    pub def: StructDef,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}
