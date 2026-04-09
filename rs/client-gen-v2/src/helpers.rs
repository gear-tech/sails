use crate::type_generators::generate_type_decl_with_path;
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::ast;

pub(crate) fn fn_args(params: &[ast::FuncParam]) -> String {
    params
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn encoded_args(params: &[ast::FuncParam]) -> String {
    let sep = if params.len() == 1 { "," } else { "" };
    let arg_names = fn_args(params);

    format!("({arg_names}{sep})")
}

pub(crate) fn fn_args_with_types_path<'ast>(
    params: &'ast [ast::FuncParam],
    path: &'ast str,
) -> String {
    params
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.name,
                generate_type_decl_with_path(&p.type_decl, path)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn generate_doc_comments(target_tokens: &mut Tokens, docs: &[String]) {
    for doc in docs {
        quote_in! { *target_tokens =>
            $['\r'] $("///") $doc
        };
    }
}
