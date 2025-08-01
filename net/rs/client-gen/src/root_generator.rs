use crate::{
    ctor_generators::*, events_generator::*, service_generators::*, tol_level_type_generators::*,
    type_decl_generators::*,
};
use convert_case::{Case, Casing};
use csharp::Tokens;
use genco::{prelude::*, tokens::ItemStr};
use sails_idl_parser::{ast::visitor::Visitor, ast::*};
use std::collections::HashMap;

pub(crate) struct RootGenerator<'a> {
    tokens: Tokens,
    anonymous_service_name: &'a str,
    external_types: HashMap<&'a str, &'a str>,
    generated_types: Vec<&'a Type>,
}

impl<'a> RootGenerator<'a> {
    pub(crate) fn new(
        anonymous_service_name: &'a str,
        namespace: &'a str,
        external_types: HashMap<&'a str, &'a str>,
    ) -> Self {
        let mut tokens = Tokens::new();
        tokens.append(ItemStr::Static("#nullable enable"));
        tokens.line();
        tokens.append(ItemStr::Static(
            "#pragma warning disable RCS0056 // A line is too long",
        ));
        tokens.line();
        tokens.append(format!("namespace {namespace};"));
        tokens.line();
        Self {
            tokens,
            anonymous_service_name,
            external_types,
            generated_types: Vec::new(),
        }
    }

    pub(crate) fn finalize(mut self) -> Tokens {
        for &type_ in &self.generated_types {
            let mut type_gen = TopLevelTypeGenerator::new(
                type_.name(),
                TypeDeclGenerator::new(&self.generated_types),
            );
            type_gen.visit_type(type_);
            self.tokens.extend(type_gen.finalize());
        }
        self.tokens
    }
}

impl<'a> Visitor<'a> for RootGenerator<'a> {
    fn visit_ctor(&mut self, ctor: &'a Ctor) {
        let mut ctor_gen = CtorFactoryGenerator::new(
            self.anonymous_service_name.to_case(Case::Pascal),
            TypeDeclGenerator::new(&self.generated_types),
        );
        ctor_gen.visit_ctor(ctor);
        self.tokens.extend(ctor_gen.finalize());
    }

    fn visit_service(&mut self, service: &'a Service) {
        let service_name = if service.name().is_empty() {
            self.anonymous_service_name
        } else {
            service.name()
        };
        let mut service_gen = ServiceClientGenerator::new(
            service_name.to_owned(),
            TypeDeclGenerator::new(&self.generated_types),
        );
        service_gen.visit_service(service);
        self.tokens.extend(service_gen.finalize());

        if !service.events().is_empty() {
            let mut events_mod_gen =
                EventsGenerator::new(service_name, TypeDeclGenerator::new(&self.generated_types));
            events_mod_gen.visit_service(service);
            self.tokens.extend(events_mod_gen.finalize());
        }
    }

    fn visit_type(&mut self, t: &'a Type) {
        if self.external_types.contains_key(t.name()) {
            return;
        }
        // collect all generated types
        // used later to add prefix to enum types
        self.generated_types.push(t);
    }
}
