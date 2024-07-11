use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::helpers::*;
use crate::type_generators::generate_type_decl_with_path;

pub(crate) struct IoModuleGenerator {
    path: String,
    tokens: rust::Tokens,
}

impl IoModuleGenerator {
    pub(crate) fn new(path: String) -> Self {
        Self {
            path,
            tokens: rust::Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> rust::Tokens {
        quote!(
            pub mod io {
                use super::*;
                use sails_rtl::calls::EncodeDecodeWithRoute;
                $(self.tokens)
            }
        )
    }
}

impl<'ast> Visitor<'ast> for IoModuleGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        visitor::accept_service(service, self);
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let fn_name = func.name();

        let mut func_param_tokens = rust::Tokens::new();
        for func_param in func.params() {
            let type_decl_code =
                generate_type_decl_with_path(func_param.type_decl(), "super".to_owned());
            quote_in! { func_param_tokens =>
                pub $(type_decl_code),
            };
        }

        let func_output = generate_type_decl_with_path(func.output(), "super".to_owned());
        // let is_unit_output = func_output == "()";

        let (service_path_bytes, _) = path_bytes(&self.path);
        let (route_bytes, _) = method_bytes(fn_name);

        quote_in! { self.tokens =>
            #[derive(Debug, Encode)]
            #[codec(crate = sails_rtl::scale_codec)]
            pub struct $fn_name ($func_param_tokens);

            impl $fn_name {
                const ROUTE: &'static [u8] = &[
                    $service_path_bytes $route_bytes
                ];
            }

            impl EncodeDecodeWithRoute for $fn_name {
                type Reply = $func_output;

                fn route() -> &'static [u8] {
                    $fn_name::ROUTE
                }
            }
        };
    }
}
