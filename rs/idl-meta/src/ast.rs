use alloc::{
    boxed::Box,
    format,
    string::{String, ToString as _},
    vec,
    vec::Vec,
};
use core::fmt::{Display, Write};

// -------------------------------- IDL model ---------------------------------

/// Root AST node representing a single parsed Sails IDL document.
///
/// Mirrors one `.idl` file from the specification:
/// - `globals` correspond to global `!@...` annotations at the top of the file;
/// - `program` holds an optional `program <ident> { ... }` block;
/// - `services` contains all top-level `service <ident> { ... }` definitions.
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(
    feature = "templates",
    derive(askama::Template),
    template(path = "idl.askama", escape = "none")
)]
pub struct IdlDoc {
    pub globals: Vec<(String, Option<String>)>,
    pub program: Option<ProgramUnit>,
    pub services: Vec<ServiceUnit>,
}

/// AST node describing a `program` block.
///
/// A program is an entry point that:
/// - declares constructor functions in `constructors { ... }`,
/// - exposes one or more services in `services { ... }`,
/// - may define shared types in `types { ... }`,
/// - may contain documentation comments and annotations.
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(
    feature = "templates",
    derive(askama::Template),
    template(path = "program.askama", escape = "none")
)]
pub struct ProgramUnit {
    pub name: String,
    pub ctors: Vec<CtorFunc>,
    pub services: Vec<ServiceExpo>,
    pub types: Vec<Type>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// Single service export entry inside a `program { services { ... } }` section.
///
/// Represents a link between:
/// - the exported service name visible to the client,
/// - an optional low-level `route` (transport / path) used by the runtime,
/// - may contain documentation comments and annotations.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ServiceExpo {
    pub name: String,
    pub route: Option<String>,
    // TODO: interface_id: [u8; 8],
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// Constructor function of a program, declared in `constructors { ... }`.
///
/// A constructor describes how to create or initialize a program instance:
/// - `name` is the constructor identifier,
/// - `params` are the IDL-level arguments,
/// - may contain documentation comments and annotations.
#[derive(Debug, Clone, PartialEq)]
pub struct CtorFunc {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// AST node describing a `service` definition.
///
/// A service is an interface that:
/// - may `extends` other services, inheriting their events, functions and types,
/// - defines `funcs` in `functions { ... }`,
/// - defines `events` in `events { ... }`,
/// - defines service-local `types { ... }`,
/// - may contain documentation comments and annotations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "templates",
    derive(askama::Template),
    template(path = "service.askama", escape = "none")
)]
pub struct ServiceUnit {
    pub name: String,
    pub extends: Vec<String>,
    pub funcs: Vec<ServiceFunc>,
    pub events: Vec<ServiceEvent>,
    pub types: Vec<Type>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// Service function entry inside `service { functions { ... } }`.
///
/// - `params` is the ordered list of IDL parameters;
/// - `output` is the return type (use `PrimitiveType::Void` for `()` / no value);
/// - `throws` is an optional error type after the `throws` keyword;
/// - `is_query` marks read-only / query functions as defined by the spec;
/// - may contain documentation comments and annotations.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceFunc {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub output: TypeDecl,
    pub throws: Option<TypeDecl>,
    pub kind: FunctionKind,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// Function kind based on mutability.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum FunctionKind {
    #[default]
    Command,
    Query,
}

impl ServiceFunc {
    /// Returns `true` if the function is declared with a `()` return type,
    /// i.e. it does not produce a value on success.
    pub fn returns_void(&self) -> bool {
        use PrimitiveType::*;
        use TypeDecl::*;
        self.output == Primitive(Void)
    }
}

/// Function parameter used in constructors and service functions.
///
/// Stores the parameter name as written in IDL and its fully resolved type
/// (`TypeDecl`), preserving declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Service event is represented as an enum variant with an associated payload.
///
/// Events in `events { ... }` are modeled as `EnumVariant` with a `StructDef`
/// describing fields of the event, so the same machinery as for enums can be reused.
pub type ServiceEvent = EnumVariant;

/// Generalized type descriptor used throughout the AST.
///
/// Covers all kinds of IDL types:
/// - primitive types (`Primitive`),
/// - slices and fixed arrays (`Slice`, `Array`),
/// - tuples (`Tuple`),
/// - named types (e.g. `Point<u32>`)
///     - container types like `Option<T>`, `Result<T, E>`
///     - user-defined types with generics (`UserDefined`),
///     - bare generic parameters (`T`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeDecl {
    /// Slice type `[T]`.
    Slice { item: Box<TypeDecl> },
    /// Fixed-length array type `[T; N]`.
    Array { item: Box<TypeDecl>, len: u32 },
    /// Tuple type `(T1, T2, ...)`, including `()` for an empty tuple.
    Tuple { types: Vec<TypeDecl> },
    /// Named type, possibly generic (e.g. `Point<u32>`).
    ///
    /// - known named type, e.g. `Option<T>`, `Result<T, E>`
    /// - user-defined named type
    /// - generic type parameter (e.g. `T`) used in type definitions.
    Named {
        name: String,
        generics: Vec<TypeDecl>,
    },
    /// Built-in primitive type from `PrimitiveType`.
    Primitive(PrimitiveType),
}

impl TypeDecl {
    pub fn named(name: String) -> TypeDecl {
        TypeDecl::Named {
            name,
            generics: vec![],
        }
    }

    pub fn tuple(types: Vec<TypeDecl>) -> TypeDecl {
        TypeDecl::Tuple { types }
    }

    pub fn option(item: TypeDecl) -> TypeDecl {
        TypeDecl::Named {
            name: "Option".to_string(),
            generics: vec![item],
        }
    }

