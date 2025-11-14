use sails_idl_parser_v2::ast::FuncParam;
use crate::func_param_generator::FuncParamGenerator;
use sails_idl_parser_v2::ast::visitor::Visitor;

pub(crate) fn fn_args(params: &[FuncParam]) -> String {
    params
        .iter()
        .map(|a| a.name.as_str())
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
            let mut generator = FuncParamGenerator::new(path.to_owned());
            generator.visit_func_param(p);
            generator.finalize().to_string().expect("Failed to generate func param")
        })
        .collect::<Vec<_>>()
        .join(", ")
}
