use crate::func_param_generator::FuncParamGenerator;
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::ast::{FuncParam, StructField};
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

pub(crate) fn fn_args_with_types_path<'ast>(params: &'ast [FuncParam], path: &'ast str) -> String {
    params
        .iter()
        .map(|p| {
            let mut generator = FuncParamGenerator::new(path);
            generator.visit_func_param(p);
            generator
                .finalize()
                .to_string()
                .expect("Failed to generate func param")
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

pub(crate) enum FieldVariantKind {
    Unit,
    Tuple,
    Struct,
    Mixed,
}

pub(crate) fn get_field_variant_kind(fields: &[StructField]) -> FieldVariantKind {
    if fields.is_empty() {
        return FieldVariantKind::Unit;
    }

    let is_tuple = fields.iter().all(|f| f.name.is_none());
    let is_struct = fields.iter().all(|f| f.name.is_some());

    if is_tuple {
        FieldVariantKind::Tuple
    } else if is_struct {
        FieldVariantKind::Struct
    } else {
        FieldVariantKind::Mixed
    }
}
