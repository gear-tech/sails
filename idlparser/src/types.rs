#[derive(Debug)]
pub struct Program {
    pub items: Vec<ProgramItem>,
}

#[derive(Debug)]
pub enum ProgramItem {
    Service(Service),
    Type(Type),
}

#[derive(Debug)]
pub struct Service {
    pub funcs: Vec<Func>,
}

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub output: TypeDecl,
    pub is_query: bool,
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub r#type: TypeDecl,
}

#[derive(Debug)]
pub struct Type {
    pub name: String,
    pub def: TypeDef,
}

#[derive(Debug)]
pub enum TypeDecl {
    Opt(Box<TypeDecl>),
    Vec(Box<TypeDecl>),
    Result {
        ok: Box<TypeDecl>,
        err: Box<TypeDecl>,
    },
    Null,
    Id(TypeId),
    Def(TypeDef),
}

#[derive(Debug)]
pub enum TypeId {
    Primitive(PrimitiveType),
    UserDefined(String),
}

#[derive(Debug)]
pub enum PrimitiveType {
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
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug)]
pub struct StructDef {
    pub fields: Vec<StructField>,
}

#[derive(Debug)]
pub struct StructField {
    pub name: Option<String>,
    pub r#type: TypeDecl,
}

#[derive(Debug)]
pub struct EnumDef {
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub r#type: Option<TypeDecl>,
}
