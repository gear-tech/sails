use crate::helpers::*;
use convert_case::Casing;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

macro_rules! base {
    ($t: expr) => {
        concat!("global::Substrate.NetApi.Model.Types.Base.", $t)
    };
}

macro_rules! primitive {
    ($t: expr) => {
        concat!("global::Substrate.NetApi.Model.Types.Primitive.", $t)
    };
}

macro_rules! gprimitives {
    ($t: expr) => {
        concat!(
            "global::Substrate.Gear.Api.Generated.Model.gprimitives.",
            $t
        )
    };
}

macro_rules! client_base {
    ($t: expr) => {
        concat!("global::Substrate.Gear.Client.NetApi.Model.Types.Base.", $t)
    };
}

macro_rules! client_primitive {
    ($t: expr) => {
        concat!(
            "global::Substrate.Gear.Client.NetApi.Model.Types.Primitive.",
            $t
        )
    };
}

#[derive(Clone)]
pub(crate) struct TypeDeclGenerator<'a> {
    code: String,
    generated_types: &'a Vec<&'a Type>,
}

impl<'a> TypeDeclGenerator<'a> {
    pub(crate) fn new(generated_types: &'a Vec<&'a Type>) -> Self {
        Self {
            code: String::new(),
            generated_types,
        }
    }

    pub(crate) fn generate_type_decl(&mut self, type_decl: &'a TypeDecl) -> String {
        visitor::accept_type_decl(type_decl, self);
        std::mem::take(&mut self.code)
    }

    pub(crate) fn generate_struct_def(&mut self, struct_def: &'a StructDef) -> String {
        visitor::accept_struct_def(struct_def, self);
        std::mem::take(&mut self.code)
    }

    fn join_type_decls<I: IntoIterator<Item = &'a TypeDecl>>(
        &mut self,
        type_decls: I,
        separator: &str,
    ) {
        let prev_code = std::mem::take(&mut self.code);
        let mut type_decls_str: Vec<String> = Vec::new();
        let iter = type_decls.into_iter();
        for type_decl in iter {
            visitor::accept_type_decl(type_decl, self);
            type_decls_str.push(std::mem::take(&mut self.code));
        }
        _ = std::mem::replace(&mut self.code, prev_code);
        self.code.push_str(type_decls_str.join(separator).as_str());
    }

    pub(crate) fn fn_params_with_types(&mut self, params: &'a [FuncParam]) -> String {
        params
            .iter()
            .map(|p| {
                format!(
                    "{} {}",
                    self.generate_type_decl(p.type_decl()),
                    escape_keywords(p.name().to_case(convert_case::Case::Camel))
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<'a> Visitor<'a> for TypeDeclGenerator<'a> {
    fn visit_optional_type_decl(&mut self, optional_type_decl: &'a TypeDecl) {
        self.code.push_str(base!("BaseOpt<"));
        visitor::accept_type_decl(optional_type_decl, self);
        self.code.push('>');
    }

    fn visit_result_type_decl(&mut self, ok_type_decl: &'a TypeDecl, err_type_decl: &'a TypeDecl) {
        self.code.push_str(client_base!("BaseResult<"));
        visitor::accept_type_decl(ok_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(err_type_decl, self);
        self.code.push('>');
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'a TypeDecl) {
        self.code.push_str(base!("BaseVec<"));
        visitor::accept_type_decl(vector_type_decl, self);
        self.code.push('>');
    }

    fn visit_struct_def(&mut self, struct_def: &'a StructDef) {
        if struct_def.fields().is_empty() {
            return;
        }
        if struct_def.fields().len() == 1 {
            visitor::accept_type_decl(struct_def.fields()[0].type_decl(), self);
        } else {
            self.code.push_str(base!("BaseTuple<"));
            self.join_type_decls(struct_def.fields().iter().map(|f| f.type_decl()), ", ");
            self.code.push('>');
        }
    }

    fn visit_primitive_type_id(&mut self, primitive_type_id: PrimitiveType) {
        self.code
            .push_str(primitive_type_to_dotnet(primitive_type_id));
    }

    fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'a str) {
        let is_enum = self
            .generated_types
            .iter()
            .find(|&&t| t.name() == user_defined_type_id)
            .map(|&t| matches!(t.def(), TypeDef::Enum(_)))
            .unwrap_or_default();
        let type_id = if is_enum {
            &format!("Enum{user_defined_type_id}") // Enum prefix
        } else {
            user_defined_type_id
        };
        self.code.push_str(type_id);
    }

    fn visit_map_type_decl(&mut self, key_type_decl: &'a TypeDecl, value_type_decl: &'a TypeDecl) {
        self.code.push_str(client_base!("BaseDictionary<"));
        visitor::accept_type_decl(key_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(value_type_decl, self);
        self.code.push('>');
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'a TypeDecl, _len: u32) {
        visitor::accept_type_decl(item_type_decl, self);
        self.code.push_str("[]");
    }
}

pub(crate) fn primitive_type_to_dotnet(primitive_type: PrimitiveType) -> &'static str {
    match primitive_type {
        PrimitiveType::U8 => primitive!("U8"),
        PrimitiveType::U16 => primitive!("U16"),
        PrimitiveType::U32 => primitive!("U32"),
        PrimitiveType::U64 => primitive!("U64"),
        PrimitiveType::U128 => primitive!("U128"),
        PrimitiveType::I8 => primitive!("I8"),
        PrimitiveType::I16 => primitive!("I16"),
        PrimitiveType::I32 => primitive!("I32"),
        PrimitiveType::I64 => primitive!("I64"),
        PrimitiveType::I128 => primitive!("I128"),
        PrimitiveType::Bool => primitive!("Bool"),
        PrimitiveType::Str => primitive!("Str"),
        PrimitiveType::Char => primitive!("PrimChar"),
        PrimitiveType::Null => base!("BaseVoid"),
        PrimitiveType::ActorId => gprimitives!("ActorId"),
        PrimitiveType::CodeId => gprimitives!("CodeId"),
        PrimitiveType::MessageId => gprimitives!("MessageId"),
        PrimitiveType::H160 => client_primitive!("H160"),
        PrimitiveType::H256 => "global::Substrate.Gear.Api.Generated.Model.primitive_types.H256",
        PrimitiveType::U256 => primitive!("U256"),
        PrimitiveType::NonZeroU8 => client_primitive!("NonZeroU8"),
        PrimitiveType::NonZeroU16 => client_primitive!("NonZeroU16"),
        PrimitiveType::NonZeroU32 => "global::Substrate.Gear.Api.Generated.Types.Base.NonZeroU32",
        PrimitiveType::NonZeroU64 => client_primitive!("NonZeroU64"),
        PrimitiveType::NonZeroU128 => client_primitive!("NonZeroU128"),
        PrimitiveType::NonZeroU256 => client_primitive!("NonZeroU256"),
    }
}
