use crate::builder::TypeBuilder;
use crate::registry::TypeRef;
use alloc::string::String;
use alloc::vec::Vec;

/// Declared generic parameter together with the argument assigned to it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeParameter {
    /// Declared parameter name as it should appear in rendered output.
    pub name: String,
    /// Argument assigned to the parameter.
    pub arg: GenericArg,
}

/// Type-level argument assigned to a generic parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GenericArg {
    /// Type argument represented by a [`TypeRef`].
    Type(TypeRef),
    /// Const argument stored as its rendered source form.
    Const(String),
}

/// Free-form annotation attached to a type, field, or variant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Annotation {
    /// Annotation name.
    pub name: String,
    /// Optional annotation value.
    pub value: Option<String>,
}

/// Portable description of a Rust type stored in a registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    /// Module path recorded for display and disambiguation.
    pub module_path: String,
    /// Human-readable type name.
    pub name: String,
    /// Generic parameters declared by this type entry.
    pub type_params: Vec<TypeParameter>,
    /// Structural definition of the type.
    pub def: TypeDef,
    /// Documentation lines captured from source.
    pub docs: Vec<String>,
    /// Additional annotations attached to the type.
    pub annotations: Vec<Annotation>,
}

impl Type {
    /// Starts building a synthetic [`Type`] value.
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

/// Structural definition of a recorded type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDef {
    /// Primitive scalar type.
    Primitive(Primitive),
    /// Gear-specific primitive type.
    #[cfg(feature = "gprimitives")]
    GPrimitive(GPrimitive),
    /// Struct-like product type.
    Composite(Composite),
    /// Enum-like sum type.
    Variant(VariantDef),
    /// Variable-length homogeneous sequence.
    Sequence(TypeRef),
    /// Fixed-size homogeneous array.
    Array { len: u32, type_param: TypeRef },
    /// Tuple of positional elements.
    Tuple(Vec<TypeRef>),
    /// Key-value mapping.
    Map { key: TypeRef, value: TypeRef },
    /// Optional value.
    Option(TypeRef),
    /// Result-like success or error value.
    Result { ok: TypeRef, err: TypeRef },
    /// Generic type parameter placeholder such as `T`.
    Parameter(String),
    /// Application of a generic base type to concrete type arguments.
    Applied { base: TypeRef, args: Vec<TypeRef> },
}

/// Struct-like collection of fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composite {
    /// Fields in declaration order.
    pub fields: Vec<Field>,
}

/// Enum-like collection of variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDef {
    /// Variants in declaration order.
    pub variants: Vec<Variant>,
}

/// Individual variant inside a [`VariantDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// Variant name.
    pub name: String,
    /// Variant payload fields.
    pub fields: Vec<Field>,
    /// Documentation lines captured from source.
    pub docs: Vec<String>,
    /// Additional annotations attached to the variant.
    pub annotations: Vec<Annotation>,
}

/// Named or unnamed field inside a composite or variant definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Field name, or `None` for tuple-style fields.
    pub name: Option<String>,
    /// Referenced field type.
    pub ty: TypeRef,
    /// Documentation lines captured from source.
    pub docs: Vec<String>,
    /// Additional annotations attached to the field.
    pub annotations: Vec<Annotation>,
}

/// Built-in primitive types understood by the registry.
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

/// Gear-specific primitive types understood by the registry.
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
    /// Creates an annotation without an attached value.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }
}
