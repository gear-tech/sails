use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{ast, visitor, visitor::Visitor};

use crate::helpers::generate_doc_comments;

pub(crate) struct TopLevelTypeGenerator<'ast> {
    type_name: &'ast str,
    sails_path: &'ast str,
    derive_traits: &'ast str,
    type_params_tokens: Tokens,
    tokens: Tokens,
}

impl<'ast> TopLevelTypeGenerator<'ast> {
    pub(crate) fn new(type_name: &'ast str, sails_path: &'ast str, no_derive_traits: bool) -> Self {
        let derive_traits = if no_derive_traits {
            "Encode, Decode, TypeInfo, ReflectHash"
        } else {
            "PartialEq, Clone, Debug, Encode, Decode, TypeInfo, ReflectHash"
        };
        Self {
            type_name,
            sails_path,
            derive_traits,
            type_params_tokens: Tokens::new(),
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for TopLevelTypeGenerator<'ast> {
    fn visit_type(&mut self, r#type: &'ast ast::Type) {
        generate_doc_comments(&mut self.tokens, &r#type.docs);

        if !r#type.type_params.is_empty() {
            self.type_params_tokens.append("<");
            for (i, type_param) in r#type.type_params.iter().enumerate() {
                if i > 0 {
                    self.type_params_tokens.append(", ");
                }
                self.type_params_tokens.append(&type_param.name);
            }
            self.type_params_tokens.append(">");
        }

        visitor::accept_type(r#type, self);
    }

    fn visit_type_def(&mut self, type_def: &'ast ast::TypeDef) {
        match type_def {
            ast::TypeDef::Struct(struct_def) => {
                let mut struct_def_generator = StructDefGenerator::new(
                    self.type_name,
                    self.sails_path,
                    self.derive_traits,
                    self.type_params_tokens.clone(),
                );
                struct_def_generator.visit_struct_def(struct_def);
                self.tokens.extend(struct_def_generator.finalize());
            }
            ast::TypeDef::Enum(enum_def) => {
                let mut enum_def_generator = EnumDefGenerator::new(
                    self.type_name,
                    self.sails_path,
                    self.derive_traits,
                    self.type_params_tokens.clone(),
                );
                enum_def_generator.visit_enum_def(enum_def);
                self.tokens.extend(enum_def_generator.finalize());
            }
            ast::TypeDef::Alias(alias_def) => {
                let target_type_code = generate_type_decl_with_path(&alias_def.target, "");
                quote_in! { self.tokens =>
                    $['\r']
                    pub type $(self.type_name) $(self.type_params_tokens.clone()) = $target_type_code;
                };
            }
        }
    }
}

#[derive(Default)]
struct StructDefGenerator<'a> {
    type_name: &'a str,
    sails_path: &'a str,
    derive_traits: &'a str,
    type_params_tokens: Tokens,
    is_tuple_struct: bool,
    tokens: Tokens,
}

impl<'a> StructDefGenerator<'a> {
    fn new(
        type_name: &'a str,
        sails_path: &'a str,
        derive_traits: &'a str,
        type_params_tokens: Tokens,
    ) -> Self {
        Self {
            type_name,
            sails_path,
            derive_traits,
            type_params_tokens,
            is_tuple_struct: false,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let prefix = if self.is_tuple_struct { "(" } else { "{" };
        let postfix = if self.is_tuple_struct { ");" } else { "}" };
        quote! {
            $['\r']
            #[derive($(self.derive_traits))]
            #[codec(crate = $(self.sails_path)::scale_codec)]
            #[scale_info(crate = $(self.sails_path)::scale_info)]
            #[reflect_hash(crate = $(self.sails_path))]
            pub struct $(self.type_name) $(self.type_params_tokens) $prefix $(self.tokens) $postfix
        }
    }
}

impl<'ast> Visitor<'ast> for StructDefGenerator<'ast> {
    fn visit_struct_def(&mut self, struct_def: &'ast ast::StructDef) {
        self.is_tuple_struct = struct_def.is_tuple();
        visitor::accept_struct_def(struct_def, self);
    }

    fn visit_struct_field(&mut self, struct_field: &'ast ast::StructField) {
        let type_decl_code = generate_type_decl_with_path(&struct_field.type_decl, "");

        generate_doc_comments(&mut self.tokens, &struct_field.docs);

        if let Some(field_name) = &struct_field.name {
            quote_in! { self.tokens =>
                $['\r'] pub $field_name: $type_decl_code,
            };
        } else {
            quote_in! { self.tokens =>
                $['\r'] pub $type_decl_code,
            };
        }
    }
}

#[derive(Default)]
struct EnumDefGenerator<'a> {
    type_name: &'a str,
    sails_path: &'a str,
    derive_traits: &'a str,
    type_params_tokens: Tokens,
    tokens: Tokens,
}

