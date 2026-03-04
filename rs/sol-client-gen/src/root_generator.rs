use crate::{
    ctor_generators::*, helpers::generate_doc_comments, service_generators::*, type_generators::*,
};
use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::ast;
use sails_idl_parser_v2::visitor::Visitor;
use std::collections::{HashMap, HashSet};

pub(crate) struct RootGenerator<'ast> {
    tokens: Tokens,
    service_impl_tokens: Tokens,
    service_trait_tokens: Tokens,
    program_meta_tokens: Tokens,
    program_name: Option<&'ast str>,
    mocks_feature_name: Option<&'ast str>,
    sails_path: &'ast str,
    external_types: HashMap<&'ast str, &'ast str>,
    no_derive_traits: bool,
    program_types: HashSet<&'ast str>,
    program_service_routes: HashMap<&'ast str, &'ast str>,
}

impl<'ast> RootGenerator<'ast> {
    pub(crate) fn new(
        mocks_feature_name: Option<&'ast str>,
        sails_path: &'ast str,
        external_types: HashMap<&'ast str, &'ast str>,
        no_derive_traits: bool,
    ) -> Self {
        Self {
            tokens: Tokens::new(),
            service_impl_tokens: Tokens::new(),
            service_trait_tokens: Tokens::new(),
            program_meta_tokens: Tokens::new(),
            program_name: None,
            mocks_feature_name,
            sails_path,
            external_types,
            no_derive_traits,
            program_types: HashSet::new(),
            program_service_routes: HashMap::new(),
        }
    }

    pub(crate) fn finalize(self, with_no_std: bool) -> String {
        let mut tokens = if let Some(mocks_feature_name) = self.mocks_feature_name {
            quote! {
                $['\n']
                #[cfg(feature = $(quoted(mocks_feature_name)))]
                #[cfg(not(target_arch = "wasm32"))]
                extern crate std;
            }
        } else {
            Tokens::new()
        };

        quote_in! { tokens =>
            #[allow(unused_imports)]
            use $(self.sails_path)::{client::*, collections::*, prelude::*};
            #[allow(unused_imports)]
            use $(self.sails_path)::alloy_sol_types;
            #[allow(unused_imports)]
            use $(self.sails_path)::alloy_sol_types::SolValue;
            #[allow(unused_imports)]
            use $(self.sails_path)::solidity;
        };

        for (&name, &path) in &self.external_types {
            quote_in! { tokens =>
                #[allow(unused_imports)]
                use $path as $name;
            };
        }

        if let Some(program_name) = self.program_name {
            quote_in! { tokens =>
                pub struct $(program_name)Program;

                impl $(program_name)Program {
                    $(self.program_meta_tokens)
                }

                impl $(self.sails_path)::client::Program for  $(program_name)Program {}

                pub trait $program_name {
                    type Env: $(self.sails_path)::client::GearEnv;
                    $(self.service_trait_tokens)
                }

                impl<E: $(self.sails_path)::client::GearEnv> $program_name for $(self.sails_path)::client::Actor<$(program_name)Program, E> {
                    type Env = E;
                    $(self.service_impl_tokens)
                }
            };
        }

        tokens.extend(self.tokens);

        let mut result = tokens.to_file_string().unwrap();

        let sol_macro = r#"
#[macro_export]
macro_rules! io_struct_sol_impl {
    (
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty,
        $entry_id:expr,
        $interface_id:expr,
        $selector_prefix:expr,
        $selector_name:expr
    ) => {
        pub struct $name(());

        impl $name {
            fn selector() -> [u8; 4] {
                let mut s = String::from($selector_prefix);
                s.push_str($selector_name);
                s.push_str(<<Self as ServiceCall>::Params::SolType as alloy_sol_types::SolType>::SOL_NAME);
                let selector = solidity::selector(s);
                selector.into()
            }

            pub fn encode_call(route_idx: u8, $( $param: $ty, )* ) -> Vec<u8> {
                <$name as ServiceCall>::encode_params_with_header(route_idx, &(false, $( $param, )* ))
            }

            pub fn decode_reply(route_idx: u8, payload: impl AsRef<[u8]>) -> Result<$reply, scale_codec::Error> {
                <$name as ServiceCall>::decode_reply_with_header(route_idx, payload)
            }
        }

        impl Identifiable for $name {
            const INTERFACE_ID: InterfaceId = $interface_id;
        }

        impl MethodMeta for $name {
            const ENTRY_ID: u16 = $entry_id;
        }

        impl ServiceCall for $name {
            type Params = (bool, $( $ty, )* );
            type Reply = $reply;

            fn encode_params_with_header(_route_idx: u8, value: &Self::Params) -> Vec<u8> {
                let mut payload = Self::selector().to_vec();
                payload.extend(alloy_sol_types::SolValue::abi_encode_sequence(value));
                payload
            }

            fn decode_reply_with_header(
                _route_idx: u8,
                payload: impl AsRef<[u8]>,
            ) -> Result<Self::Reply, parity_scale_codec::Error> {
                alloy_sol_types::SolValue::abi_decode(payload.as_ref(), true)
                    .map_err(|_| parity_scale_codec::Error::from("Failed to decode ABI reply"))
            }
        }
    };
}

"#;

        if with_no_std {
            result.insert_str(
                0,
                "// Code generated by sails-sol-client-gen. DO NOT EDIT.\n#![no_std]\n\n",
            );
        } else {
            result.insert_str(0, "// Code generated by sails-sol-client-gen. DO NOT EDIT.\n");
        }

        result.insert_str(
            result
                .find("use ")
                .unwrap_or(0),
            sol_macro,
        );

        result
    }
}

