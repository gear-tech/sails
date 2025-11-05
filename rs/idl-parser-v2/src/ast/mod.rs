pub mod visitor;

// -------------------------------- Target model ---------------------------------

/// A structure describing program
#[derive(Debug, Default, PartialEq, Clone)]
pub struct IdlDoc {
    pub globals: Vec<(String, Option<String>)>,
    pub program: Option<ProgramUnit>,
    pub services: Vec<ServiceUnit>,
}

/// A structure describing program
#[derive(Debug, Default, PartialEq, Clone)]
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
    pub route: String,
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

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct FuncParam {
    pub name: String,
    pub type_decl: TypeDecl,
}

pub type ServiceEvent = EnumVariant;

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDef {
    pub fields: Vec<StructField>,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub def: StructDef,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}
