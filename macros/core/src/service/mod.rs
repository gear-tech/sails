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
    shared::{self, extract_lifetime_names, Func, ImplType},
};
use args::ServiceArgs;
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use std::collections::BTreeMap;
use syn::{
    parse_quote, spanned::Spanned, Ident, ImplItemFn, ItemImpl, Lifetime, Path, Type, Visibility,
};

mod args;

static mut SERVICE_SPANS: BTreeMap<String, Span> = BTreeMap::new();

pub fn gservice(args: TokenStream, service_impl: TokenStream) -> TokenStream {
    let service_impl = parse_gservice_impl(service_impl);
    ensure_single_gservice_on_impl(&service_impl);
    ensure_single_gservice_by_name(&service_impl);
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

fn generate_gservice(args: TokenStream, service_impl: ItemImpl) -> TokenStream {
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

    let service_handlers = discover_service_handlers(&service_impl);

    if service_handlers.is_empty() && service_args.base_types().is_empty() {
        abort!(
            service_impl,
            "`gservice` attribute requires impl to define at least one public method or extend another service"
        );
    }

    let events_type = service_args.events_type().as_ref();

    let mut service_impl = service_impl.clone();
    if let Some(events_type) = events_type {
        service_impl.items.push(parse_quote!(
            fn notify_on(&mut self, event: #events_type ) -> sails_rtl::errors::Result<()>  {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let self_ptr = self as *const _ as usize;
                    let event_listeners = event_listeners().lock();
                    if let Some(event_listener_ptr) = event_listeners.get(&self_ptr) {
                        let event_listener =
                            unsafe { &mut *(*event_listener_ptr as *mut Box<dyn FnMut(& #events_type )>) };
                        core::mem::drop(event_listeners);
                        event_listener(&event);
                    }
                }
                sails_rtl::gstd::events::__notify_on(event)
            }
        ));
    }

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

    let code_for_base_types = service_args.base_types().iter()
        .enumerate()
        .map(|(idx, base_type)| {
            let base_ident = Ident::new(&format!("base_{}", idx), Span::call_site());
            let as_base_ident = Ident::new(&format!("as_base_{}", idx), Span::call_site());

            let base_exposure_accessor = quote!(
                pub fn #as_base_ident (&self) -> &< #base_type as sails_rtl::gstd::services::Service>::Exposure {
                    &self. #base_ident
                }
            );

            let base_exposure_member = quote!(
                #base_ident : < #base_type as sails_rtl::gstd::services::Service>::Exposure,
            );

            let base_exposure_instantiation = quote!(
                #base_ident : < #base_type as Clone>::clone(AsRef::< #base_type >::as_ref( #inner_ident ))
                    .expose( #message_id_ident , #route_ident ),
            );

            let base_exposure_invocation = quote!(
                if let Some(output) = self. #base_ident .try_handle(#input_ident).await {
                    return Some(output);
                }
            );

            let base_service_meta = quote!(sails_rtl::meta::AnyServiceMeta::new::< #base_type >());

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

    let events_listeners_code = events_type.map(|_| generate_event_listeners());

    let exposure_set_event_listener_code = events_type.map(|t| {
        // get non conflicting lifetime name
        let lifetimes = extract_lifetime_names(&service_type_args);
        let mut lt = "__elg".to_owned();
        while lifetimes.contains(&lt) {
            lt = format!("_{}", lt);
        }
        let lifetime_name = format!("'{0}", lt);
        generate_exposure_set_event_listener(t, Lifetime::new(&lifetime_name, Span::call_site()))
    });

    let exposure_drop_code = events_type.map(|_| generate_exposure_drop());

    let no_events_type = Path::from(Ident::new("NoEvents", Span::call_site()));
    let events_type = events_type.unwrap_or(&no_events_type);

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
            #[cfg(not(target_arch = "wasm32"))]
            #inner_ident : Box<T>, // Ensure service is not movable
            #[cfg(not(target_arch = "wasm32"))]
            #inner_ptr_ident : *const T, // Prevent exposure being Send + Sync
            #[cfg(target_arch = "wasm32")]
            #inner_ident : T,
            #( #base_exposures_members )*
        }

        #exposure_drop_code

        impl #service_type_args Exposure< #service_type > #service_type_constraints {
            #( #exposure_funcs )*

            #( #base_exposure_accessors )*

            pub async fn handle(&mut self, #input_ident: &[u8]) -> Vec<u8> {
                self.try_handle( #input_ident ).await.unwrap_or_else(|| {
                    #unexpected_route_panic
                })
            }

            pub async fn try_handle(&mut self, #input_ident : &[u8]) -> Option<Vec<u8>> {
                #( #invocation_dispatches )*
                #( #base_exposures_invocations )*
                None
            }

            #( #invocation_funcs )*

            #exposure_set_event_listener_code
        }

        impl #service_type_args sails_rtl::gstd::services::Exposure for Exposure< #service_type > #service_type_constraints {
            fn message_id(&self) -> sails_rtl::MessageId {
                self. #message_id_ident
            }

            fn route(&self) -> &'static [u8] {
                self. #route_ident
            }
        }

        impl #service_type_args sails_rtl::gstd::services::Service for #service_type #service_type_constraints {
            type Exposure = Exposure< #service_type >;

            fn expose(self, #message_id_ident : sails_rtl::MessageId, #route_ident : &'static [u8]) -> Self::Exposure {
                #[cfg(not(target_arch = "wasm32"))]
                let inner_box = Box::new(self);
                #[cfg(not(target_arch = "wasm32"))]
                let #inner_ident = inner_box.as_ref();
                #[cfg(target_arch = "wasm32")]
                let #inner_ident = &self;
                Self::Exposure {
                    #message_id_ident ,
                    #route_ident ,
                    #( #base_exposures_instantiations )*
                    #[cfg(not(target_arch = "wasm32"))]
                    #inner_ptr_ident : inner_box.as_ref() as *const Self,
                    #[cfg(not(target_arch = "wasm32"))]
                    #inner_ident : inner_box ,
                    #[cfg(target_arch = "wasm32")]
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

        #events_listeners_code

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

// Generates function for accessing event listeners map in non-wasm code.
fn generate_event_listeners() -> TokenStream {
    quote!(
        type __EventlistenersMap = sails_rtl::collections::BTreeMap<usize, usize>;
        type __Mutex<T> = sails_rtl::spin::Mutex<T>;

        #[cfg(not(target_arch = "wasm32"))]
        fn event_listeners() -> &'static __Mutex<__EventlistenersMap> {
            static EVENT_LISTENERS: __Mutex<__EventlistenersMap> =
                __Mutex::new(__EventlistenersMap::new());
            &EVENT_LISTENERS
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub struct EventListenerGuard<'a> {
            service_ptr: usize,
            listener_ptr: usize,
            _phantom: core::marker::PhantomData<&'a ()>,
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl<'a> Drop for EventListenerGuard<'a> {
            fn drop(&mut self) {
                let mut event_listeners = event_listeners().lock();
                let listener_ptr = event_listeners.remove(&self.service_ptr);
                if listener_ptr != Some(self.listener_ptr) {
                    panic!("event listener is being removed out of order");
                }
            }
        }
    )
}

fn generate_exposure_drop() -> TokenStream {
    quote!(
        #[cfg(not(target_arch = "wasm32"))]
        impl<T> Drop for Exposure<T> {
            fn drop(&mut self) {
                let service_ptr = self.inner_ptr as usize;
                let mut event_listeners = event_listeners().lock();
                if event_listeners.remove(&service_ptr).is_some() {
                    panic!("there should be no any event listeners left by this time");
                }
            }
        }
    )
}

fn generate_exposure_set_event_listener(events_type: &Path, lifetime: Lifetime) -> TokenStream {
    quote!(
        #[cfg(not(target_arch = "wasm32"))]
        // Immutable so one can set it via AsRef when used with extending
        pub fn set_event_listener<#lifetime>(
            &self,
            listener: impl FnMut(& #events_type ) + #lifetime,
        ) -> EventListenerGuard<#lifetime> {
            if core::mem::size_of_val(self.inner.as_ref()) == 0 {
                panic!("setting event listener on a zero-sized service is not supported for now");
            }
            let service_ptr = self.inner_ptr as usize;
            let listener: Box<dyn FnMut(& #events_type )> = Box::new(listener);
            let listener = Box::new(listener);
            let listener_ptr = Box::into_raw(listener) as usize;
            let mut event_listeners = event_listeners().lock();
            if event_listeners.contains_key(&service_ptr) {
                panic!("event listener is already set");
            }
            event_listeners.insert(service_ptr, listener_ptr);
            EventListenerGuard {
                service_ptr,
                listener_ptr,
                _phantom: core::marker::PhantomData,
            }
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
