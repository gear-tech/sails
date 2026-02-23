use crate::{
    helpers::{doc_tokens, push_doc},
    naming::escape_ident,
};
use genco::prelude::*;
use js::Tokens;
use sails_idl_parser_v2::ast;

pub(crate) struct TypeGenerator;

impl TypeGenerator {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn render_all(&self, tokens: &mut Tokens, types: &[ast::Type]) {
        for ty in types {
            self.render_type(tokens, ty);
        }
    }

    pub(crate) fn render_type(&self, tokens: &mut Tokens, ty: &ast::Type) {
        let name = &ty.name;
        if name == "NonZeroU8"
            || name == "NonZeroU16"
            || name == "NonZeroU32"
            || name == "NonZeroU64"
            || name == "NonZeroU128"
            || name == "NonZeroU256"
        {
            return;
        }

        push_doc(tokens, &ty.docs);
        match &ty.def {
            ast::TypeDef::Struct(def) => {
                tokens.append(self.render_struct(&ty.name, &ty.type_params, def))
            }
            ast::TypeDef::Enum(def) => {
                tokens.append(self.render_enum(&ty.name, &ty.type_params, def))
            }
        }
        tokens.line();
    }

    fn render_struct(
        &self,
        name: &str,
        type_params: &[ast::TypeParameter],
        def: &ast::StructDef,
    ) -> Tokens {
        let generics = render_type_params(type_params);
        let payload = self.ts_struct_def_tokens(def);
        if def.is_unit() || def.is_tuple() {
            quote! {
                export type $(name)$(generics) = $payload;
            }
        } else {
            quote! {
                export interface $(name)$(generics) $payload
            }
        }
    }

    fn render_enum(
        &self,
        name: &str,
        type_params: &[ast::TypeParameter],
        def: &ast::EnumDef,
    ) -> Tokens {
        let generics = render_type_params(type_params);
        let is_unit_only = def.variants.iter().all(|v| v.def.is_unit());

        if is_unit_only {
            quote! {
                export type $(name)$(generics) = $(for v in &def.variants join( | ) => $(quoted(&v.name)));
            }
        } else {
            let variant_tokens = def.variants.iter().map(|variant| {
                let docs = doc_tokens(&variant.docs);
                let payload = self.ts_struct_def_tokens(&variant.def);
                quote!({ $docs$(&variant.name): $payload })
            });

            quote! {
                export type $(name)$(generics) = $(for v in variant_tokens join ( | ) => $v);
            }
        }
    }

    pub(crate) fn ts_struct_def_tokens(&self, def: &ast::StructDef) -> Tokens {
        if def.is_unit() {
            quote! { null }
        } else if def.is_tuple() {
            if def.fields.len() == 1 {
                self.ts_type_decl(&def.fields[0].type_decl)
            } else {
                let tuple_types = def
                    .fields
                    .iter()
                    .map(|field| self.ts_type_decl(&field.type_decl));
                quote! { [$(for ty in tuple_types join (, ) => $ty)] }
            }
        } else {
            let fields = def.fields.iter().map(|field| {
                let field_name =
                    escape_ident(field.name.as_deref().expect("field name should be present"));
                let field_ty = self.ts_type_decl(&field.type_decl);
                quote! { $(field_name): $field_ty }
            });
            quote! { { $(for f in fields join (; ) => $f) } }
        }
    }

    pub(crate) fn ts_type_decl(&self, ty: &ast::TypeDecl) -> Tokens {
        match ty {
            ast::TypeDecl::Primitive(p) => match p {
                ast::PrimitiveType::Void => quote! { null },
                ast::PrimitiveType::Bool => quote! { boolean },
                ast::PrimitiveType::Char | ast::PrimitiveType::String => quote! { string },
                ast::PrimitiveType::I8
                | ast::PrimitiveType::I16
                | ast::PrimitiveType::I32
                | ast::PrimitiveType::I64
                | ast::PrimitiveType::U8
                | ast::PrimitiveType::U16
                | ast::PrimitiveType::U32
                | ast::PrimitiveType::U64 => quote! { number },
                ast::PrimitiveType::I128 | ast::PrimitiveType::U128 | ast::PrimitiveType::U256 => {
                    quote! { bigint }
                }
                ast::PrimitiveType::ActorId => {
                    let import = js::import("sails-js", "ActorId");
                    quote! { $import }
                }
                ast::PrimitiveType::CodeId => {
                    let import = js::import("sails-js", "CodeId");
                    quote! { $import }
                }
                ast::PrimitiveType::MessageId => {
                    let import = js::import("sails-js", "MessageId");
                    quote! { $import }
                }
                ast::PrimitiveType::H160 => {
                    let import = js::import("sails-js", "H160");
                    quote! { $import }
                }
                ast::PrimitiveType::H256 => {
                    let import = js::import("sails-js", "H256");
                    quote! { $import }
                }
            },
            ast::TypeDecl::Slice { item } => {
                let ty = self.ts_type_decl(item);
                quote! { $ty[] }
            }
            ast::TypeDecl::Array { item, len } => {
                if *len == 32 {
                    let rendered = "`0x${string}`".to_string();
                    quote! { $rendered }
                } else {
                    let ty = self.ts_type_decl(item);
                    quote! { $ty[] }
                }
            }
            ast::TypeDecl::Tuple { types } => {
                if types.is_empty() {
                    quote! { null }
                } else {
                    let tuple_types = types.iter().map(|t| self.ts_type_decl(t));
                    quote! { [$(for ty in tuple_types join (, ) => $ty)] }
                }
            }
            ast::TypeDecl::Named { name, generics } => {
                if name == "Option" && generics.len() == 1 {
                    let ty = self.ts_type_decl(&generics[0]);
                    return quote!($ty | null);
                }
                if name == "Result" && generics.len() == 2 {
                    let ok = self.ts_type_decl(&generics[0]);
                    let err = self.ts_type_decl(&generics[1]);
                    return quote!({ ok: $ok } | { err: $err });
                }

                if name == "NonZeroU8"
                    || name == "NonZeroU16"
                    || name == "NonZeroU32"
                    || name == "NonZeroU64"
                    || name == "NonZeroU128"
                    || name == "NonZeroU256"
                {
                    let import = js::import("sails-js", name);
                    return quote! { $import };
                }

                if generics.is_empty() {
                    quote! { $name }
                } else {
                    let generic_tokens = generics.iter().map(|g| self.ts_type_decl(g));
                    quote! { $name<$(for g in generic_tokens join (, ) => $g)> }
                }
            }
        }
    }
}

fn render_type_params(params: &[ast::TypeParameter]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        format!(
            "<{}>",
            params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
