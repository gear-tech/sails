use crate::{helpers::push_doc, naming::escape_ident};
use genco::prelude::*;
use js::Tokens;
use sails_idl_parser_v2::ast;
use std::collections::BTreeMap;

pub(crate) struct TypeGenerator<'a> {
    type_index: BTreeMap<&'a str, &'a ast::Type>,
}

impl<'a> TypeGenerator<'a> {
    pub(crate) fn new(types: &'a [ast::Type]) -> Self {
        let mut type_index = BTreeMap::new();
        for ty in types {
            type_index.insert(ty.name.as_str(), ty);
        }
        Self { type_index }
    }

    pub(crate) fn render_all(&self, tokens: &mut Tokens, types: &'a [ast::Type]) {
        for ty in types {
            self.render_type(tokens, ty);
            tokens.push();
        }
    }

    pub(crate) fn render_type(&self, tokens: &mut Tokens, ty: &'a ast::Type) {
        let _ = self.type_index.get(ty.name.as_str());
        push_doc(tokens, &ty.docs);
        match &ty.def {
            ast::TypeDef::Struct(def) => self.render_struct(tokens, &ty.name, &ty.type_params, def),
            ast::TypeDef::Enum(def) => self.render_enum(tokens, &ty.name, &ty.type_params, def),
        }
    }

    fn render_struct(
        &self,
        tokens: &mut Tokens,
        name: &str,
        type_params: &[ast::TypeParameter],
        def: &ast::StructDef,
    ) {
        let generics = render_type_params(type_params);
        if def.is_tuple() {
            if def.fields.is_empty() {
                tokens.append(format!("export type {name}{generics} = null;\n"));
            } else if def.fields.len() == 1 {
                let ty = self.ts_type_decl(&def.fields[0].type_decl);
                tokens.append(format!("export type {name}{generics} = {ty};\n"));
            } else {
                let mut tuple = String::new();
                tuple.push('[');
                for (idx, field) in def.fields.iter().enumerate() {
                    if idx > 0 {
                        tuple.push_str(", ");
                    }
                    tuple.push_str(&self.ts_type_decl(&field.type_decl));
                }
                tuple.push(']');
                tokens.append(format!("export type {name}{generics} = {tuple};\n"));
            }
            return;
        }

        tokens.append(format!("export interface {name}{generics} {{\n"));
        for field in &def.fields {
            push_doc(tokens, &field.docs);
            let field_name = escape_ident(field.name.as_deref().unwrap_or("field"));
            let field_ty = self.ts_type_decl(&field.type_decl);
            tokens.append(format!("  {field_name}: {field_ty};\n"));
        }
        tokens.append("}\n");
    }

    fn render_enum(
        &self,
        tokens: &mut Tokens,
        name: &str,
        type_params: &[ast::TypeParameter],
        def: &ast::EnumDef,
    ) {
        let generics = render_type_params(type_params);
        let is_unit_only = def.variants.iter().all(|v| v.def.is_unit());

        if is_unit_only {
            let mut union = String::new();
            for (idx, variant) in def.variants.iter().enumerate() {
                if idx > 0 {
                    union.push_str(" | ");
                }
                union.push_str(&format!("'{}'", variant.name));
            }
            tokens.append(format!("export type {name}{generics} = {union};\n"));
            return;
        }

        tokens.append(format!("export type {name}{generics} =\n"));
        for (idx, variant) in def.variants.iter().enumerate() {
            push_doc(tokens, &variant.docs);
            let payload = if variant.def.is_unit() {
                "null".to_string()
            } else if variant.def.is_tuple() {
                if variant.def.fields.len() == 1 {
                    self.ts_type_decl(&variant.def.fields[0].type_decl)
                } else {
                    let mut tuple = String::new();
                    tuple.push('[');
                    for (f_idx, field) in variant.def.fields.iter().enumerate() {
                        if f_idx > 0 {
                            tuple.push_str(", ");
                        }
                        tuple.push_str(&self.ts_type_decl(&field.type_decl));
                    }
                    tuple.push(']');
                    tuple
                }
            } else {
                let mut obj = String::new();
                obj.push('{');
                for (f_idx, field) in variant.def.fields.iter().enumerate() {
                    if f_idx > 0 {
                        obj.push(' ');
                    }
                    let field_name = escape_ident(field.name.as_deref().unwrap_or("field"));
                    let ty = self.ts_type_decl(&field.type_decl);
                    obj.push_str(&format!("{field_name}: {ty};"));
                }
                obj.push('}');
                obj
            };

            let suffix = if idx + 1 == def.variants.len() { ";" } else { "" };
            tokens.append(format!("  | {{ {}: {} }}{suffix}\n", variant.name, payload));
        }
    }

    fn ts_type_decl(&self, ty: &ast::TypeDecl) -> String {
        match ty {
            ast::TypeDecl::Primitive(p) => match p {
                ast::PrimitiveType::Void => "null".to_string(),
                ast::PrimitiveType::Bool => "boolean".to_string(),
                ast::PrimitiveType::Char | ast::PrimitiveType::String => "string".to_string(),
                ast::PrimitiveType::I8
                | ast::PrimitiveType::I16
                | ast::PrimitiveType::I32
                | ast::PrimitiveType::I64
                | ast::PrimitiveType::U8
                | ast::PrimitiveType::U16
                | ast::PrimitiveType::U32
                | ast::PrimitiveType::U64 => "number".to_string(),
                ast::PrimitiveType::I128 | ast::PrimitiveType::U128 | ast::PrimitiveType::U256 => {
                    "bigint".to_string()
                }
                ast::PrimitiveType::ActorId
                | ast::PrimitiveType::CodeId
                | ast::PrimitiveType::MessageId
                | ast::PrimitiveType::H160
                | ast::PrimitiveType::H256 => "`0x${string}`".to_string(),
            },
            ast::TypeDecl::Slice { item } => format!("{}[]", self.ts_type_decl(item)),
            ast::TypeDecl::Array { item, len } => {
                if *len == 32 {
                    "`0x${string}`".to_string()
                } else {
                    format!("{}[]", self.ts_type_decl(item))
                }
            }
            ast::TypeDecl::Tuple { types } => {
                if types.is_empty() {
                    "null".to_string()
                } else {
                    format!(
                        "[{}]",
                        types
                            .iter()
                            .map(|t| self.ts_type_decl(t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            ast::TypeDecl::Named { name, generics } => {
                if name == "Option" && generics.len() == 1 {
                    return format!("{} | null", self.ts_type_decl(&generics[0]));
                }
                if name == "Result" && generics.len() == 2 {
                    return format!(
                        "{{ ok: {} }} | {{ err: {} }}",
                        self.ts_type_decl(&generics[0]),
                        self.ts_type_decl(&generics[1])
                    );
                }

                if name == "NonZeroU8"
                    || name == "NonZeroU16"
                    || name == "NonZeroU32"
                    || name == "NonZeroU64"
                {
                    return "number".to_string();
                }
                if name == "NonZeroU128" || name == "NonZeroU256" {
                    return "bigint".to_string();
                }

                if generics.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{}<{}>",
                        name,
                        generics
                            .iter()
                            .map(|g| self.ts_type_decl(g))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
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

