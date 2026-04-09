use crate::type_generators::generate_type_decl_with_path;
use sails_idl_parser::ast::FuncParam;

pub(crate) fn fn_args(params: &[FuncParam]) -> String {
    params
        .iter()
        .map(|a| a.name())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn encoded_args(params: &[FuncParam]) -> String {
    let sep = if params.len() == 1 { "," } else { "" };
    let arg_names = fn_args(params);

    format!("({arg_names}{sep})")
}

pub(crate) fn fn_args_with_types(params: &[FuncParam]) -> String {
    fn_args_with_types_path(params, "")
}

pub(crate) fn fn_args_with_types_path(params: &[FuncParam], path: &str) -> String {
    params
        .iter()
        .map(|p| {
            let ty = generate_type_decl_with_path(p.type_decl(), path.to_owned());
            format!("{}: {}", p.name(), ty)
        })
        .collect::<Vec<_>>()
        .join(", ")
}
