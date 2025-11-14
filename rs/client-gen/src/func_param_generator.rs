use crate::type_generators::generate_type_decl_with_path;
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{ast::FuncParam, ast::visitor::Visitor};

pub(crate) struct FuncParamGenerator<'ast> {
    tokens: Tokens,
    path: &'ast str,
}

impl<'ast> FuncParamGenerator<'ast> {
    pub(crate) fn new(path: &'ast str) -> Self {
        Self {
            tokens: Tokens::new(),
            path,
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for FuncParamGenerator<'ast> {
    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_with_path(&func_param.type_decl, self.path);
        quote_in! { self.tokens =>
            $(&func_param.name): $type_decl_code
        };
    }
}
