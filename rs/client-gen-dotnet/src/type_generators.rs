use crate::helpers::summary_comment;
use convert_case::{Case, Casing};
use csharp::Tokens;
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) fn generate_type_decl_code(type_decl: &TypeDecl) -> String {
    let mut type_decl_generator = TypeDeclGenerator::default();
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) fn generate_type_decl_with_path(type_decl: &TypeDecl, path: String) -> String {
    let mut type_decl_generator = TypeDeclGenerator {
        code: String::new(),
        path,
    };
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) struct TopLevelTypeGenerator<'a> {
    type_name: &'a str,
    tokens: Tokens,
}

impl<'a> TopLevelTypeGenerator<'a> {
    pub(crate) fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'a, 'ast> Visitor<'ast> for TopLevelTypeGenerator<'a> {
    fn visit_type(&mut self, r#type: &'ast Type) {
        quote_in! { self.tokens =>
            $['\r']
            $(summary_comment(r#type.docs()))
        };
        visitor::accept_type(r#type, self);
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let mut struct_def_generator = StructDefGenerator::new(self.type_name);
        struct_def_generator.visit_struct_def(struct_def);
        self.tokens.extend(struct_def_generator.finalize());
    }

    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        let mut enum_def_generator = EnumDefGenerator::new(self.type_name);
        enum_def_generator.visit_enum_def(enum_def);
        self.tokens.extend(enum_def_generator.finalize());
    }
}

#[derive(Default)]
struct StructDefGenerator<'a> {
    type_name: &'a str,
    is_tuple_struct: bool,
    tokens: Tokens,
}

impl<'a> StructDefGenerator<'a> {
    fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
            tokens: Tokens::new(),
            is_tuple_struct: false,
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote! {
            $['\r']
            public partial record struct $(self.type_name)
            (
                $(self.tokens)
            )
        }
    }
}

impl<'ast> Visitor<'ast> for StructDefGenerator<'ast> {
    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let is_regular_struct = struct_def.fields().iter().all(|f| f.name().is_some());
        let is_tuple_struct = struct_def.fields().iter().all(|f| f.name().is_none());
        if !is_regular_struct && !is_tuple_struct {
            panic!("Struct must be either regular or tuple");
        }
        self.is_tuple_struct = is_tuple_struct;

        visitor::accept_struct_def(struct_def, self);
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        let type_decl_code = generate_type_decl_with_path(struct_field.type_decl(), "".into());

        quote_in! { self.tokens =>
            $['\r']
            $(csharp::block_comment(struct_field.docs()))
        };

        if let Some(field_name) = struct_field.name() {
            quote_in! { self.tokens =>
                $['\r'] $type_decl_code $(field_name.to_case(Case::Pascal)),
            };
        } else {
            quote_in! { self.tokens =>
                $['\r'] $type_decl_code,
            };
        }
    }
}

#[derive(Default)]
struct EnumDefGenerator<'a> {
    type_name: &'a str,
    enum_tokens: Tokens,
    class_tokens: Tokens,
}

impl<'a> EnumDefGenerator<'a> {
    pub(crate) fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
            enum_tokens: Tokens::new(),
            class_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let class_name = format!("Enum{}", self.type_name);
        quote!(
            $['\r']
            public enum $(self.type_name)
            {
                $(self.enum_tokens)
            }

            $['\r']
            public sealed class $(&class_name) : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<$(self.type_name)>
            {
                $['\r']
                public $(&class_name)()
                $['\r']
                {
                    $['\r']
                    $(self.class_tokens)
                    $['\r']
                }
            }
        )
    }
}

impl<'ast> Visitor<'ast> for EnumDefGenerator<'ast> {
    fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
        quote_in! { self.enum_tokens =>
            $['\r'] $(summary_comment(enum_variant.docs()))
        };
        quote_in! { self.enum_tokens =>
            $['\r'] $(enum_variant.name()),
        };

        quote_in! { self.class_tokens =>
            AddTypeDecoder<>($(self.type_name).$(enum_variant.name()));
        }
    }
}

