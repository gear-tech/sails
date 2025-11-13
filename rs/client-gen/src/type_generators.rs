use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{ast::visitor, ast::visitor::Visitor, ast::*};


pub(crate) struct TopLevelTypeGenerator<'a> {
    type_name: &'a str,
    sails_path: &'a str,
    derive_traits: &'a str,
    tokens: Tokens,
}

impl<'a> TopLevelTypeGenerator<'a> {
    pub(crate) fn new(type_name: &'a str, sails_path: &'a str, no_derive_traits: bool) -> Self {
        let derive_traits = if no_derive_traits {
            "Encode, Decode, TypeInfo"
        } else {
            "PartialEq, Clone, Debug, Encode, Decode, TypeInfo"
        };
        Self {
            type_name,
            sails_path,
            derive_traits,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for TopLevelTypeGenerator<'_> {
    fn visit_type(&mut self, r#type: &'ast Type) {
        for doc in &r#type.docs {
            quote_in! { self.tokens =>
                $['\r'] $("///") $doc
            };
        }
        visitor::accept_type(r#type, self);
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let mut struct_def_generator =
            StructDefGenerator::new(self.type_name, self.sails_path, self.derive_traits);
        struct_def_generator.visit_struct_def(struct_def);
        self.tokens.extend(struct_def_generator.finalize());
    }

    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        let mut enum_def_generator =
            EnumDefGenerator::new(self.type_name, self.sails_path, self.derive_traits);
        enum_def_generator.visit_enum_def(enum_def);
        self.tokens.extend(enum_def_generator.finalize());
    }
}

#[derive(Default)]
struct StructDefGenerator<'a> {
    type_name: &'a str,
    sails_path: &'a str,
    derive_traits: &'a str,
    is_tuple_struct: bool,
    tokens: Tokens,
}

impl<'a> StructDefGenerator<'a> {
    fn new(type_name: &'a str, sails_path: &'a str, derive_traits: &'a str) -> Self {
        Self {
            type_name,
            sails_path,
            derive_traits,
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
            pub struct $(self.type_name) $prefix $(self.tokens) $postfix
        }
    }
}

impl<'ast> Visitor<'ast> for StructDefGenerator<'ast> {
    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let is_regular_struct = struct_def.fields.iter().all(|f| f.name.is_some());
        let is_tuple_struct = struct_def.fields.iter().all(|f| f.name.is_none());
        if !is_regular_struct && !is_tuple_struct {
            panic!("Struct must be either regular or tuple");
        }
        self.is_tuple_struct = is_tuple_struct;
        visitor::accept_struct_def(struct_def, self);
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        let type_decl_code = generate_type_decl_with_path(&struct_field.type_decl, "".into());

        for doc in &struct_field.docs {
            quote_in! { self.tokens =>
                $['\r'] $("///") $doc
            };
        }

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
    tokens: Tokens,
}

