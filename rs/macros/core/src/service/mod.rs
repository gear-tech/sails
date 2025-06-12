//! Supporting functions and structures for the `gservice` macro.

use crate::{
    sails_paths,
    shared::{self, FnBuilder},
};
use args::ServiceArgs;
use convert_case::{Case, Casing};
use proc_macro_error::abort;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use syn::{Generics, Ident, ImplItemFn, ItemImpl, Path, Type, TypePath, Visibility, WhereClause};

mod args;
#[cfg(feature = "ethexe")]
mod ethexe;
mod exposure;
mod meta;

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

struct ServiceBuilder<'a> {
    service_impl: &'a ItemImpl,
    sails_path: &'a Path,
    base_types: &'a [Path],
    generics: Generics,
    type_constraints: Option<WhereClause>,
    type_path: &'a TypePath,
    events_type: Option<&'a Path>,
    service_handlers: Vec<FnBuilder<'a>>,
    exposure_ident: Ident,
    message_id_ident: Ident,
    route_ident: Ident,
    inner_ident: Ident,
    inner_ptr_ident: Ident,
    base_ident: Ident,
    input_ident: Ident,
    meta_module_ident: Ident,
}

impl<'a> ServiceBuilder<'a> {
    fn from(
        service_impl: &'a ItemImpl,
        sails_path: &'a Path,
        service_args: &'a ServiceArgs,
    ) -> Self {
        let (generics, type_constraints) = shared::impl_constraints(service_impl);
        let (type_path, _type_args, service_ident) =
            shared::impl_type_refs(service_impl.self_ty.as_ref());
        let service_handlers = discover_service_handlers(service_impl)
            .into_iter()
            .map(|(route, (fn_impl, _idx, unwrap_result))| {
                FnBuilder::from(route, fn_impl, unwrap_result, sails_path)
            })
            .collect::<Vec<_>>();
        let exposure_name = format!(
            "{}Exposure",
            service_ident.to_string().to_case(Case::Pascal)
        );
        let exposure_ident = Ident::new(&exposure_name, Span::call_site());
        let message_id_ident = Ident::new("message_id", Span::call_site());
        let route_ident = Ident::new("route", Span::call_site());
        let inner_ident = Ident::new("inner", Span::call_site());
        let inner_ptr_ident = Ident::new("inner_ptr", Span::call_site());
        let base_ident = Ident::new("base", Span::call_site());
        let input_ident = Ident::new("input", Span::call_site());
        let meta_module_name = format!("{}_meta", service_ident.to_string().to_case(Case::Snake));
        let meta_module_ident = Ident::new(&meta_module_name, Span::call_site());

        Self {
            service_impl,
            sails_path,
            base_types: service_args.base_types(),
            generics,
            type_constraints,
            type_path,
            events_type: service_args.events_type(),
            service_handlers,
            exposure_ident,
            message_id_ident,
            route_ident,
            inner_ident,
            inner_ptr_ident,
            base_ident,
            input_ident,
            meta_module_ident,
        }
    }

    fn type_constraints(&self) -> Option<&WhereClause> {
        self.type_constraints.as_ref()
    }
}

#[cfg(not(feature = "ethexe"))]
impl ServiceBuilder<'_> {
    fn service_signature_impl(&self) -> TokenStream {
        quote!()
    }

    fn try_handle_solidity_impl(&self, _base_ident: &Ident) -> TokenStream {
        quote!()
    }

    fn service_emit_eth_impls(&self) -> Option<TokenStream> {
        None
    }

    fn exposure_emit_eth_impls(&self) -> Option<TokenStream> {
        None
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

    let service_builder = ServiceBuilder::from(&service_impl, &sails_path, &service_args);

    let service_handlers = discover_service_handlers(&service_impl);

    if service_handlers.is_empty() && service_args.base_types().is_empty() {
        abort!(
            service_impl,
            "`service` attribute requires impl to define at least one public method or extend another service"
        );
    }

    let meta_trait_impl = service_builder.meta_trait_impl();
    let meta_module = service_builder.meta_module();

    let exposure_struct = service_builder.exposure_struct();
    let exposure_impl = service_builder.exposure_impl();
    let service_trait_impl = service_builder.service_trait_impl();
    let service_emit_event_impls = service_builder.service_emit_event_impls();
    let service_emit_eth_impls = service_builder.service_emit_eth_impls();
    let exposure_listen_and_drop = service_builder.exposure_listen_and_drop();

    // ethexe
    let service_signature_impl = service_builder.service_signature_impl();

    quote!(
        #service_impl

        #exposure_struct

        #exposure_impl

        #service_trait_impl

        #meta_trait_impl

        #meta_module

        #service_signature_impl

        #service_emit_event_impls
        #service_emit_eth_impls

        #exposure_listen_and_drop
    )
}

