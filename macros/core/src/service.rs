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

use crate::{
    sails_paths,
    shared::{self, Func, ImplType},
};
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Expr, GenericArgument, Ident, ImplItemFn, ItemImpl, Path, PathArguments, Result as SynResult,
    Token, Type, TypeParamBound, Visibility, WhereClause, WherePredicate,
};

static mut SERVICE_SPANS: BTreeMap<String, Span> = BTreeMap::new();

pub fn gservice(args: TokenStream, service_impl: TokenStream) -> TokenStream {
    let service_impl = parse_gservice_impl(service_impl);
    ensure_single_gservice_on_impl(&service_impl);
    ensure_single_gservice_by_name(&service_impl);
    gen_gservice_impl(args, service_impl)
}

#[doc(hidden)]
pub fn __gservice_internal(args: TokenStream, service_impl: TokenStream) -> TokenStream {
    let service_impl = parse_gservice_impl(service_impl);
    gen_gservice_impl(args, service_impl)
}

fn parse_gservice_impl(service_impl_tokens: TokenStream) -> ItemImpl {
    syn::parse2(service_impl_tokens).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`gservice` attribute can be applied to impls only: {}",
            err
        )
    })
}

fn ensure_single_gservice_on_impl(service_impl: &ItemImpl) {
    let attr_gservice = service_impl.attrs.iter().find(|attr| {
        attr.meta
            .path()
            .segments
            .last()
            .map(|s| s.ident == "gservice")
            .unwrap_or(false)
    });
    if attr_gservice.is_some() {
        abort!(
            service_impl,
            "multiple `gservice` attributes on the same impl are not allowed",
        )
    }
}

fn ensure_single_gservice_by_name(service_impl: &ItemImpl) {
    let path = shared::impl_type_path(service_impl);
    let type_ident = path.path.segments.last().unwrap().ident.to_string();
    if unsafe { SERVICE_SPANS.get(&type_ident) }.is_some() {
        abort!(
            service_impl,
            "multiple `gservice` attributes on a type with the same name are not allowed"
        )
    }
    unsafe { SERVICE_SPANS.insert(type_ident, service_impl.span()) };
}

