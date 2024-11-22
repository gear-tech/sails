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
#![allow(unused_variables)] // temporary
use crate::{
    sails_paths,
    shared::{self, Func},
};
use args::ServiceArgs;
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, Ident, ImplItemFn, Index,
    ItemImpl, Lifetime, Path, Type, Visibility,
};

mod args;

pub fn gservice(args: TokenStream, service_impl: TokenStream) -> TokenStream {
    let service_impl = parse_gservice_impl(service_impl);
    ensure_single_gservice_on_impl(&service_impl);
    generate_gservice(args, service_impl)
}

#[doc(hidden)]
pub fn __gservice_internal(args: TokenStream, service_impl: TokenStream) -> TokenStream {
    let service_impl = parse_gservice_impl(service_impl);
    generate_gservice(args, service_impl)
}

fn parse_gservice_impl(service_impl_tokens: TokenStream) -> ItemImpl {
    syn::parse2(service_impl_tokens).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`service` attribute can be applied to impls only: {}",
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
            .map(|s| s.ident == "service")
            .unwrap_or(false)
    });
    if attr_gservice.is_some() {
        abort!(
            service_impl,
            "multiple `service` attributes on the same impl are not allowed",
        )
    }
}

fn generate_gservice(args: TokenStream, service_impl: ItemImpl) -> TokenStream {
    let service_args = syn::parse2::<ServiceArgs>(args).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "failed to parse `service` attribute arguments: {}",
            err
        )
    });
    let sails_path = service_args.sails_path();
    let scale_codec_path = sails_paths::scale_codec_path(&sails_path);
    let scale_info_path = sails_paths::scale_info_path(&sails_path);

    let (service_type_path, service_type_args, service_ident) = shared::impl_type(&service_impl);
    let (generics, service_type_constraints) = shared::impl_constraints(&service_impl);

    let service_handlers = discover_service_handlers(&service_impl);

    if service_handlers.is_empty() && service_args.base_types().is_empty() {
        abort!(
            service_impl,
            "`service` attribute requires impl to define at least one public method or extend another service"
        );
    }

    let events_type = service_args.events_type().as_ref();

    let mut service_impl = service_impl.clone();
    if let Some(events_type) = events_type {
        service_impl.items.push(parse_quote!(
            fn notify_on(&mut self, event: #events_type ) -> #sails_path::errors::Result<()>  {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let self_ptr = self as *const _ as usize;
                    let event_listeners = #sails_path::gstd::events::event_listeners().lock();
                    if let Some(event_listener_ptr) = event_listeners.get(&self_ptr) {
                        let event_listener =
                            unsafe { &mut *(*event_listener_ptr as *mut Box<dyn FnMut(& #events_type )>) };
                        core::mem::drop(event_listeners);
                        event_listener(&event);
                    }
                }
                #sails_path::gstd::events::__notify_on(event)
            }
        ));
    }

    let meta_module_name = format!("{}_meta", service_ident.to_string().to_case(Case::Snake));
    let meta_module_ident = Ident::new(&meta_module_name, Span::call_site());

    let inner_ident = Ident::new("inner", Span::call_site());
    let inner_ptr_ident = Ident::new("inner_ptr", Span::call_site());
    let input_ident = Ident::new("input", Span::call_site());

    let mut exposure_funcs = Vec::with_capacity(service_handlers.len());
    let mut invocation_params_structs = Vec::with_capacity(service_handlers.len());
    let mut invocation_funcs = Vec::with_capacity(service_handlers.len());
    let mut invocation_dispatches = Vec::with_capacity(service_handlers.len());
    let mut commands_meta_variants = Vec::with_capacity(service_handlers.len());
    let mut queries_meta_variants = Vec::with_capacity(service_handlers.len());

    for (handler_route, (handler_fn, ..)) in &service_handlers {
        // We propagate only known attributes as we don't know the consequences of unknown ones
        let handler_allow_attrs = handler_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("allow"));
        let handler_docs_attrs = handler_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"));

        let handler_fn = &handler_fn.sig;
        let handler_func = Func::from(handler_fn);
        let handler_generator = HandlerGenerator::from(handler_func.clone());
        let invocation_func_ident = handler_generator.invocation_func_ident();

        exposure_funcs.push({
            let handler_ident = handler_func.ident();
            let handler_params = handler_func.params().iter().map(|item| item.0);
            let handler_await_token = handler_func.is_async().then(|| quote!(.await));
            quote!(
                #( #handler_allow_attrs )*
                pub #handler_fn {
                    let exposure_scope = #sails_path::gstd::services::ExposureCallScope::new(self);
                    self. #inner_ident . #handler_ident (#(#handler_params),*) #handler_await_token
                }
            )
        });
        invocation_params_structs.push(handler_generator.params_struct());
        invocation_funcs.push(handler_generator.invocation_func(&meta_module_ident, &sails_path));
        invocation_dispatches.push({
            let handler_route_bytes = handler_route.encode();
            let handler_route_len = handler_route_bytes.len();
            quote!(
                if #input_ident.starts_with(& [ #(#handler_route_bytes),* ]) {
                    let (output, value) = self.#invocation_func_ident(&#input_ident[#handler_route_len..]).await;
                    static INVOCATION_ROUTE: [u8; #handler_route_len] = [ #(#handler_route_bytes),* ];
                    return Some(([INVOCATION_ROUTE.as_ref(), &output].concat(), value));
                }
            )
        });

        let handler_meta_variant = {
            let params_struct_ident = handler_generator.params_struct_ident();
            let result_type = handler_generator.result_type();
            let handler_route_ident = Ident::new(handler_route, Span::call_site());

            quote!(
                #( #handler_docs_attrs )*
                #handler_route_ident(#params_struct_ident, #result_type)
            )
        };
        if handler_generator.is_query() {
            queries_meta_variants.push(handler_meta_variant);
        } else {
            commands_meta_variants.push(handler_meta_variant);
        }
    }

    let message_id_ident = Ident::new("message_id", Span::call_site());
    let route_ident = Ident::new("route", Span::call_site());

    let code_for_base_types = service_args.base_types().iter()
        .enumerate()
        .map(|(idx, base_type)| {
            let base_ident = Ident::new(&format!("base_{}", idx), Span::call_site());
            let as_base_ident = Ident::new(&format!("as_base_{}", idx), Span::call_site());

            let base_exposure_accessor = quote!(
                pub fn #as_base_ident (&self) -> &< #base_type as #sails_path::gstd::services::Service>::Exposure {
                    &self. #base_ident
                }
            );

            let base_exposure_member = quote!(
                #base_ident : < #base_type as #sails_path::gstd::services::Service>::Exposure,
            );

            let base_exposure_instantiation = quote!(
                #base_ident : < #base_type as Clone>::clone(AsRef::< #base_type >::as_ref( #inner_ident ))
                    .expose( #message_id_ident , #route_ident ),
            );

            let base_exposure_invocation = quote!(
                if let Some((output, value)) = self. #base_ident .try_handle(#input_ident).await {
                    return Some((output, value));
                }
            );

            let base_service_meta = quote!(#sails_path::meta::AnyServiceMeta::new::< #base_type >());

            (base_exposure_accessor, base_exposure_member, base_exposure_instantiation, base_exposure_invocation, base_service_meta)
        });

    let base_exposure_accessors = code_for_base_types
        .clone()
        .map(|(base_exposure_accessor, ..)| base_exposure_accessor);

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

    let lifetimes = shared::extract_lifetime_names(&service_type_args);
    let exposure_set_event_listener_code = events_type.map(|event_type_path| {
        // get non conflicting lifetime name
        let mut lt = "__elg".to_owned();
        while lifetimes.contains(&lt) {
            lt = format!("_{}", lt);
        }
        let lifetime_name = format!("'{0}", lt);
        generate_exposure_set_event_listener(
            &sails_path,
            event_type_path,
            Lifetime::new(&lifetime_name, Span::call_site()),
        )
    });

    let exposure_name = format!(
        "{}Exposure",
        service_ident.to_string().to_case(Case::Pascal)
    );
    let exposure_type_path = Path::from(Ident::new(&exposure_name, Span::call_site()));

    let exposure_drop_code =
        events_type.map(|_| generate_exposure_drop(&sails_path, &exposure_type_path));

    let no_events_type = Path::from(Ident::new("NoEvents", Span::call_site()));
    let events_type = events_type.unwrap_or(&no_events_type);

    let unexpected_route_panic =
        shared::generate_unexpected_input_panic(&input_ident, "Unknown request", &sails_path);

    let mut exposure_lifetimes: Punctuated<Lifetime, Comma> = Punctuated::new();
    if !service_args.base_types().is_empty() {
        for lt in lifetimes.iter().map(|lt| {
            let lt = format!("'{lt}");
            Lifetime::new(&lt, Span::call_site())
        }) {
            exposure_lifetimes.push(lt);
        }
    };

    let exposure_generic_args = if exposure_lifetimes.is_empty() {
        quote! { T }
    } else {
        quote! { #exposure_lifetimes, T }
    };
    let exposure_args = if exposure_lifetimes.is_empty() {
        quote! { #service_type_path }
    } else {
        quote! { #exposure_lifetimes, #service_type_path }
    };

    // We propagate only known attributes as we don't know the consequences of unknown ones
    let exposure_allow_attrs = service_impl
        .attrs
        .iter()
        .filter(|attr| matches!(attr.path().get_ident(), Some(ident) if ident == "allow"));

    // V2
    let trait_ident = format_ident!("{}ImplTrait", service_ident);

    let mut trait_funcs = Vec::with_capacity(service_handlers.len());
    let mut trait_funcs_impl = Vec::with_capacity(service_handlers.len());
    let mut invocation_dispatches = Vec::with_capacity(service_handlers.len());
    for (handler_route, (handler_fn, ..)) in &service_handlers {
        let handler_allow_attrs = handler_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("allow"));
        let handler_docs_attrs = handler_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"));

        let handler_sig = &handler_fn.sig;
        let handler_func = Func::from(handler_sig);

        trait_funcs.push(quote! {
            #handler_sig;
        });

        let handler_ident = handler_func.ident();
        let handler_params = handler_func.params().iter().map(|item| item.0);
        let handler_await_token = handler_func.is_async().then(|| quote!(.await));
        trait_funcs_impl.push({
            quote!(
                #( #handler_allow_attrs )*
                #handler_sig {
                    let exposure_scope = #sails_path::gstd::services::ExposureCallScope::new2(self);
                    self. #inner_ident . #handler_ident (#(#handler_params),*) #handler_await_token
                }
            )
        });

        let handler_generator = HandlerGenerator::from(handler_func.clone());
        let invocation_func = handler_generator.invocation_func_v2(&meta_module_ident, &sails_path);

        invocation_dispatches.push({
            quote!(
                #handler_route => {
                    #invocation_func
                    Some((#sails_path::Encode::encode(&(#handler_route, &result)), value))
                }
            )
        });
    }

    let mut exposure_lifetimes: Punctuated<Lifetime, Comma> = Punctuated::new();
    for lt in lifetimes.iter().map(|lt| {
        let lt = format!("'{lt}");
        Lifetime::new(&lt, Span::call_site())
    }) {
        exposure_lifetimes.push(lt);
    }
    let trait_lifetimes = if !exposure_lifetimes.is_empty() {
        quote! { < #exposure_lifetimes> }
    } else {
        quote! {}
    };

    // base v2
    let single_base_type = service_args.base_types().len() == 1;

    let mut base_expo_types = Vec::with_capacity(service_args.base_types().len());
    let mut base_types_funcs = Vec::with_capacity(service_args.base_types().len());
    let mut base_types_impl = Vec::with_capacity(service_args.base_types().len());
    let mut base_exposure_instantiation = Vec::with_capacity(service_args.base_types().len());

    service_args.base_types().iter()
        .enumerate()
        .for_each(|(idx, base_type)| {
            let as_base_ident = format_ident!("as_base_{}", idx);
            let base_expo_type = quote! { #sails_path::gstd::services::ServiceExposure< #base_type, () > };

            base_expo_types.push(quote! {
                #base_expo_type
            });

            base_types_funcs.push(quote!{
                fn #as_base_ident (&self) -> & #base_expo_type;
            });

            let extend_ref = if single_base_type {
                quote! { &self.extend }
            } else {
                let base_idx = Index::from(idx);
                quote! { &self.extend.#base_idx }
            };

            base_types_impl.push(quote!{
                fn #as_base_ident (&self) -> & #base_expo_type {
                    #extend_ref
                }
            });

            base_exposure_instantiation.push(quote!(
                < #base_type as Clone>::clone(AsRef::< #base_type >::as_ref( &self )).expose( #message_id_ident , #route_ident )
            ));
        });

    let base_type = if single_base_type {
        let single_type = &base_expo_types[0];
        quote! { #single_type }
    } else {
        quote! { ( #( #base_expo_types ),* ) }
    };
    let base_inst: TokenStream = quote! { ( #( #base_exposure_instantiation ),* ) };

    let expo_type =
        quote! { #sails_path::gstd::services::ServiceExposure< #service_type_path, #base_type > };

    quote!(
        #service_impl

        #[allow(async_fn_in_trait)]
        pub trait #trait_ident #trait_lifetimes {
            #( #trait_funcs )*

            #( #base_types_funcs )*
        }

        impl #generics #trait_ident #trait_lifetimes for #expo_type #service_type_constraints {
            #( #trait_funcs_impl )*

            #( #base_types_impl )*
        }

        impl #generics #sails_path::gstd::services::ServiceHandle for #service_type_path #service_type_constraints {
            async fn try_handle(&mut self, #input_ident : &[u8]) -> Option<(Vec<u8>, u128)> {
                let mut __input = #input_ident;
                let route: String = #sails_path::Decode::decode(&mut __input).ok()?;
                match route.as_str() {
                    #( #invocation_dispatches )*
                    _ => None,
                }
            }
        }

        impl #generics #sails_path::gstd::services::Service for #service_type_path #service_type_constraints {
            type Exposure = #expo_type;
            type Extend = #base_type;

            fn expose(self, #message_id_ident : #sails_path::MessageId, #route_ident : &'static [u8]) -> Self::Exposure {
                let extend = #base_inst;
                Self::Exposure::new(#message_id_ident, #route_ident, self, extend)
            }
        }

        impl #generics #sails_path::meta::ServiceMeta for #service_type_path #service_type_constraints {
            fn commands() -> #scale_info_path ::MetaType {
                #scale_info_path ::MetaType::new::<#meta_module_ident::CommandsMeta>()
            }

            fn queries() -> #scale_info_path ::MetaType {
                #scale_info_path ::MetaType::new::<#meta_module_ident::QueriesMeta>()
            }

            fn events() -> #scale_info_path ::MetaType {
                #scale_info_path ::MetaType::new::<#meta_module_ident::EventsMeta>()
            }

            fn base_services() -> impl Iterator<Item = #sails_path::meta::AnyServiceMeta> {
                [
                    #( #base_services_meta ),*
                ].into_iter()
            }
        }

        mod #meta_module_ident {
            use super::*;
            use #sails_path::{Decode, TypeInfo};

            #(
                #[derive(Decode, TypeInfo)]
                #[codec(crate = #scale_codec_path )]
                #[scale_info(crate = #scale_info_path )]
                #invocation_params_structs
            )*

            #[derive(TypeInfo)]
            #[scale_info(crate = #scale_info_path)]
            pub enum CommandsMeta {
                #(#commands_meta_variants),*
            }

            #[derive(TypeInfo)]
            #[scale_info(crate = #scale_info_path)]
            pub enum QueriesMeta {
                #(#queries_meta_variants),*
            }

            #[derive(TypeInfo)]
            #[scale_info(crate = #scale_info_path )]
            pub enum #no_events_type {}

            pub type EventsMeta = #events_type;
        }
    )
}

