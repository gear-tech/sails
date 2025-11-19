use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{ast, visitor::Visitor};

pub(crate) struct TypeParameterGenerator {
    tokens: Tokens,
}

impl TypeParameterGenerator {
    pub(crate) fn new() -> Self {
        Self {
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for TypeParameterGenerator {
    fn visit_type_parameter(&mut self, type_param: &'ast ast::TypeParameter) {
        quote_in! { self.tokens =>
            $(&type_param.name)
        };
    }
}
