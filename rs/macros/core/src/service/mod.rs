//! Supporting functions and structures for the `gservice` macro.

use crate::{
    sails_paths,
    shared::{self, Func},
};
use args::ServiceArgs;
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use std::collections::BTreeMap;
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, GenericArgument,
    GenericParam, Ident, ImplItemFn, ItemImpl, Lifetime, LifetimeParam, Path, PathArguments, Type,
    Visibility,
};

mod args;
#[cfg(feature = "ethexe")]
mod ethexe;

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

    for (handler_route, (handler_fn, _, unwrap_result)) in service_handlers {
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
        let handler_generator = HandlerGenerator::from(handler_func.clone(), unwrap_result);
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
            let handler_route_ident = Ident::new(handler_route.as_str(), Span::call_site());

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

            let base_service_meta = quote!(#sails_path::meta::AnyServiceMeta::new::< #base_type >);

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

    // lifetime names like '_', 'a', 'b' etc.
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

    // Replace special "_" lifetimes with '_1', '_2' etc.
    let mut service_impl_generics = generics.clone();
    let mut service_impl_type_path = service_type_path.clone();
    if let PathArguments::AngleBracketed(mut type_args) = service_type_args {
        for (idx, a) in type_args.args.iter_mut().enumerate() {
            if let GenericArgument::Lifetime(lifetime) = a {
                if lifetime.ident == "_" {
                    let ident = Ident::new(&format!("_{idx}"), Span::call_site());
                    lifetime.ident = ident;
                    service_impl_generics
                        .params
                        .push(GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())));
                }
            }
        }

        service_impl_type_path
            .path
            .segments
            .last_mut()
            .unwrap()
            .arguments = PathArguments::AngleBracketed(type_args);
    }

    let service_impl_exposure_args = if exposure_lifetimes.is_empty() {
        quote! { #service_impl_type_path }
    } else {
        quote! { #exposure_lifetimes, #service_impl_type_path }
    };

    // We propagate only known attributes as we don't know the consequences of unknown ones
    let exposure_allow_attrs = service_impl
        .attrs
        .iter()
        .filter(|attr| matches!(attr.path().get_ident(), Some(ident) if ident == "allow"));

    // ethexe
    #[cfg(feature = "ethexe")]
    let service_signature_impl = ethexe::service_signature_impl(&service_impl, &sails_path);
    #[cfg(not(feature = "ethexe"))]
    let service_signature_impl = quote!();
    #[cfg(feature = "ethexe")]
    let try_handle_solidity_impl = ethexe::try_handle_impl(&service_impl, &sails_path);
    #[cfg(not(feature = "ethexe"))]
    let try_handle_solidity_impl = quote!();

    quote!(
        #service_impl

        pub struct #exposure_type_path<#exposure_generic_args> {
            #message_id_ident : #sails_path::MessageId,
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

        #( #exposure_allow_attrs )*
        impl #generics #exposure_type_path< #exposure_args > #service_type_constraints {
            #( #exposure_funcs )*

            #( #base_exposure_accessors )*

            pub async fn try_handle(&mut self, #input_ident : &[u8]) -> Option<(Vec<u8>, u128)> {
                #( #invocation_dispatches )*
                #( #base_exposures_invocations )*
                None
            }

            #( #invocation_funcs )*

            #try_handle_solidity_impl

            #exposure_set_event_listener_code
        }

        impl #generics #sails_path::gstd::services::Exposure for #exposure_type_path< #exposure_args > #service_type_constraints {
            fn message_id(&self) -> #sails_path::MessageId {
                self. #message_id_ident
            }

            fn route(&self) -> &'static [u8] {
                self. #route_ident
            }
        }

        impl #service_impl_generics #sails_path::gstd::services::Service for #service_impl_type_path #service_type_constraints {
            type Exposure = #exposure_type_path< #service_impl_exposure_args >;

            fn expose(self, #message_id_ident : #sails_path::MessageId, #route_ident : &'static [u8]) -> Self::Exposure {
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

        impl #generics #sails_path::meta::ServiceMeta for #service_type_path #service_type_constraints {
            type CommandsMeta = #meta_module_ident::CommandsMeta;
            type QueriesMeta = #meta_module_ident::QueriesMeta;
            type EventsMeta = #meta_module_ident::EventsMeta;
            const BASE_SERVICES: &'static [fn() -> #sails_path::meta::AnyServiceMeta] = &[
                #( #base_services_meta ),*
            ];
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

        #service_signature_impl
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

fn discover_service_handlers(
    service_impl: &ItemImpl,
) -> BTreeMap<String, (&ImplItemFn, usize, bool)> {
    shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    })
}

struct HandlerGenerator<'a> {
    handler: Func<'a>,
    result_type: Type,
    unwrap_result: bool,
    reply_with_value: bool,
    is_query: bool,
}

impl<'a> HandlerGenerator<'a> {
    fn from(handler: Func<'a>, unwrap_result: bool) -> Self {
        // process result type if set unwrap result
        let result_type = unwrap_result
            .then(|| {
                shared::extract_result_type_from_path(handler.result()).unwrap_or_else(|| {
                    abort!(
                        handler.result().span(),
                        "`unwrap_result` can be applied to methods returns result only"
                    )
                })
            })
            .unwrap_or_else(|| handler.result());
        // process result type to extact value and replace any lifetime with 'static
        let (result_type, reply_with_value) = shared::extract_reply_type_with_value(result_type)
            .map_or_else(|| (result_type, false), |ty| (ty, true));
        let result_type = shared::replace_any_lifetime_with_static(result_type.clone());
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
            unwrap_result,
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
        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));

        let handle_token = if self.reply_with_value() {
            quote! {
                let command_reply: CommandReply<#result_type> = self.#handler_func_ident(#(#handler_func_params),*)#await_token #unwrap_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_func_ident(#(#handler_func_params),*)#await_token #unwrap_token;
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

    /// Generates code for encode/decode parameters and fn invocation
    /// ```rust
    /// let (p1, p2): (u32, String) = SolValue::abi_decode_params(input, false).expect("Failed to decode request");
    /// let result: u32 = self.do_this(p1, p2).await;
    /// let value = 0u128;
    /// return Some((SolValue::abi_encode(&result), value));
    /// ```
    #[cfg(feature = "ethexe")]
    fn invocation_func_solidity(&self, sails_path: &Path) -> TokenStream {
        let handler_func_ident = self.handler_func_ident();
        let handler_params = self.handler.params().iter().map(|item| {
            let param_ident = item.0;
            quote!(#param_ident)
        });
        let handler_params_comma = self.handler.params().iter().map(|item| {
            let param_ident = item.0;
            quote!(#param_ident,)
        });
        let handler_types = self.handler.params().iter().map(|item| {
            let param_type = item.1;
            quote!(#param_type,)
        });

        let result_type = self.result_type();
        let await_token = self.handler.is_async().then(|| quote!(.await));
        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));

        let handle_token = if self.reply_with_value() {
            quote! {
                let command_reply: CommandReply<#result_type> = self.#handler_func_ident(#(#handler_params),*)#await_token #unwrap_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_func_ident(#(#handler_params),*)#await_token #unwrap_token;
                let value = 0u128;
            }
        };

        quote! {
            let (#(#handler_params_comma)*) : (#(#handler_types)*) = #sails_path::alloy_sol_types::SolValue::abi_decode_params(input, false).expect("Failed to decode request");
            #handle_token
            return Some((#sails_path::alloy_sol_types::SolValue::abi_encode(&result), value));
        }
    }
}