    pub fn result(ok: TypeDecl, err: TypeDecl) -> TypeDecl {
        TypeDecl::Named {
            name: "Result".to_string(),
            generics: vec![ok, err],
        }
    }

    pub fn option_type_decl(ty: &TypeDecl) -> Option<TypeDecl> {
        match ty {
            TypeDecl::Named { name, generics } if name == "Option" => {
                if let [item] = generics.as_slice() {
                    Some(item.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn result_type_decl(ty: &TypeDecl) -> Option<(TypeDecl, TypeDecl)> {
        match ty {
            TypeDecl::Named { name, generics } if name == "Result" => {
                if let [ok, err] = generics.as_slice() {
                    Some((ok.clone(), err.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Display for TypeDecl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TypeDecl::*;
        match self {
            Slice { item } => write!(f, "[{item}]"),
            Array { item, len } => write!(f, "[{item}; {len}]"),
            Tuple { types } => {
                f.write_char('(')?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                f.write_char(')')?;
                Ok(())
            }
            Named { name, generics } => {
                write!(f, "{name}")?;
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
            Primitive(primitive_type) => write!(f, "{primitive_type}"),
        }
    }
}

/// Enumeration of all built-in primitive types supported by the IDL.
///
/// Includes booleans, characters, signed/unsigned integers, string, and
/// platform-specific identifiers and hashes (ActorId, CodeId, MessageId,
/// H160/H256/U256) used by the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PrimitiveType {
    /// Unit / void type `()`.
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
    /// Identifier of an actor / service instance.
    ActorId,
    /// Identifier of deployed code.
    CodeId,
    /// Identifier of a message.
    MessageId,
    /// 160-bit hash / address type.
    H160,
    /// 256-bit hash (32-byte array) type.
    H256,
    /// 256-bit unsigned integer type.
    U256,
}

impl PrimitiveType {
    /// Returns the canonical textual representation used when rendering IDL.
    pub fn as_str(&self) -> &'static str {
        use PrimitiveType::*;
        match self {
            Void => "()",
            Bool => "bool",
            Char => "char",
            String => "String",
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
            H160 => "H160",
            H256 => "H256",
            U256 => "U256",
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

    /// Parses a primitive type from its textual representation used in IDL.
    ///
    /// Accepts several common aliases and case variations (e.g. `string` / `String`,
    /// `actor_id` / `ActorId`) for convenience.
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

            "ActorId" | "actor" | "actor_id" => Ok(ActorId),
            "CodeId" | "code" | "code_id" => Ok(CodeId),
            "MessageId" | "messageid" | "message_id" => Ok(MessageId),

            "H256" | "h256" => Ok(H256),
            "U256" | "u256" => Ok(U256),
            "H160" | "h160" => Ok(H160),

            other => Err(format!("Unknown primitive type: {other}")),
        }
    }
}

/// Top-level named type definition inside `types { ... }` of a service or program.
///
/// `Type` describes either a struct or enum with an optional list of generic
/// type parameters, along with documentation and annotations taken from IDL.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "templates",
    derive(askama::Template),
    template(path = "type.askama", escape = "none")
)]
pub struct Type {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub def: TypeDef,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// Generic type parameter in a type definition.
///
/// - `name` is the declared identifier of the parameter (e.g. `T`);
/// - `ty` is an optional concrete type bound / substitution; `None` means that
///   the parameter is left generic at this level.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
        let TypeParameter { name, ty: _ } = self;
        write!(f, "{name}")
    }
}

/// Underlying definition of a named type: either a struct or an enum.
///
/// This mirrors the two composite categories in the IDL:
/// - `Struct` - record / tuple / unit structs;
/// - `Enum` - tagged unions with variants that may carry payloads.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

/// Struct definition backing a named type or an enum variant payload.
///
/// A struct can represent:
/// - unit form (`fields.is_empty()`),
/// - classic form with named fields,
/// - tuple-like form with unnamed fields.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "templates",
    derive(askama::Template),
    template(path = "struct_def.askama", escape = "none")
)]
pub struct StructDef {
    pub fields: Vec<StructField>,
}

impl StructDef {
    /// Returns `true` if the struct has no fields (unit struct).
    pub fn is_unit(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns `true` if the struct is inline and purely positional:
    /// all fields are unnamed and have no docs or annotations.
    pub fn is_inline(&self) -> bool {
        self.fields
            .iter()
            .all(|f| f.name.is_none() && f.docs.is_empty() && f.annotations.is_empty())
    }

    /// Returns `true` if the struct is tuple-like (all fields are unnamed).
    pub fn is_tuple(&self) -> bool {
        self.fields.iter().all(|f| f.name.is_none())
    }
}

/// Field of a struct or of an enum variant payload.
///
/// `name` is `None` for tuple-like structs / variants; otherwise it stores the
/// field identifier from IDL. Each field keeps its own documentation and annotations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "templates",
    derive(askama::Template),
    template(path = "field.askama", escape = "none")
)]
pub struct StructField {
    pub name: Option<String>,
    pub type_decl: TypeDecl,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// Enum definition backing a named enum type.
///
/// Stores the ordered list of `EnumVariant` items that form a tagged union.
/// Each variant may be unit-like, classic (named fields) or tuple-like.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub variants: Vec<EnumVariant>,
}

/// Single variant of an enum or service event.
///
/// - `name` is the variant identifier,
/// - `def` is a `StructDef` describing the payload shape (unit / classic / tuple),
/// - `docs` and `annotations` are attached to the variant in IDL.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "templates",
    derive(askama::Template),
    template(path = "variant.askama", escape = "none")
)]
pub struct EnumVariant {
    pub name: String,
    pub def: StructDef,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}
