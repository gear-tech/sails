use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{ast::visitor::Visitor, ast::*};

use crate::events_generator::EventsModuleGenerator;
use crate::helpers::*;
use crate::type_generators::generate_type_decl_with_path;

/// Generates a service module with trait and struct implementation
pub(crate) struct ServiceGenerator<'ast> {
    service_name: &'ast str,
    sails_path: &'ast str,
    trait_tokens: Tokens,
    impl_tokens: Tokens,
    io_tokens: Tokens,
    events_tokens: Tokens,
}

impl<'ast> ServiceGenerator<'ast> {
    pub(crate) fn new(service_name: &'ast str, sails_path: &'ast str) -> Self {
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
                    type Env: $(self.sails_path)::client::GearEnv;
                    $(self.trait_tokens)
                }

                pub struct $(self.service_name)Impl;

                impl<E: $(self.sails_path)::client::GearEnv> $(self.service_name) for $(self.sails_path)::client::Service<$(self.service_name)Impl, E> {
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
impl<'ast> Visitor<'ast> for ServiceGenerator<'ast> {
    fn visit_service_unit(&mut self, service: &'ast ServiceUnit) {
        for func in &service.funcs {
            self.visit_service_func(func);
        }

        if !service.events.is_empty() {
            let mut events_mod_gen = EventsModuleGenerator::new(self.service_name, self.sails_path);
            events_mod_gen.visit_service_unit(service);
            self.events_tokens = events_mod_gen.finalize();
        }
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let self_ref = if func.is_query { "&self" } else { "&mut self" };
        let fn_name = &func.name;
        let fn_name_snake = &fn_name.to_case(Case::Snake);

        let params_with_types = &fn_args_with_types(&func.params);
        let args = encoded_args(&func.params);

        generate_doc_comments(&mut self.trait_tokens, &func.docs);

        quote_in! { self.trait_tokens =>
            $['\r'] fn $fn_name_snake ($self_ref, $params_with_types) -> $(self.sails_path)::client::PendingCall<io::$fn_name, Self::Env>;
        };

        quote_in! {self.impl_tokens =>
            $['\r'] fn $fn_name_snake ($self_ref, $params_with_types) -> $(self.sails_path)::client::PendingCall<io::$fn_name, Self::Env> {
                self.pending_call($args)
            }
        };

        let output_type_decl_code = generate_type_decl_with_path(&func.output, "super");
        let params_with_types_super = &fn_args_with_types_path(&func.params, "super");
        quote_in! { self.io_tokens =>
            $(self.sails_path)::io_struct_impl!($fn_name ($params_with_types_super) -> $output_type_decl_code);
        };
    }
}