fn gen_gservice_impl(args: TokenStream, service_impl: ItemImpl) -> TokenStream {
    let service_args = syn::parse2::<ServiceArgs>(args).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "failed to parse `gservice` attribute arguments: {}",
            err
        )
    });

    let (service_type, service_type_args, service_type_constraints) = {
        let service_type = ImplType::new(&service_impl);
        (
            service_type.path().clone(),
            service_type.args().clone(),
            service_type.constraints().cloned(),
        )
    };

    let service_base_types = service_args.items.iter().flat_map(|item| match item {
        ServiceArg::Extends(paths) => paths,
    });

    let service_handlers = discover_service_handlers(&service_impl);

    if service_handlers.is_empty() && !service_base_types.clone().any(|_| true) {
        abort!(
            service_impl,
            "`gservice` attribute requires impl to define at least one public method or extend another service"
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
                    return Some([INVOCATION_ROUTE.as_ref(), &output].concat());
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

    let service_base_types = service_args.items.iter().flat_map(|item| match item {
        ServiceArg::Extends(paths) => paths,
    });

    let code_for_base_types = service_base_types
        .enumerate()
        .map(|(idx, base_type)| {
            let base_ident = Ident::new(&format!("base_{}", idx), Span::call_site());

            let exposure_as_ref_impl = quote!(
                impl AsRef<< #base_type as sails_rtl::gstd::services::Service>::Exposure> for Exposure< #service_type > {
                    fn as_ref(&self) -> &< #base_type as sails_rtl::gstd::services::Service>::Exposure {
                        &self. #base_ident
                    }
                }
            );

            let base_exposure_member = quote!(
                #base_ident : < #base_type as sails_rtl::gstd::services::Service>::Exposure,
            );

            let base_exposure_instantiation = quote!(
                #base_ident : < #base_type as Clone>::clone(AsRef::< #base_type >::as_ref(&self))
                    .expose( #message_id_ident , #route_ident ),
            );

            let base_exposure_invocation = quote!(
                if let Some(output) = self. #base_ident .try_handle(#input_ident).await {
                    return Some(output);
                }
            );

            let base_service_meta = quote!(sails_rtl::meta::AnyServiceMeta::new::< #base_type >());

            (exposure_as_ref_impl, base_exposure_member, base_exposure_instantiation, base_exposure_invocation, base_service_meta)
        });

    let exposure_as_ref_impls = code_for_base_types
        .clone()
        .map(|(exposure_as_ref_impl, ..)| exposure_as_ref_impl);

    let base_exposures_members = code_for_base_types
        .clone()
        .map(|(_, base_exposure_member, ..)| base_exposure_member);

    let base_exposures_instantiations = code_for_base_types
        .clone()
        .map(|(_, _, base_exposure_instantiation, ..)| base_exposure_instantiation);

    let base_exposures_invocations = code_for_base_types
        .clone()
        .map(|(_, _, _, base_exposure_invocation, ..)| base_exposure_invocation);

    let base_services_meta =
        code_for_base_types.map(|(_, _, _, _, base_service_meta)| base_service_meta);

    let scale_types_path = sails_paths::scale_types_path();
    let scale_codec_path = sails_paths::scale_codec_path();
    let scale_info_path = sails_paths::scale_info_path();

    let unexpected_route_panic =
        shared::generate_unexpected_input_panic(&input_ident, "Unknown request");

    quote!(
        #service_impl

        pub struct Exposure<T> {
            #message_id_ident : sails_rtl::MessageId,
            #route_ident : &'static [u8],
            #inner_ident : T,
            #( #base_exposures_members )*
        }

        impl #service_type_args Exposure<#service_type> #service_type_constraints {
            #( #exposure_funcs )*

            pub async fn handle(&mut self, mut #input_ident: &[u8]) -> Vec<u8> {
                self.try_handle( #input_ident ).await.unwrap_or_else(|| {
                    #unexpected_route_panic
                })
            }

            pub async fn try_handle(&mut self, #input_ident : &[u8]) -> Option<Vec<u8>> {
                #( #invocation_dispatches )*
                #( #base_exposures_invocations )*
                None
            }

            #(#invocation_funcs)*
        }

        impl #service_type_args sails_rtl::gstd::services::Exposure for Exposure< #service_type > #service_type_constraints {
            fn message_id(&self) -> sails_rtl::MessageId {
                self. #message_id_ident
            }

            fn route(&self) -> &'static [u8] {
                self. #route_ident
            }
        }

        #( #exposure_as_ref_impls )*

        impl #service_type_args sails_rtl::gstd::services::Service for #service_type #service_type_constraints {
            type Exposure = Exposure< #service_type >;

            fn expose(self, #message_id_ident : sails_rtl::MessageId, #route_ident : &'static [u8]) -> Self::Exposure {
                Self::Exposure {
                    #message_id_ident ,
                    #route_ident ,
                    #( #base_exposures_instantiations )*
                    #inner_ident : self,
                }
            }
        }

        impl #service_type_args sails_rtl::meta::ServiceMeta for #service_type #service_type_constraints {
            fn commands() -> #scale_info_path ::MetaType {
                #scale_info_path ::MetaType::new::<meta::CommandsMeta>()
            }

            fn queries() -> #scale_info_path ::MetaType {
                #scale_info_path ::MetaType::new::<meta::QueriesMeta>()
            }

            fn events() -> #scale_info_path ::MetaType {
                #scale_info_path ::MetaType::new::<meta::EventsMeta>()
            }

            fn base_services() -> impl Iterator<Item = sails_rtl::meta::AnyServiceMeta> {
                [
                    #( #base_services_meta ),*
                ].into_iter()
            }
        }

        use #scale_types_path ::Decode as __ServiceDecode;
        use #scale_types_path ::Encode as __ServiceEncode;
        use #scale_types_path ::TypeInfo as __ServiceTypeInfo;

        #(
            #[derive(__ServiceDecode, __ServiceTypeInfo)]
            #[codec(crate = #scale_codec_path )]
            #[scale_info(crate = #scale_info_path )]
            #invocation_params_structs
        )*

        mod meta {
            use super::*;

            #[derive(__ServiceTypeInfo)]
            #[scale_info(crate = #scale_info_path )]
            pub enum CommandsMeta {
                #(#commands_meta_variants),*
            }

            #[derive(__ServiceTypeInfo)]
            #[scale_info(crate = #scale_info_path )]
            pub enum QueriesMeta {
                #(#queries_meta_variants),*
            }

            #[derive(__ServiceTypeInfo)]
            #[scale_info(crate = #scale_info_path )]
            pub enum #no_events_type {}

            pub type EventsMeta = #events_type;
        }
    )
}

struct ServiceArgs {
    items: Punctuated<ServiceArg, Token![,]>,
}

impl Parse for ServiceArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(Self {
            items: input.parse_terminated(ServiceArg::parse, Token![,])?,
        })
    }
}

#[derive(Debug)]
enum ServiceArg {
    Extends(Vec<Path>),
}

impl Parse for ServiceArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let values = input.parse::<Expr>()?;
        match ident.to_string().as_str() {
            "extends" => {
                if let Expr::Path(path_expr) = values {
                    // Check path_expr.attrs is empty and qself is none
                    return Ok(Self::Extends(vec![path_expr.path]));
                } else if let Expr::Array(array_expr) = values {
                    let mut paths = Vec::new();
                    for item_expr in array_expr.elems {
                        if let Expr::Path(path_expr) = item_expr {
                            paths.push(path_expr.path);
                        } else {
                            abort!(
                                item_expr,
                                "unexpected value for `extends` argument: {}",
                                item_expr.to_token_stream()
                            )
                        }
                    }
                    return Ok(Self::Extends(paths));
                }
                abort!(
                    ident,
                    "unexpected value for `extends` argument: {}",
                    values.to_token_stream()
                )
            }
            _ => abort!(ident, "unknown argument: {}", ident),
        }
    }
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
                if last_segment.ident == "EventNotifier" {
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

    fn params_struct(&self) -> TokenStream {
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

    fn invocation_func(&self) -> TokenStream {
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
            impl<T> SomeService<T> where T: EventNotifier<events::SomeEvents> {
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
