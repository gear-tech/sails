use crate::visitor::Visitor;

#[derive(Debug, PartialEq)]
pub struct Program {
    service: Service,
    types: Vec<Type>,
}

impl Program {
    pub(crate) fn new(service: Service, types: Vec<Type>) -> Self {
        Self { service, types }
    }

    pub fn service(&self) -> &Service {
        &self.service
    }

    pub fn types(&self) -> &[Type] {
        &self.types
    }

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        visitor.visit_service(&self.service);
        for r#type in &self.types {
            visitor.visit_type(r#type);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Service {
    funcs: Vec<Func>,
}

impl Service {
    pub(crate) fn new(funcs: Vec<Func>) -> Self {
        Self { funcs }
    }

    pub fn funcs(&self) -> &[Func] {
        &self.funcs
    }

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        for func in &self.funcs {
            visitor.visit_func(func);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Func {
    name: String,
    params: Vec<Param>,
    output: TypeDecl,
    is_query: bool,
}

impl Func {
    pub(crate) fn new(name: String, params: Vec<Param>, output: TypeDecl, is_query: bool) -> Self {
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

    pub fn params(&self) -> &[Param] {
        &self.params
    }

    pub fn output(&self) -> &TypeDecl {
        &self.output
    }

    pub fn is_query(&self) -> bool {
        self.is_query
    }

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        for param in &self.params {
            visitor.visit_param(param);
        }
        visitor.visit_output(&self.output);
    }
}

#[derive(Debug, PartialEq)]
pub struct Param {
    name: String,
    r#type: TypeDecl,
}

impl Param {
    pub(crate) fn new(name: String, r#type: TypeDecl) -> Self {
        Self { name, r#type }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn r#type(&self) -> &TypeDecl {
        &self.r#type
    }

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        self.r#type.accept(visitor);
    }
}

#[derive(Debug, PartialEq)]
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

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        self.def.accept(visitor);
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeDecl {
    Opt(Box<TypeDecl>),
    Vec(Box<TypeDecl>),
    Result {
        ok: Box<TypeDecl>,
        err: Box<TypeDecl>,
    },
    Id(TypeId),
    Def(TypeDef),
}

impl TypeDecl {
    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        match self {
            TypeDecl::Opt(type_decl) => {
                visitor.visit_optional_type_decl(type_decl);
            }
            TypeDecl::Vec(type_decl) => {
                visitor.visit_vector_type_decl(type_decl);
            }
            TypeDecl::Result { ok, err } => {
                visitor.visit_result_type_decl(ok, err);
            }
            TypeDecl::Id(type_id) => match type_id {
                TypeId::Primitive(primitive_type_id) => {
                    visitor.visit_primitive_type_id(primitive_type_id);
                }
                TypeId::UserDefined(user_defined_type_id) => {
                    visitor.visit_user_defined_type_id(user_defined_type_id);
                }
            },
            TypeDecl::Def(type_def) => {
                type_def.accept(visitor);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeId {
    Primitive(PrimitiveType),
    UserDefined(String),
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

impl TypeDef {
    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        match self {
            TypeDef::Struct(struct_def) => {
                visitor.visit_struct_def(struct_def);
            }
            TypeDef::Enum(enum_def) => {
                visitor.visit_enum_def(enum_def);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
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

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        for field in &self.fields {
            visitor.visit_struct_field(field);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StructField {
    name: Option<String>,
    r#type: TypeDecl,
}

impl StructField {
    pub(crate) fn new(name: Option<String>, r#type: TypeDecl) -> Self {
        Self { name, r#type }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn r#type(&self) -> &TypeDecl {
        &self.r#type
    }

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        self.r#type.accept(visitor);
    }
}

#[derive(Debug, PartialEq)]
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

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        for variant in &self.variants {
            visitor.visit_enum_variant(variant);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EnumVariant {
    name: String,
    r#type: Option<TypeDecl>,
}

impl EnumVariant {
    pub(crate) fn new(name: String, r#type: Option<TypeDecl>) -> Self {
        Self { name, r#type }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn r#type(&self) -> Option<&TypeDecl> {
        self.r#type.as_ref()
    }

    pub fn accept<'ast>(&'ast self, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
        self.r#type.as_ref().map(|r#type| r#type.accept(visitor));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    struct ProgramVisitor<'ast> {
        service: Option<&'ast Service>,
        types: Vec<&'ast Type>,
    }

    impl<'ast> Visitor<'ast> for ProgramVisitor<'ast> {
        fn visit_service(&mut self, service: &'ast Service) {
            self.service = Some(service);
        }

        fn visit_type(&mut self, r#type: &'ast Type) {
            self.types.push(r#type);
        }
    }

    #[test]
    fn program_accept_works() {
        let program = Program::new(
            Service::new(vec![]),
            vec![
                Type::new("Type1".into(), TypeDef::Struct(StructDef::new(vec![]))),
                Type::new("Type2".into(), TypeDef::Enum(EnumDef::new(vec![]))),
            ],
        );
        let mut program_visitor = ProgramVisitor {
            service: None,
            types: vec![],
        };

        program.accept(&mut program_visitor);

        assert_eq!(program_visitor.service, Some(program.service()));
        assert!(itertools::equal(program_visitor.types, program.types()));
    }
}
