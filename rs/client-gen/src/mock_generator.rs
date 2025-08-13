use crate::helpers::fn_args_with_types;
use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct MockGenerator<'a> {
    service_name: &'a str,
    tokens: rust::Tokens,
}

impl<'a> MockGenerator<'a> {
    pub(crate) fn new(service_name: &'a str) -> Self {
        Self {
            service_name,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let service_name_snake = &self.service_name.to_case(Case::Snake);
        quote! {
            mock! {
                pub $(self.service_name) {}

                #[allow(refining_impl_trait)]
                #[allow(clippy::type_complexity)]
                impl $service_name_snake::$(self.service_name) for $(self.service_name) {
                    type Env = MockEnv;
                    $(self.tokens)
                }
            }
        }
    }
}

impl<'ast> Visitor<'ast> for MockGenerator<'_> {
    fn visit_service(&mut self, service: &'ast Service) {
        visitor::accept_service(service, self);
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let service_name_snake = &self.service_name.to_case(Case::Snake);
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name();
        let fn_name_snake = func.name().to_case(Case::Snake);
        let params_with_types = &fn_args_with_types(func.params());

        quote_in! { self.tokens =>
            fn $fn_name_snake (&$mutability self, $params_with_types) -> PendingCall<MockEnv, $service_name_snake::io::$fn_name>;
        };
    }
}
