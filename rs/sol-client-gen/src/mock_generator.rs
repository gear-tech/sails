use crate::helpers::fn_args_with_types_path;
use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{
    ast::{self, ServiceIdent},
    visitor::{self, Visitor},
};

pub(crate) struct MockGenerator<'ast> {
    service_name: &'ast str,
    sails_path: &'ast str,
    tokens: rust::Tokens,
}

impl<'ast> MockGenerator<'ast> {
    pub(crate) fn new(service_name: &'ast str, sails_path: &'ast str) -> Self {
        Self {
            service_name,
            sails_path,
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
                    type Env = $(self.sails_path)::client::GstdEnv;
                    $(self.tokens)
                }
            }
        }
    }
}

impl<'ast> Visitor<'ast> for MockGenerator<'ast> {
    fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
        visitor::accept_service_unit(service, self);

        for ServiceIdent {
            name,
            interface_id: _,
        } in &service.extends
        {
            let method_name = name.to_case(Case::Snake);
            let impl_name = name.to_case(Case::Pascal);
            let mod_name = name.to_case(Case::Snake);

            quote_in! { self.tokens =>
                fn $(&method_name) (&self, ) -> $(self.sails_path)::client::Service<super::$(mod_name.as_str())::$(impl_name.as_str())Impl, $(self.sails_path)::client::GstdEnv>;
            };
        }
    }

    fn visit_service_func(&mut self, func: &'ast ast::ServiceFunc) {
        let service_name_snake = &self.service_name.to_case(Case::Snake);
        let self_ref = if func.kind == ast::FunctionKind::Query {
            "&self"
        } else {
            "&mut self"
        };
        let fn_name = &func.name;
        let fn_name_snake = func.name.to_case(Case::Snake);
        let params_with_types = &fn_args_with_types_path(&func.params, "");

        quote_in! { self.tokens =>
            fn $fn_name_snake ($self_ref, $params_with_types) -> $(self.sails_path)::client::PendingCall<$service_name_snake::io::$fn_name, $(self.sails_path)::client::GstdEnv>;
        };
    }
}
