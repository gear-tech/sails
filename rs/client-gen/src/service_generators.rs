use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::events_generator::EventsModuleGenerator;
use crate::helpers::*;
use crate::type_generators::generate_type_decl_code;

/// Generates a trait with service methods
pub(crate) struct ServiceCtorGenerator<'a> {
    service_name: &'a str,
    trait_tokens: Tokens,
    impl_tokens: Tokens,
}

impl<'a> ServiceCtorGenerator<'a> {
    pub(crate) fn new(service_name: &'a str) -> Self {
        Self {
            service_name,
            trait_tokens: Tokens::new(),
            impl_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> (Tokens, Tokens) {
        (self.trait_tokens, self.impl_tokens)
    }
}

impl<'ast> Visitor<'ast> for ServiceCtorGenerator<'_> {
    fn visit_service(&mut self, service: &'ast Service) {
        let service_name_snake = &self.service_name.to_case(Case::Snake);
        quote_in!(self.trait_tokens =>
            fn $service_name_snake(&self) -> Service<Self::Env, $service_name_snake::$(self.service_name)Impl>;
        );
        quote_in!(self.impl_tokens =>
            fn $service_name_snake(&self) -> Service<Self::Env, $service_name_snake::$(self.service_name)Impl> {
                self.service(stringify!($(self.service_name)))
            }
        );
    }
}

/// Generates a service module with trait and struct implementation
pub(crate) struct ServiceGenerator<'a> {
    service_name: &'a str,
    sails_path: &'a str,
    trait_tokens: Tokens,
    impl_tokens: Tokens,
    io_tokens: Tokens,
    events_tokens: Tokens,
}

impl<'a> ServiceGenerator<'a> {
    pub(crate) fn new(service_name: &'a str, sails_path: &'a str) -> Self {
        Self {
            service_name,
            sails_path,
            trait_tokens: Tokens::new(),
            impl_tokens: Tokens::new(),
            io_tokens: Tokens::new(),
            events_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let service_name_snake = &self.service_name.to_case(Case::Snake);
        quote! {
            $['\n']
            pub mod $service_name_snake {
                use super::*;

                pub trait $(self.service_name) {
                    type Env: GearEnv;
                    $(self.trait_tokens)
                }

                pub struct $(self.service_name)Impl;

                impl<E: GearEnv> $(self.service_name) for Service<E, $(self.service_name)Impl> {
                    type Env = E;
                    $(self.impl_tokens)
                }

                $['\n']
                pub mod io {
                    use super::*;
                    $(self.io_tokens)
                }

                $(self.events_tokens)
            }
        }
    }
}

// using quote_in instead of tokens.append
impl<'ast> Visitor<'ast> for ServiceGenerator<'_> {
    fn visit_service(&mut self, service: &'ast Service) {
        visitor::accept_service(service, self);

        if !service.events().is_empty() {
            let mut events_mod_gen = EventsModuleGenerator::new(self.service_name, self.sails_path);
            events_mod_gen.visit_service(service);
            self.events_tokens = events_mod_gen.finalize();
        }
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name();
        let fn_name_snake = &fn_name.to_case(Case::Snake);

        let params_with_types = &fn_args_with_types(func.params());
        let args = encoded_args(func.params());
        let output_type_decl_code = generate_type_decl_code(func.output());

        for doc in func.docs() {
            quote_in! { self.trait_tokens =>
                $['\r'] $("///") $doc
            };
        }
        quote_in! { self.trait_tokens =>
            $['\r'] fn $fn_name_snake (&$mutability self, $params_with_types) -> PendingCall<Self::Env, io::$fn_name>;
        };

        quote_in! {self.impl_tokens =>
            $['\r'] fn $fn_name_snake (&$mutability self, $params_with_types) -> PendingCall<Self::Env, io::$fn_name> {
                self.pending_call($args)
            }
        };

        quote_in! { self.io_tokens =>
            $(self.sails_path)::io_struct_impl!($fn_name ($params_with_types) -> $output_type_decl_code);
        };
    }
}