fn generate_exposure_drop(sails_path: &Path, exposure_type_path: &Path) -> TokenStream {
    quote!(
        #[cfg(not(target_arch = "wasm32"))]
        impl<T> Drop for #exposure_type_path <T> {
            fn drop(&mut self) {
                let service_ptr = self.inner_ptr as usize;
                let mut event_listeners = #sails_path::gstd::events::event_listeners().lock();
                if event_listeners.remove(&service_ptr).is_some() {
                    panic!("there should be no any event listeners left by this time");
                }
            }
        }
    )
}

fn generate_exposure_set_event_listener(
    sails_path: &Path,
    events_type: &Path,
    lifetime: Lifetime,
) -> TokenStream {
    quote!(
        #[cfg(not(target_arch = "wasm32"))]
        // Immutable so one can set it via AsRef when used with extending
        pub fn set_event_listener<#lifetime>(
            &self,
            listener: impl FnMut(& #events_type ) + #lifetime,
        ) -> #sails_path::gstd::events::EventListenerGuard<#lifetime> {
            if core::mem::size_of_val(self.inner.as_ref()) == 0 {
                panic!("setting event listener on a zero-sized service is not supported for now");
            }
            let service_ptr = self.inner_ptr as usize;
            let listener: Box<dyn FnMut(& #events_type )> = Box::new(listener);
            let listener = Box::new(listener);
            let listener_ptr = Box::into_raw(listener) as usize;
            #sails_path::gstd::events::EventListenerGuard::new(service_ptr, listener_ptr)
        }
    )
}