#[derive(Default)]
struct TypeDeclGenerator {
    code: String,
    path: String,
}

impl<'ast> Visitor<'ast> for TypeDeclGenerator {
    fn visit_optional_type_decl(&mut self, optional_type_decl: &'ast TypeDecl) {
        self.code.push_str("Option<");
        visitor::accept_type_decl(optional_type_decl, self);
        self.code.push('>');
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        self.code.push_str("Result<");
        visitor::accept_type_decl(ok_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(err_type_decl, self);
        self.code.push('>');
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'ast TypeDecl) {
        self.code.push_str("Vec<");
        visitor::accept_type_decl(vector_type_decl, self);
        self.code.push('>');
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let mut struct_def_generator = StructTypeGenerator::new(self.path.clone());
        struct_def_generator.visit_struct_def(struct_def);
        self.code.push_str(&struct_def_generator.code);
    }

    fn visit_primitive_type_id(&mut self, primitive_type_id: PrimitiveType) {
        self.code.push_str(match primitive_type_id {
            PrimitiveType::U8 => "u8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::U64 => "u64",
            PrimitiveType::U128 => "u128",
            PrimitiveType::I8 => "i8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::I128 => "i128",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Str => "String",
            PrimitiveType::Char => "char",
            PrimitiveType::Null => "()",
            PrimitiveType::ActorId => "ActorId",
            PrimitiveType::CodeId => "CodeId",
            PrimitiveType::MessageId => "MessageId",
            PrimitiveType::H160 => "H160",
            PrimitiveType::H256 => "H256",
            PrimitiveType::U256 => "U256",
            PrimitiveType::NonZeroU8 => "NonZeroU8",
            PrimitiveType::NonZeroU16 => "NonZeroU16",
            PrimitiveType::NonZeroU32 => "NonZeroU32",
            PrimitiveType::NonZeroU64 => "NonZeroU64",
            PrimitiveType::NonZeroU128 => "NonZeroU128",
            PrimitiveType::NonZeroU256 => "NonZeroU256",
        });
    }

    fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'ast str) {
        if !self.path.is_empty() {
            self.code.push_str(&self.path);
            self.code.push_str("::");
        }
        self.code.push_str(user_defined_type_id);
    }

    fn visit_map_type_decl(
        &mut self,
        key_type_decl: &'ast TypeDecl,
        value_type_decl: &'ast TypeDecl,
    ) {
        self.code.push_str("BTreeMap<");
        visitor::accept_type_decl(key_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(value_type_decl, self);
        self.code.push('>');
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast TypeDecl, len: u32) {
        self.code.push('[');
        visitor::accept_type_decl(item_type_decl, self);
        self.code.push_str(&format!("; {len}]"));
    }
}

struct StructTypeGenerator {
    code: String,
    path: String,
}

impl StructTypeGenerator {
    fn new(path: String) -> Self {
        Self {
            code: String::new(),
            path,
        }
    }
}

impl<'ast> Visitor<'ast> for StructTypeGenerator {
    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let is_regular_struct = struct_def.fields().iter().all(|f| f.name().is_some());
        let is_tuple_struct = struct_def.fields().iter().all(|f| f.name().is_none());
        if !is_regular_struct && !is_tuple_struct {
            panic!("Struct must be either regular or tuple");
        }
        if is_regular_struct {
            self.code.push('{');
        } else {
            self.code.push('(');
        }
        visitor::accept_struct_def(struct_def, self);
        if is_regular_struct {
            self.code.push('}');
        } else {
            self.code.push(')');
        }
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        let type_decl_code =
            generate_type_decl_with_path(struct_field.type_decl(), self.path.clone());

        if let Some(field_name) = struct_field.name() {
            self.code
                .push_str(&format!("{field_name}: {type_decl_code},"));
        } else {
            self.code.push_str(&format!("{type_decl_code},"));
        }
    }
}
