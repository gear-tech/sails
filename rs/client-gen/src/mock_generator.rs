use crate::type_generators::generate_type_decl_code;
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
            tokens: rust::Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> rust::Tokens {
        quote! {
            mock! {
                pub $(self.service_name)<R: Remoting> {}

                #[allow(refining_impl_trait)]
                #[allow(clippy::type_complexity)]
                impl<R: Remoting> traits::$(self.service_name) for $(self.service_name)<R> {
                    type Remoting = R;
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
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name().to_case(Case::Snake);

        let mut params_tokens = Tokens::new();
        for param in func.params() {
            let type_decl_code = generate_type_decl_code(param.type_decl());
            quote_in! {params_tokens =>
                $(param.name()): $(type_decl_code),
            };
        }

        let output_type_decl_code = generate_type_decl_code(func.output());
        let output_mock = if func.is_query() {
            "MockQuery"
        } else {
            "MockCall"
        };

        quote_in! { self.tokens=>
            fn $fn_name (&$mutability self, $params_tokens) -> $output_mock<R, $output_type_decl_code>;
        };
    }
}
