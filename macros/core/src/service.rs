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
use syn::{Ident, ItemImpl, Signature, Type, Visibility};

pub fn gservice(service_impl_tokens: TokenStream2) -> TokenStream2 {
    let mut service_impl = syn::parse2(service_impl_tokens.clone())
        .unwrap_or_else(|err| abort!(err.span(), "Failed to parse service impl: {}", err));

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
            "No handlers found. Try either defining one or removing the macro usage"
        );
    }

    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_params_structs = Vec::with_capacity(service_handlers.len());
    let mut invocation_funcs = Vec::with_capacity(service_handlers.len());
    let mut invocation_dispatches = Vec::with_capacity(service_handlers.len());
    let mut commands_meta_variants = Vec::with_capacity(service_handlers.len());
    let mut queries_meta_variants = Vec::with_capacity(service_handlers.len());

    for (invocation_route, service_handler) in &service_handlers {
        let handler = Func::from(service_handler);
        let handler_generator = HandlerGenerator::from(handler);
        let invocation_func_ident = handler_generator.invocation_func_ident();

        invocation_params_structs.push(handler_generator.params_struct());
        invocation_funcs.push(handler_generator.invocation_func());
        invocation_dispatches.push({
            let invocation_route_bytes = invocation_route.encode();
            let invocation_route_len = invocation_route_bytes.len();
            quote!(
                if #input_ident.starts_with(& [ #(#invocation_route_bytes),* ]) {
                    let output = self.#invocation_func_ident(&#input_ident[#invocation_route_len..]).await;
                    static INVOCATION_ROUTE: [u8; #invocation_route_len] = [ #(#invocation_route_bytes),* ];
                    return [INVOCATION_ROUTE.as_ref(), &output].concat();
                }
            )
        });

        let handler_meta_variant = {
            let params_struct_ident = handler_generator.params_struct_ident();
            let result_type = handler_generator.result_type();
            let invocation_route_ident = Ident::new(invocation_route, Span::call_site());

            quote!(#invocation_route_ident(#params_struct_ident, #result_type))
        };
        if handler_generator.is_query() {
            queries_meta_variants.push(handler_meta_variant);
        } else {
            commands_meta_variants.push(handler_meta_variant);
        }
    }

    service_impl.items.push(syn::parse2(quote!(
            pub async fn handle(&mut self, mut #input_ident: &[u8]) -> Vec<u8> {
                #(#invocation_dispatches)*
                let invocation_path = String::decode(&mut #input_ident).expect("Failed to decode invocation path");
            panic!("Unknown request: {}", invocation_path);
        })).expect("Unable to parse process function"));

    service_impl.items.extend(
        invocation_funcs
            .into_iter()
            .map(|func| syn::parse2(func).expect("Unable to parse invocation function")),
    );

    quote!(
        #service_impl

        impl #service_type_args sails_idl_meta::ServiceMeta for #service_type_path #service_type_constraints {
            fn commands() -> scale_info::MetaType {
                scale_info::MetaType::new::<meta::CommandsMeta>()
            }

            fn queries() -> scale_info::MetaType {
                scale_info::MetaType::new::<meta::QueriesMeta>()
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
        }
    )
}

fn discover_service_handlers(service_impl: &ItemImpl) -> BTreeMap<String, &Signature> {
    shared::discover_invocation_targets(
        service_impl,
        |fn_item| matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some(),
        false,
    )
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

    #[test]
    fn gservice_works() {
        let input = quote! {
            impl SomeService {
                pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                    p1
                }

                pub fn this(&self, p1: bool) -> bool {
                    p1
                }
            }
        };

        let result = gservice(input).to_string();
        let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

        insta::assert_snapshot!(result);
    }

    #[test]
    fn gservice_works_for_lifetimes_and_generics() {
        let input = quote! {
            impl<'a, 'b, T> SomeService<'a, 'b, T> where T : Clone {
                pub fn do_this(&mut self) -> u32 {
                    42
                }
            }
        };

        let result = gservice(input).to_string();
        let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

        insta::assert_snapshot!(result);
    }
}
