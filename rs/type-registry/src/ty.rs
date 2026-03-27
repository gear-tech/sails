use crate::builder::TypeBuilder;
use crate::registry::TypeRef;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeParameter {
    pub name: String,
    pub arg: GenericArg,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GenericArg {
    Type(TypeRef),
    Const(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Annotation {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub module_path: String,
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub def: TypeDef,
    pub docs: Vec<String>,
    pub annotations: Vec<Annotation>,
}

impl Type {
    pub fn builder() -> TypeBuilder {
        TypeBuilder::new()
    }

    pub(crate) fn placeholder() -> Self {
        Self {
            module_path: String::new(),
            name: String::new(),
            type_params: Vec::new(),
            def: TypeDef::Tuple(Vec::new()),
            docs: Vec::new(),
            annotations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDef {
    Primitive(Primitive),
    #[cfg(feature = "gprimitives")]
    GPrimitive(GPrimitive),
    Composite(Composite),
    Variant(VariantDef),
    Sequence(TypeRef),
    Array {
        len: u32,
        type_param: TypeRef,
    },
    Tuple(Vec<TypeRef>),
    Map {
        key: TypeRef,
        value: TypeRef,
    },
    Option(TypeRef),
    Result {
        ok: TypeRef,
        err: TypeRef,
    },
    Parameter(String),
    Applied {
        base: TypeRef,
        args: Vec<TypeRef>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composite {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDef {
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
    pub docs: Vec<String>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Option<String>,
    pub ty: TypeRef,
    pub docs: Vec<String>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrayLen {
    Static(u32),
    Parameter(String),
}

impl From<u32> for ArrayLen {
    fn from(len: u32) -> Self {
        Self::Static(len)
    }
}

impl From<String> for ArrayLen {
    fn from(name: String) -> Self {
        Self::Parameter(name)
    }
}

impl From<&str> for ArrayLen {
    fn from(name: &str) -> Self {
        Self::Parameter(name.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Primitive {
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
}

#[cfg(feature = "gprimitives")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GPrimitive {
    U256,
    H160,
    H256,
    ActorId,
    MessageId,
    CodeId,
}

impl Annotation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }
}