fn discover_service_handlers(
    service_impl: &ItemImpl,
) -> BTreeMap<String, (&ImplItemFn, usize, bool)> {
    shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    })
}

impl FnBuilder<'_> {
    fn result_type_with_static_lifetime(&self) -> Type {
        let (result_type, _) = self.result_type_with_value();

        shared::replace_any_lifetime_with_static(result_type.clone())
    }

    fn handler_meta_variant(&self) -> TokenStream {
        let handler_route_ident = Ident::new(self.route.as_str(), Span::call_site());
        let handler_docs_attrs = self
            .impl_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"));
        let params_struct_ident = &self.params_struct_ident;
        let result_type = self.result_type_with_static_lifetime();

        quote!(
            #( #handler_docs_attrs )*
            #handler_route_ident(#params_struct_ident, #result_type)
        )
    }

    fn params_struct(&self, scale_codec_path: &Path, scale_info_path: &Path) -> TokenStream {
        let params_struct_ident = &self.params_struct_ident;
        let params_struct_members = self.params().map(|(ident, ty)| quote!(#ident: #ty));
        let handler_route_bytes = self.encoded_route.as_slice();
        let is_async = self.is_async();

        quote!(
            #[derive(Decode, TypeInfo)]
            #[codec(crate = #scale_codec_path )]
            #[scale_info(crate = #scale_info_path )]
            pub struct #params_struct_ident {
                #(pub(super) #params_struct_members,)*
            }

            impl InvocationIo for #params_struct_ident {
                const ROUTE: &'static [u8] = &[ #(#handler_route_bytes),* ];
                type Params = Self;
                const ASYNC: bool = #is_async;
            }
        )
    }

    fn try_handle_branch_impl(
        &self,
        meta_module_ident: &Ident,
        input_ident: &Ident,
    ) -> TokenStream {
        let handler_func_ident = self.ident;

        let params_struct_ident = &self.params_struct_ident;
        let handler_func_params = self
            .params_idents()
            .iter()
            .map(|ident| quote!(request.#ident));

        let (result_type, reply_with_value) = self.result_type_with_value();
        let await_token = self.is_async().then(|| quote!(.await));
        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));

        let handle_token = if reply_with_value {
            quote! {
                let command_reply: CommandReply< #result_type > = self.#handler_func_ident(#(#handler_func_params),*)#await_token #unwrap_token.into();
                let (result, value) = command_reply.to_tuple();
            }
        } else {
            quote! {
                let result = self.#handler_func_ident(#(#handler_func_params),*)#await_token #unwrap_token;
                let value = 0u128;
            }
        };

        quote! {
            if let Ok(request) = #meta_module_ident::#params_struct_ident::decode_params( #input_ident) {
                #handle_token
                #meta_module_ident::#params_struct_ident::with_optimized_encode(
                    &result,
                    self.route().as_ref(),
                    |encoded_result| result_handler(encoded_result, value),
                );
                return Some(());
            }
        }
    }

    fn exposure_func(&self, inner_ident: &Ident) -> TokenStream {
        let sails_path = self.sails_path;
        let handler_ident = self.ident;
        let handler_fn_sig = &self.impl_fn.sig;
        let handler_params = self.params_idents();

        let handler_await_token = self.is_async().then(|| quote!(.await));
        let handler_allow_attrs = self
            .impl_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("allow"));

        quote!(
            #( #handler_allow_attrs )*
            pub #handler_fn_sig {
                let exposure_scope = #sails_path::gstd::services::ExposureCallScope::new(self);
                self. #inner_ident . #handler_ident (#(#handler_params),*) #handler_await_token
            }
        )
    }

    fn check_asyncness_branch_impl(
        &self,
        meta_module_ident: &Ident,
        input_ident: &Ident,
    ) -> TokenStream {
        let params_struct_ident = &self.params_struct_ident;

        quote! {
            if #meta_module_ident::#params_struct_ident::check_route( #input_ident).is_ok() {
                return Some(#meta_module_ident::#params_struct_ident::ASYNC);
            }
        }
    }
}
