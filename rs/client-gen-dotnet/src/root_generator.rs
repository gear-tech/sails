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
    generated_types: Vec<&'a Type>,
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
            generated_types: Vec::new(),
        }
    }

    pub(crate) fn finalize(mut self) -> Tokens {
        for &type_ in &self.generated_types {
            let mut type_gen = TopLevelTypeGenerator::new(&type_.name(), &self.generated_types);
            type_gen.visit_type(type_);
            self.tokens.extend(type_gen.finalize());
        }
        self.tokens
    }
}

impl<'a> Visitor<'a> for RootGenerator<'a> {
    fn visit_ctor(&mut self, ctor: &'a Ctor) {}

    fn visit_service(&mut self, service: &'a Service) {}

    fn visit_type(&mut self, t: &'a Type) {
        if self.external_types.contains_key(t.name()) {
            return;
        }
        // collect all generated types
        // used later to add prefix to enum types
        self.generated_types.push(t);
    }
}