impl<'a> EnumDefGenerator<'a> {
    pub(crate) fn new(
        type_name: &'a str,
        sails_path: &'a str,
        derive_traits: &'a str,
        type_params_tokens: Tokens,
    ) -> Self {
        Self {
            type_name,
            sails_path,
            derive_traits,
            type_params_tokens,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote!(
            $['\r']
            #[derive($(self.derive_traits))]
            #[codec(crate = $(self.sails_path)::scale_codec)]
            #[scale_info(crate = $(self.sails_path)::scale_info)]
            #[reflect_hash(crate = $(self.sails_path))]
            pub enum $(self.type_name) $(self.type_params_tokens) { $(self.tokens) }
        )
    }
}

impl<'ast> Visitor<'ast> for EnumDefGenerator<'ast> {
    fn visit_enum_variant(&mut self, enum_variant: &'ast ast::EnumVariant) {
        generate_doc_comments(&mut self.tokens, &enum_variant.docs);

        let variant_name = &enum_variant.name;

        if enum_variant.def.is_unit() {
            // Unit variant: `Variant,`
            quote_in! { self.tokens =>
                $['\r'] $variant_name,
            };
        } else if enum_variant.def.is_tuple() {
            // Tuple variant: `Variant(Type1, Type2),`
            let mut field_tokens = Tokens::new();
            for (i, field) in enum_variant.def.fields.iter().enumerate() {
                if i > 0 {
                    field_tokens.append(", ");
                }
                let type_code = generate_type_decl_with_path(&field.type_decl, "");
                field_tokens.append(type_code);
            }
            quote_in! { self.tokens =>
                $['\r'] $variant_name($field_tokens),
            };
        } else {
            // Struct variant: `Variant { field1: Type1, ... },`
            let mut field_tokens = Tokens::new();
            for field in &enum_variant.def.fields {
                generate_doc_comments(&mut field_tokens, &field.docs);
                let field_name = field.name.as_ref().unwrap();
                let type_code = generate_type_decl_with_path(&field.type_decl, "");
                quote_in! { field_tokens =>
                    $['\r'] $field_name: $type_code,
                };
            }
            quote_in! { self.tokens =>
                $['\r'] $variant_name {
                    $(field_tokens)
                $['\r'] },
            };
        }
    }
}

pub(crate) fn generate_type_decl_with_path<'ast>(
    type_decl: &'ast ast::TypeDecl,
    path: &'ast str,
) -> String {
    let mut type_decl_generator = TypeDeclGenerator::new(path);
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator
        .tokens
        .to_string()
        .expect("Failed to generate type decl code")
}

struct TypeDeclGenerator<'ast> {
    tokens: Tokens,
    path: &'ast str,
}

impl<'ast> TypeDeclGenerator<'ast> {
    fn new(path: &'ast str) -> Self {
        Self {
            tokens: Tokens::new(),
            path,
        }
    }
}

impl<'ast> Visitor<'ast> for TypeDeclGenerator<'ast> {
    fn visit_slice_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl) {
        self.tokens.append("Vec<");
        visitor::accept_type_decl(item_type_decl, self);
        self.tokens.append(">");
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl, len: u32) {
        self.tokens.append("[");
        visitor::accept_type_decl(item_type_decl, self);
        self.tokens.append(format!("; {len}]"));
    }

    fn visit_tuple_type_decl(&mut self, items: &'ast [ast::TypeDecl]) {
        self.tokens.append("(");
        for item in items {
            visitor::accept_type_decl(item, self);
            self.tokens.append(", ");
        }
        self.tokens.append(")");
    }

    fn visit_primitive_type(&mut self, primitive_type: ast::PrimitiveType) {
        self.tokens.append(primitive_type.as_str());
    }

    fn visit_named_type_decl(&mut self, name: &'ast str, generics: &'ast [ast::TypeDecl]) {
        if !self.path.is_empty() {
            self.tokens.append(self.path);
            self.tokens.append("::");
        }
        self.tokens.append(name);
        if !generics.is_empty() {
            self.tokens.append("<");
            for generic in generics {
                visitor::accept_type_decl(generic, self);
                self.tokens.append(", ");
            }
            self.tokens.append(">");
        }
    }
}