impl<'a> EnumDefGenerator<'a> {
    pub(crate) fn new(type_name: &'a str, sails_path: &'a str, derive_traits: &'a str) -> Self {
        Self {
            type_name,
            sails_path,
            derive_traits,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote!(
            $['\r']
            #[derive($(self.derive_traits))]
            #[codec(crate = $(self.sails_path)::scale_codec)]
            #[scale_info(crate = $(self.sails_path)::scale_info)]
            pub enum $(self.type_name) { $(self.tokens) }
        )
    }
}

impl<'ast> Visitor<'ast> for EnumDefGenerator<'ast> {
    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        visitor::accept_enum_def(enum_def, self);
    }

    fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
        for doc in &enum_variant.docs {
            quote_in! { self.tokens =>
                $['\r'] $("///") $doc
            };
        }

        let variant_name = &enum_variant.name;

        if enum_variant.def.fields.is_empty() {
            // Unit variant: `Variant,`
            quote_in! { self.tokens =>
                $['\r'] $variant_name,
            };
            return;
        }

        let is_tuple = enum_variant.def.fields.iter().all(|f| f.name.is_none());
        let is_struct = enum_variant.def.fields.iter().all(|f| f.name.is_some());

        if !is_tuple && !is_struct {
            panic!(
                "Enum variant '{}' has a mix of named and unnamed fields, which is not supported.",
                variant_name
            );
        }

        if is_tuple {
            // Tuple variant: `Variant(Type1, Type2),`
            let mut field_tokens = Tokens::new();
            for (i, field) in enum_variant.def.fields.iter().enumerate() {
                if i > 0 {
                    field_tokens.append(", ");
                }
                let type_code =
                    generate_type_decl_with_path(&field.type_decl, self.sails_path.into());
                field_tokens.append(type_code);
            }
            quote_in! { self.tokens =>
                $['\r'] $variant_name($field_tokens),
            };
        } else {
            // Struct variant: `Variant { field1: Type1, ... },`
            let mut field_tokens = Tokens::new();
            for field in &enum_variant.def.fields {
                for doc in &field.docs {
                    quote_in! { field_tokens =>
                        $['\r'] $("///") $doc
                    };
                }
                let field_name = field.name.as_ref().unwrap();
                let type_code =
                    generate_type_decl_with_path(&field.type_decl, self.sails_path.into());
                quote_in! { field_tokens =>
                    $['\r'] pub $field_name: $type_code,
                };
            }
            quote_in! { self.tokens =>
                $['\r'] $variant_name {
                    $(field_tokens)
                $['\r'] },
            };
        }
    }
}pub(crate) fn generate_type_decl_with_path(type_decl: &TypeDecl, path: String) -> String {
    let mut type_decl_generator = TypeDeclGenerator {
        tokens: Tokens::new(),
        path,
    };
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator
        .tokens
        .to_string()
        .expect("Failed to generate type decl code")
}

// (The rest of the file remains the same for now)
// ...

/*
 * Commented out old implementation
...
*/

#[derive(Default)]
struct TypeDeclGenerator {
    tokens: Tokens,
    path: String,
}

impl<'ast> Visitor<'ast> for TypeDeclGenerator {
    fn visit_slice_type_decl(&mut self, item_type_decl: &'ast TypeDecl) {
        self.tokens.append("Vec<");
        visitor::accept_type_decl(item_type_decl, self);
        self.tokens.append(">");
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast TypeDecl, len: u32) {
        self.tokens.append("[");
        visitor::accept_type_decl(item_type_decl, self);
        self.tokens.append(format!("; {len}]"));
    }

    fn visit_tuple_type_decl(&mut self, items: &'ast Vec<TypeDecl>) {
        self.tokens.append("(");
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                self.tokens.append(", ");
            }
            visitor::accept_type_decl(item, self);
        }
        if items.len() == 1 {
            self.tokens.append(",");
        }
        self.tokens.append(")");
    }

    fn visit_option_type_decl(&mut self, inner_type_decl: &'ast TypeDecl) {
        self.tokens.append("Option<");
        visitor::accept_type_decl(inner_type_decl, self);
        self.tokens.append(">");
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        self.tokens.append("Result<");
        visitor::accept_type_decl(ok_type_decl, self);
        self.tokens.append(", ");
        visitor::accept_type_decl(err_type_decl, self);
        self.tokens.append(">");
    }

    fn visit_primitive_type(&mut self, primitive_type: PrimitiveType) {
        self.tokens.append(match primitive_type {
            PrimitiveType::Void => "()",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Char => "char",
            PrimitiveType::String => "String",
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
            PrimitiveType::ActorId => "ActorId",
            PrimitiveType::CodeId => "CodeId",
            PrimitiveType::MessageId => "MessageId",
            PrimitiveType::H160 => "H160",
            PrimitiveType::H256 => "H256",
            PrimitiveType::U256 => "U256",
        });
    }

    fn visit_user_defined_type(
        &mut self,
        path: &'ast str,
        generics: &'ast Vec<TypeDecl>,
    ) {
        if !self.path.is_empty() {
            self.tokens.append(self.path.as_str());
            self.tokens.append("::");
        }
        self.tokens.append(path);
        if !generics.is_empty() {
            self.tokens.append("<");
            for (i, generic) in generics.iter().enumerate() {
                if i > 0 {
                    self.tokens.append(", ");
                }
                visitor::accept_type_decl(generic, self);
            }
            self.tokens.append(">");
        }
    }
}