fn discover_service_handlers(service_impl: &ItemImpl) -> BTreeMap<String, (&ImplItemFn, usize)> {
    shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    })
}

struct HandlerGenerator<'a> {
    handler: Func<'a>,
    result_type: Type,
    reply_with_value: bool,
    is_query: bool,
}

impl<'a> HandlerGenerator<'a> {
    fn from(handler: Func<'a>) -> Self {
        // process result type to extact value and replace any lifetime with 'static
        let (result_type, reply_with_value) =
            shared::extract_reply_type_with_value(handler.result())
                .map_or_else(|| (handler.result().clone(), false), |t| (t, true));
        let result_type = shared::replace_any_lifetime_with_static(result_type);
        let is_query = handler.receiver().map_or(true, |r| r.mutability.is_none());

        if reply_with_value && is_query {
            abort!(
                handler.result().span(),
                "using `CommandReply` type in a query is not allowed"
            );
        }

        Self {
            handler,
            result_type,
            reply_with_value,
            is_query,
        }
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

    fn result_type(&self) -> &Type {
        &self.result_type
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
        self.is_query
    }

    fn reply_with_value(&self) -> bool {
        self.reply_with_value
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
                #(pub(super) #params_struct_members),*
            }
        )
    }

    fn invocation_func(&self, meta_module_ident: &Ident, sails_path: &Path) -> TokenStream {
        let invocation_func_ident = self.invocation_func_ident();
        let receiver = self.handler.receiver();
        let params_struct_ident = self.params_struct_ident();
        let handler_func_ident = self.handler_func_ident();
        let handler_func_params = self.handler.params().iter().map(|item| {
            let param_ident = item.0;
            quote!(request.#param_ident)
        });

        let result_type = self.result_type();
        let await_token = self.handler.is_async().then(|| quote!(.await));
        let handle_token = if self.reply_with_value() {
            quote! {
                let command_reply: CommandReply<#result_type> = self.#handler_func_ident(#(#handler_func_params),*)#await_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_func_ident(#(#handler_func_params),*)#await_token;
                let value = 0u128;
            }
        };

        quote!(
            async fn #invocation_func_ident(#receiver, mut input: &[u8]) -> (Vec<u8>, u128)
            {
                let request: #meta_module_ident::#params_struct_ident = #sails_path::Decode::decode(&mut input).expect("Failed to decode request");
                #handle_token
                return (#sails_path::Encode::encode(&result), value);
            }
        )
    }

    fn invocation_func_v2(&self, meta_module_ident: &Ident, sails_path: &Path) -> TokenStream {
        let params_struct_ident = self.params_struct_ident();
        let handler_func_ident = self.handler_func_ident();
        let handler_func_params = self.handler.params().iter().map(|item| {
            let param_ident = item.0;
            quote!(request.#param_ident)
        });

        let result_type = self.result_type();
        let await_token = self.handler.is_async().then(|| quote!(.await));
        let handle_token = if self.reply_with_value() {
            quote! {
                let command_reply: CommandReply<#result_type> = self.#handler_func_ident(#(#handler_func_params),*)#await_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_func_ident(#(#handler_func_params),*)#await_token;
                let value = 0u128;
            }
        };

        quote!(
                let request: #meta_module_ident::#params_struct_ident = #sails_path::Decode::decode(&mut __input).expect("Failed to decode request");
                #handle_token
        )
    }
}
