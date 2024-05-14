// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Supporting functions and structures for the `gservice` macro.

use crate::shared::{self, Func, ImplType};
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::quote;
use std::collections::BTreeMap;
use syn::{
    GenericArgument, Ident, ImplItemFn, ItemImpl, Path, PathArguments, Type, TypeParamBound,
    Visibility, WhereClause, WherePredicate,
};

pub fn gservice(service_impl_tokens: TokenStream2) -> TokenStream2 {
    let service_impl = syn::parse2(service_impl_tokens).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`gservice` attribute can be applied to impls only: {}",
            err
        )
    });

    let (service_type_path, service_type_args, service_type_constraints) = {
        let service_type = ImplType::new(&service_impl);
        (
            service_type.path().clone(),
            service_type.args().clone(),
            service_type.constraints().cloned(),
        )
    };

    let service_handlers = discover_service_handlers(&service_impl);

    if service_handlers.is_empty() {
        abort!(
            service_impl,
            "`gservice` attribute requires impl to define at least one public method"
        );
    }

    let no_events_type = Path::from(Ident::new("NoEvents", Span::call_site()));
    let events_type = service_type_constraints
        .as_ref()
        .and_then(discover_service_events_type)
        .unwrap_or(&no_events_type);

    let inner_ident = Ident::new("inner", Span::call_site());
    let input_ident = Ident::new("input", Span::call_site());

    let mut exposure_funcs = Vec::with_capacity(service_handlers.len());
    let mut invocation_params_structs = Vec::with_capacity(service_handlers.len());
    let mut invocation_funcs = Vec::with_capacity(service_handlers.len());
    let mut invocation_dispatches = Vec::with_capacity(service_handlers.len());
    let mut commands_meta_variants = Vec::with_capacity(service_handlers.len());
    let mut queries_meta_variants = Vec::with_capacity(service_handlers.len());

    for (handler_route, (handler_fn, ..)) in &service_handlers {
        let handler_fn = &handler_fn.sig;
        let handler_func = Func::from(handler_fn);
        let handler_generator = HandlerGenerator::from(handler_func.clone());
        let invocation_func_ident = handler_generator.invocation_func_ident();

        exposure_funcs.push({
            let handler_ident = handler_func.ident();
            let handler_params = handler_func.params().iter().map(|item| item.0);
            let handler_await_token = handler_func.is_async().then(|| quote!(.await));
            quote!(
                pub #handler_fn {
                    let exposure_scope = sails_rtl::gstd::services::ExposureCallScope::new(self);
                    self. #inner_ident . #handler_ident (#(#handler_params),*) #handler_await_token
                }
            )
        });
        invocation_params_structs.push(handler_generator.params_struct());
        invocation_funcs.push(handler_generator.invocation_func());
        invocation_dispatches.push({
            let handler_route_bytes = handler_route.encode();
            let handler_route_len = handler_route_bytes.len();
            quote!(
                if #input_ident.starts_with(& [ #(#handler_route_bytes),* ]) {
                    let output = self.#invocation_func_ident(&#input_ident[#handler_route_len..]).await;
                    static INVOCATION_ROUTE: [u8; #handler_route_len] = [ #(#handler_route_bytes),* ];
                    return [INVOCATION_ROUTE.as_ref(), &output].concat();
                }
            )
        });

        let handler_meta_variant = {
            let params_struct_ident = handler_generator.params_struct_ident();
            let result_type = handler_generator.result_type();
            let handler_route_ident = Ident::new(handler_route, Span::call_site());

            quote!(#handler_route_ident(#params_struct_ident, #result_type))
        };
        if handler_generator.is_query() {
            queries_meta_variants.push(handler_meta_variant);
        } else {
            commands_meta_variants.push(handler_meta_variant);
        }
    }

    let message_id_ident = Ident::new("message_id", Span::call_site());
    let route_ident = Ident::new("route", Span::call_site());

    quote!(
        #service_impl

        pub struct Exposure<T> {
            #message_id_ident : sails_rtl::MessageId,
            #route_ident : &'static [u8],
            #inner_ident : T,
        }

        impl #service_type_args Exposure<#service_type_path> #service_type_constraints {
            #(#exposure_funcs)*

            pub async fn handle(&mut self, mut #input_ident: &[u8]) -> Vec<u8> {
                #(#invocation_dispatches)*
                let invocation_path = String::decode(&mut #input_ident).expect("Failed to decode invocation path");
                panic!("Unknown request: {}", invocation_path);
            }

            #(#invocation_funcs)*
        }

        impl #service_type_args sails_rtl::gstd::services::Exposure for Exposure<#service_type_path> #service_type_constraints {
            fn message_id(&self) -> sails_rtl::MessageId {
                self. #message_id_ident
            }

            fn route(&self) -> &'static [u8] {
                self. #route_ident
            }
        }

        impl #service_type_args sails_rtl::gstd::services::Service for #service_type_path #service_type_constraints {
            type Exposure = Exposure< #service_type_path >;

            fn expose(self, #message_id_ident : sails_rtl::MessageId, #route_ident : &'static [u8]) -> Self::Exposure {
                Self::Exposure { #message_id_ident , #route_ident , inner: self }
            }
        }

        impl #service_type_args sails_rtl::meta::ServiceMeta for #service_type_path #service_type_constraints {
            fn commands() -> scale_info::MetaType {
                scale_info::MetaType::new::<meta::CommandsMeta>()
            }

            fn queries() -> scale_info::MetaType {
                scale_info::MetaType::new::<meta::QueriesMeta>()
            }

            fn events() -> scale_info::MetaType {
                scale_info::MetaType::new::<meta::EventsMeta>()
            }
        }

        #(#[derive(Decode, TypeInfo)] #invocation_params_structs)*

        mod meta {
            use super::*;

            #[derive(TypeInfo)]
            pub enum CommandsMeta {
                #(#commands_meta_variants),*
            }

            #[derive(TypeInfo)]
            pub enum QueriesMeta {
                #(#queries_meta_variants),*
            }

            #[derive(TypeInfo)]
            pub enum #no_events_type {}

            pub type EventsMeta = #events_type;
        }
    )
}

