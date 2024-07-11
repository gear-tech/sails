use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::helpers::*;
use crate::type_generators::generate_type_decl_code;

/// Generates a trait with service methods
pub(crate) struct ServiceTraitGenerator {
    service_name: String,
    tokens: Tokens,
}

impl ServiceTraitGenerator {
    pub(crate) fn new(service_name: String) -> Self {
        Self {
            service_name,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote! {
            pub trait $(&self.service_name)<A> {
                $(self.tokens)
            }
        }
    }
}

impl<'ast> Visitor<'ast> for ServiceTraitGenerator {
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
        let output_trait = if func.is_query() { "Query" } else { "Call" };

        quote_in! { self.tokens=>
            #[allow(clippy::type_complexity)]
            fn $fn_name (&$mutability self, $params_tokens) -> impl $output_trait<A, $output_type_decl_code>;
        };
    }
}

/// Generates a client that implements service trait
pub(crate) struct ServiceClientGenerator {
    service_name: String,
    tokens: Tokens,
}

impl ServiceClientGenerator {
    pub(crate) fn new(service_name: String) -> Self {
        Self {
            service_name,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

// using quote_in instead of tokens.append
impl<'ast> Visitor<'ast> for ServiceClientGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        let name = &self.service_name;

        quote_in! {self.tokens =>
            #[derive(Clone)]
            pub struct $name<R, A> where
                R: Remoting<A>,
                A: Default,
            {
                remoting: R,
                _phantom: PhantomData<A>,
            }

            impl<A: Default, R: Remoting<A>> $name<R, A> {
                pub fn new(remoting: R) -> Self {
                    Self { remoting, _phantom: PhantomData }
                }
            }

            impl<R, A> traits::$name<A> for $name<R, A>
            where
                R: Remoting<A> + Clone,
                A: Default,
            $("{")
        };

        visitor::accept_service(service, self);

        quote_in! {self.tokens =>
            $("}")
        };
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name();
        let fn_name_snake = fn_name.to_case(Case::Snake);

        let mut params_tokens = Tokens::new();
        for param in func.params() {
            let type_decl_code = generate_type_decl_code(param.type_decl());
            quote_in! {params_tokens =>
                $(param.name()): $(type_decl_code),
            };
        }

        let output_type_decl_code = generate_type_decl_code(func.output());
        let output_trait = if func.is_query() { "Query" } else { "Call" };

        let args = encoded_fn_args(func.params());

        let service_name_snake = self.service_name.to_case(Case::Snake);
        let params_type = format!("{service_name_snake}::io::{fn_name}");

        quote_in! {self.tokens =>
            fn $fn_name_snake (&$mutability self, $params_tokens) -> impl $output_trait<A, $output_type_decl_code> {
                RemotingAction::new(self.remoting.clone(), $params_type ($args))
            }
        };
    }
}
