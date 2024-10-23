use crate::{
    ctor_generators::*, events_generator::*, io_generators::*, mock_generator::MockGenerator,
    service_generators::*, type_generators::*,
};
use convert_case::{Case, Casing};
use csharp::Tokens;
use genco::prelude::*;
use sails_idl_parser::{ast::visitor::Visitor, ast::*};
use std::collections::HashMap;

pub(crate) struct RootGenerator<'a> {
    tokens: Tokens,
    traits_tokens: Tokens,
    mocks_tokens: Tokens,
    anonymous_service_name: &'a str,
    external_types: HashMap<&'a str, &'a str>,
}

impl<'a> RootGenerator<'a> {
    pub(crate) fn new(
        anonymous_service_name: &'a str,
        external_types: HashMap<&'a str, &'a str>,
    ) -> Self {
        Self {
            anonymous_service_name,
            tokens: Tokens::new(),
            traits_tokens: Tokens::new(),
            mocks_tokens: Tokens::new(),
            external_types,
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote! {
            $(self.tokens)
        }
    }
}

impl<'a, 'ast> Visitor<'ast> for RootGenerator<'a> {
    fn visit_ctor(&mut self, ctor: &'ast Ctor) {}

    fn visit_service(&mut self, service: &'ast Service) {}

    fn visit_type(&mut self, t: &'ast Type) {
        if self.external_types.contains_key(t.name()) {
            return;
        }
        let mut type_gen = TopLevelTypeGenerator::new(t.name());
        type_gen.visit_type(t);
        self.tokens.extend(type_gen.finalize());
    }
}
