use genco::prelude::*;
use js::Tokens;
use sails_idl_parser_v2::ast;

pub(crate) fn push_doc(tokens: &mut Tokens, docs: &[String]) {
    if docs.is_empty() {
        return;
    }

    tokens.append("/**");
    tokens.push();
    for line in docs {
        tokens.append(format!(" * {}", line));
        tokens.push();
    }
    tokens.append(" */");
    tokens.push();
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
