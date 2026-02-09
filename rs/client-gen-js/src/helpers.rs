use genco::prelude::*;
use js::Tokens;
use sails_idl_parser_v2::ast;

pub(crate) fn push_doc(tokens: &mut Tokens, docs: &[String]) {
    if docs.is_empty() {
        return;
    }

    tokens.append("/**\n");
    for line in docs {
        tokens.append(format!(" * {}\n", line));
    }
    tokens.append(" */\n");
}

pub(crate) fn doc_tokens(docs: &[String]) -> Tokens {
    let mut tokens = Tokens::new();
    push_doc(&mut tokens, docs);
    tokens
}

pub(crate) fn serialize_type(ty: &ast::Type) -> String {
    ty.to_json_string()
        .expect("Type should be serializable to JSON")
}

pub(crate) fn serialize_type_decl(ty: &ast::TypeDecl) -> String {
    ty.to_json_string()
        .expect("TypeDecl should be serializable to JSON")
}

pub(crate) fn ts_type_decl(ty: &ast::TypeDecl) -> String {
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
        ast::TypeDecl::Slice { item } => format!("{}[]", ts_type_decl(item)),
        ast::TypeDecl::Array { item, len } => {
            if *len == 32 {
                "`0x${string}`".to_string()
            } else {
                format!("{}[]", ts_type_decl(item))
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
                        .map(ts_type_decl)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        ast::TypeDecl::Named { name, generics } => {
            if name == "Option" && generics.len() == 1 {
                return format!("{} | null", ts_type_decl(&generics[0]));
            }
            if name == "Result" && generics.len() == 2 {
                return format!(
                    "{{ ok: {} }} | {{ err: {} }}",
                    ts_type_decl(&generics[0]),
                    ts_type_decl(&generics[1])
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
                        .map(ts_type_decl)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

pub(crate) fn payload_type_expr(params: &[ast::FuncParam], resolver_expr: &str) -> String {
    if params.is_empty() {
        "null".to_string()
    } else if params.len() == 1 {
        format!(
            "{resolver_expr}.getTypeDeclString({})",
            serialize_type_decl(&params[0].type_decl)
        )
    } else {
        let tuple_types = params
            .iter()
            .map(|p| serialize_type_decl(&p.type_decl))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{resolver_expr}.getTypeDeclString({{\"kind\":\"tuple\",\"types\":[{tuple_types}]}})"
        )
    }
}