fn discover_service_handlers(service_impl: &ItemImpl) -> BTreeMap<String, (&ImplItemFn, usize)> {
    shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    })
}

fn discover_service_events_type(where_clause: &WhereClause) -> Option<&Path> {
    let event_types = where_clause
        .predicates
        .iter()
        .filter_map(|predicate| {
            if let WherePredicate::Type(predicate) = predicate {
                return Some(&predicate.bounds);
            }
            None
        })
        .flatten()
        .filter_map(|bound| {
            if let TypeParamBound::Trait(trait_bound) = bound {
                let last_segment = trait_bound.path.segments.last()?;
                if last_segment.ident == "EventTrigger" {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if args.args.len() == 1 {
                            if let GenericArgument::Type(Type::Path(type_path)) =
                                args.args.first().unwrap()
                            {
                                return Some(&type_path.path);
                            }
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>();
    if event_types.len() > 1 {
        abort!(
            where_clause,
            "Multiple event types found. Please specify only one event type"
        );
    }
    event_types.first().copied()
}

struct HandlerGenerator<'a> {
    handler: Func<'a>,
}

impl<'a> HandlerGenerator<'a> {
    fn from(handler: Func<'a>) -> Self {
        Self { handler }
    }

    fn params_struct_ident(&self) -> Ident {
        Ident::new(
            &format!(
                "__{}Params",
                self.handler.ident().to_string().to_case(Case::Pascal)
            ),
            Span::call_site(),
        )
    }

    fn result_type(&self) -> Type {
        self.handler.result().clone()
    }

    fn handler_func_ident(&self) -> Ident {
        self.handler.ident().clone()
    }

    fn invocation_func_ident(&self) -> Ident {
        Ident::new(
            &format!("__{}", self.handler_func_ident()),
            Span::call_site(),
        )
    }

    fn is_query(&self) -> bool {
        self.handler
            .receiver()
            .map_or(true, |r| r.mutability.is_none())
    }

    fn params_struct(&self) -> TokenStream2 {
        let params_struct_ident = self.params_struct_ident();
        let params_struct_members = self.handler.params().iter().map(|item| {
            let arg_ident = item.0;
            let arg_type = item.1;
            quote!(#arg_ident: #arg_type)
        });

        quote!(
            pub struct #params_struct_ident {
                #(#params_struct_members),*
            }
        )
    }

    fn invocation_func(&self) -> TokenStream2 {
        let invocation_func_ident = self.invocation_func_ident();
        let receiver = self.handler.receiver();
        let params_struct_ident = self.params_struct_ident();
        let handler_func_ident = self.handler_func_ident();
        let handler_func_params = self.handler.params().iter().map(|item| {
            let param_ident = item.0;
            quote!(request.#param_ident)
        });

        let await_token = self.handler.is_async().then(|| quote!(.await));

        quote!(
            async fn #invocation_func_ident(#receiver, mut input: &[u8]) -> Vec<u8>
            {
                let request = #params_struct_ident::decode(&mut input).expect("Failed to decode request");
                let result = self.#handler_func_ident(#(#handler_func_params),*)#await_token;
                return result.encode();
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    #[test]
    fn events_type_is_discovered_via_where_clause() {
        let input = quote! {
            impl<T> SomeService<T> where T: EventTrigger<events::SomeEvents> {
                pub async fn do_this(&mut self) -> u32 {
                    42
                }
            }
        };
        let service_impl = syn::parse2(input).unwrap();
        let item_type = ImplType::new(&service_impl);

        let result = discover_service_events_type(item_type.constraints().unwrap());

        assert_eq!(
            result.unwrap().to_token_stream().to_string(),
            quote!(events::SomeEvents).to_string()
        );
    }
}
