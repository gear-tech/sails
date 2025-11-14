use crate::type_generators::generate_type_decl_with_path;
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::ast::visitor::Visitor; // Import Visitor trait
use sails_idl_parser_v2::ast::FuncParam;

pub(crate) struct FuncParamGenerator {
    path: String,
    tokens: Tokens,
}

impl FuncParamGenerator {
    pub(crate) fn new(path: String) -> Self {
        Self {
            path,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for FuncParamGenerator { // Remove lifetime from impl block
    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_with_path(&func_param.type_decl, self.path.clone());
        quote_in! { self.tokens =>
            $(&func_param.name): $type_decl_code
        };
    }
}