impl<'ast> Visitor<'ast> for RootGenerator<'ast> {
    fn visit_program_unit(&mut self, program: &'ast ast::ProgramUnit) {
        self.program_name = Some(&program.name);
        self.program_service_routes.clear();
        for service in &program.services {
            self.program_service_routes.insert(
                &service.name.name,
                service.route.as_deref().unwrap_or(&service.name.name),
            );
        }

        let mut ctor_gen = CtorGenerator::new(&program.name, self.sails_path);
        ctor_gen.visit_program_unit(program);
        self.tokens.extend(ctor_gen.finalize());

        sails_idl_parser_v2::visitor::accept_program_unit(program, self);
    }

    fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
        let service_route = self
            .program_service_routes
            .get(service.name.name.as_str())
            .copied()
            .unwrap_or(service.name.name.as_str());
        let mut client_gen = ServiceGenerator::new(
            &service.name.name,
            service_route,
            self.sails_path,
            &self.external_types,
            self.mocks_feature_name,
            service.name.interface_id.unwrap_or(sails_idl_meta::InterfaceId::zero()),
            self.no_derive_traits,
        );
        client_gen.visit_service_unit(service);
        self.tokens.extend(client_gen.finalize());
    }

    fn visit_type(&mut self, t: &'ast ast::Type) {
        self.program_types.insert(&t.name);
        if self.external_types.contains_key(t.name.as_str()) {
            return;
        }
        let mut type_gen =
            TopLevelTypeGenerator::new(&t.name, self.sails_path, self.no_derive_traits);
        type_gen.visit_type(t);
        self.tokens.extend(type_gen.finalize());
    }

    fn visit_service_expo(&mut self, service_item: &'ast ast::ServiceExpo) {
        let service_route = service_item
            .route
            .as_ref()
            .unwrap_or(&service_item.name.name);
        let method_name = service_route.to_case(Case::Snake);
        let name_pascal_case = service_item.name.name.to_case(Case::Pascal);
        let name_snake_case = service_item.name.name.to_case(Case::Snake);
        let program_name = self.program_name.unwrap();
        let program_program_name = format!("{program_name}Program");

        generate_doc_comments(&mut self.service_trait_tokens, &service_item.docs);

        let route_id_const_name = format!("ROUTE_ID_{}", service_route.to_case(Case::UpperSnake));

        quote_in!(self.program_meta_tokens =>
             pub const $(&route_id_const_name): u8 = $(service_item.route_idx);
        );

        quote_in!(self.service_trait_tokens =>
            $['\r'] fn $(&method_name)(&self) -> $(self.sails_path)::client::Service<$(&name_snake_case)::$(&name_pascal_case)Impl, Self::Env>;
        );

        quote_in!(self.service_impl_tokens =>
            $['\r'] fn $(&method_name)(&self) -> $(self.sails_path)::client::Service<$(&name_snake_case)::$(&name_pascal_case)Impl, Self::Env> {
                self.service($(&program_program_name)::$(&route_id_const_name))
            }
        );
    }
}
