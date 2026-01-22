use crate::events_generator::EventsModuleGenerator;
use crate::helpers::*;
use crate::mock_generator::MockGenerator;
use crate::type_generators::{TopLevelTypeGenerator, generate_type_decl_with_path};
use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{ast, visitor, visitor::Visitor};
use std::collections::HashMap;

/// Generates a service module with trait and struct implementation
pub(crate) struct ServiceGenerator<'ast> {
    service_name: &'ast str,
    sails_path: &'ast str,
    external_types: &'ast HashMap<&'ast str, &'ast str>,
    mocks_feature_name: Option<&'ast str>,
    trait_tokens: Tokens,
    impl_tokens: Tokens,
    io_tokens: Tokens,
    events_tokens: Tokens,
    types_tokens: Tokens,
    mocks_tokens: Tokens,
    interface_id: sails_idl_meta::InterfaceId,
    entry_ids: HashMap<&'ast str, u16>,
    no_derive_traits: bool,
}

impl<'ast> ServiceGenerator<'ast> {
    pub(crate) fn new(
        service_name: &'ast str,
        sails_path: &'ast str,
        external_types: &'ast HashMap<&'ast str, &'ast str>,
        mocks_feature_name: Option<&'ast str>,
        interface_id: sails_idl_meta::InterfaceId,
        no_derive_traits: bool,
    ) -> Self {
        Self {
            service_name,
            sails_path,
            external_types,
            mocks_feature_name,
            trait_tokens: Tokens::new(),
            impl_tokens: Tokens::new(),
            io_tokens: Tokens::new(),
            events_tokens: Tokens::new(),
            types_tokens: Tokens::new(),
            mocks_tokens: Tokens::new(),
            interface_id,
            entry_ids: HashMap::new(),
            no_derive_traits,
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let service_name_snake = &self.service_name.to_case(Case::Snake);
        let mock_tokens = if let Some(mocks_feature_name) = self.mocks_feature_name {
            quote! {
                $['\n']
                #[cfg(feature = $(quoted(mocks_feature_name)))]
                #[cfg(not(target_arch = "wasm32"))]
                pub mod mockall {
                    use super::*;
                    use $(self.sails_path)::mockall::*;
                    $(self.mocks_tokens)
                }
            }
        } else {
            quote!()
        };

        let bytes = self.interface_id.as_bytes().iter().copied();
        let interface_id_tokens = quote! {
            const INTERFACE_ID: $(self.sails_path)::InterfaceId = $(self.sails_path)::InterfaceId::from_bytes_8([ $(for b in bytes join (, ) => $b) ]);
        };

        quote! {
            $['\n']
            pub mod $service_name_snake {
                use super::*;

                $(self.types_tokens)

                pub trait $(self.service_name) {
                    type Env: $(self.sails_path)::client::GearEnv;
                    $(self.trait_tokens)
                }

                pub struct $(self.service_name)Impl;

                impl $(self.sails_path)::client::Identifiable for $(self.service_name)Impl {
                    $interface_id_tokens
                }

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

                $(mock_tokens)
            }
        }
    }
}

// using quote_in instead of tokens.append
impl<'ast> Visitor<'ast> for ServiceGenerator<'ast> {
    fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
        let (mut commands, mut queries): (Vec<_>, Vec<_>) = service
            .funcs
            .iter()
            .partition(|f| f.kind != ast::FunctionKind::Query);

        commands.sort_by_key(|f| f.name.to_lowercase());
        queries.sort_by_key(|f| f.name.to_lowercase());

        for (entry_id, func) in commands.into_iter().chain(queries.into_iter()).enumerate() {
            self.entry_ids.insert(func.name.as_str(), entry_id as u16);
        }

        for (idx, event) in service.events.iter().enumerate() {
            self.entry_ids.insert(event.name.as_str(), idx as u16);
        }

        visitor::accept_service_unit(service, self);

        for service_ident in &service.extends {
            let name = &service_ident.name;
            let method_name = name.to_case(Case::Snake);
            let impl_name = name.to_case(Case::Pascal);
            let mod_name = name.to_case(Case::Snake);

            quote_in! { self.trait_tokens =>
                $['\r'] fn $(&method_name)(&self) -> $(self.sails_path)::client::Service<super::$(mod_name.as_str())::$(impl_name.as_str())Impl, Self::Env>;
            };

            quote_in! { self.impl_tokens =>
                $['\r'] fn $(&method_name)(&self) -> $(self.sails_path)::client::Service<super::$(mod_name.as_str())::$(impl_name.as_str())Impl, Self::Env> {
                    self.base_service()
                }
            };
        }

        let mut mock_gen = MockGenerator::new(self.service_name, self.sails_path);
        mock_gen.visit_service_unit(service);
        self.mocks_tokens.extend(mock_gen.finalize());

        if !service.events.is_empty() {
            let mut events_mod_gen = EventsModuleGenerator::new(
                self.service_name,
                self.sails_path,
                self.entry_ids.clone(),
            );
            events_mod_gen.visit_service_unit(service);
            self.events_tokens = events_mod_gen.finalize();
        }
    }

    fn visit_type(&mut self, t: &'ast ast::Type) {
        if self.external_types.contains_key(t.name.as_str()) {
            return;
        }

        let mut type_gen =
            TopLevelTypeGenerator::new(&t.name, self.sails_path, self.no_derive_traits);
        type_gen.visit_type(t);
        self.types_tokens.extend(type_gen.finalize());
    }

    fn visit_service_func(&mut self, func: &'ast ast::ServiceFunc) {
        let self_ref = if func.kind == ast::FunctionKind::Query {
            "&self"
        } else {
            "&mut self"
        };
        let fn_name = &func.name;
        let fn_name_snake = &fn_name.to_case(Case::Snake);

        let params_with_types = &fn_args_with_types_path(&func.params, "");
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

        let output_type_decl_code = if let Some(throws_type) = &func.throws {
            let ok_type = generate_type_decl_with_path(&func.output, "super");
            let err_type = generate_type_decl_with_path(throws_type, "super");
            format!("super::Result<{ok_type}, {err_type}>")
        } else {
            generate_type_decl_with_path(&func.output, "super")
        };

        let params_with_types_super = &fn_args_with_types_path(&func.params, "super");
        let entry_id = self.entry_ids.get(func.name.as_str()).copied().unwrap_or(0);
        let is_throws = if func.throws.is_some() {
            "true"
        } else {
            "false"
        };

        quote_in! { self.io_tokens =>
            $(self.sails_path)::io_struct_impl!($fn_name ($params_with_types_super) -> $output_type_decl_code, $entry_id, <super::$(self.service_name)Impl as $(self.sails_path)::client::Identifiable>::INTERFACE_ID, throws $is_throws);
        };
    }
}
