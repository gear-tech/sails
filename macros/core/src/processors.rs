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

//! Supporting functions and structures for the `command_handlers` and `query_handlers` macros.

use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{self, spanned::Spanned, Ident, Receiver, Signature, Type, TypePath};

pub(super) fn gservice(impl_tokens: TokenStream2) -> TokenStream2 {
    let handlers_impl = syn::parse2::<syn::ItemImpl>(impl_tokens.clone())
        .unwrap_or_else(|err| abort!(err.span(), "Failed to parse handlers impl: {}", err));

    let handler_funcs = handler_funcs(&handlers_impl).collect::<Vec<_>>();

    if handler_funcs.is_empty() {
        abort!(
            handlers_impl,
            "No handlers found. Try either defining one or removing the macro usage"
        );
    }

    let service_type = service_type(&handlers_impl.self_ty);

    let mut params_structs = vec![];
    let mut invocation_funcs = vec![];
    let mut invocations = vec![];
    let mut commands_meta_variants = vec![];
    let mut queries_meta_variants = vec![];

    for handler_func in &handler_funcs {
        let handler = Handler::from(handler_func);
        let handler_generator = HandlerGenerator::from(service_type, handler);
        let invocation_func_ident = handler_generator.invocation_func_ident();
        let invocation_path = invocation_func_ident.to_string().to_case(Case::Pascal);
        let invocation_route = format!("{}/", invocation_path);

        params_structs.push(handler_generator.params_struct());
        invocation_funcs.push(handler_generator.invocation_func());
        invocations.push(quote!(
            if input.starts_with(#invocation_route.as_bytes()) {
                return #invocation_func_ident(service, &input[#invocation_route.as_bytes().len()..]).await;
            }
        ));

        let params_struct_ident = handler_generator.params_struct_ident();
        let result_type = handler_generator.result_type();
        let invocation_path = Ident::new(&invocation_path, proc_macro2::Span::call_site());
        let handler_meta_variant = quote!(
            #invocation_path(#params_struct_ident, #result_type),
        );
        if handler_generator.is_query() {
            queries_meta_variants.push(handler_meta_variant);
        } else {
            commands_meta_variants.push(handler_meta_variant);
        }
    }

    quote!(
        #impl_tokens

        #(#[derive(Decode, TypeInfo)] #params_structs)*

        pub mod meta {
            use super::*;

            #[derive(TypeInfo)]
            pub enum CommandsMeta {
                #(#commands_meta_variants)*
            }

            #[derive(TypeInfo)]
            pub enum QueriesMeta {
                #(#queries_meta_variants)*
            }

            pub struct ServiceMeta;

            impl sails_service::ServiceMeta for ServiceMeta {
                type Commands = CommandsMeta;
                type Queries = QueriesMeta;
            }
        }

        pub mod handlers {
            use super::*;

            pub async fn process_request(service: &mut #service_type, mut input: &[u8]) -> Vec<u8> {
                #(#invocations)*
                panic!("Unknown request");
            }

            #(#invocation_funcs)*
        }
    )
}

fn service_type(service_type: &Type) -> &TypePath {
    if let syn::Type::Path(type_path) = service_type {
        type_path
    } else {
        abort!(
            service_type.span(),
            "Failed to parse service type: {}",
            service_type.to_token_stream()
        )
    }
}

fn handler_funcs(handlers_impl: &syn::ItemImpl) -> impl Iterator<Item = &syn::Signature> {
    handlers_impl.items.iter().filter_map(|item| {
        if let syn::ImplItem::Fn(fn_item) = item {
            if matches!(fn_item.vis, syn::Visibility::Public(_)) && fn_item.sig.receiver().is_some()
            {
                return Some(&fn_item.sig);
            }
        }
        None
    })
}

struct Handler<'a> {
    func: &'a Ident,
    receiver: &'a Receiver,
    params: Vec<(&'a Ident, &'a Type)>,
    result: &'a Type,
    is_async: bool,
    is_query: bool,
}

impl<'a> Handler<'a> {
    fn from(handler_signature: &'a Signature) -> Self {
        let func = &handler_signature.ident;
        let receiver = handler_signature.receiver().unwrap_or_else(|| {
            abort!(
                handler_signature.span(),
                "Handler must be a public method of service"
            )
        });
        let params = params(handler_signature).collect();
        let result = result(handler_signature);
        Self {
            func,
            receiver,
            params,
            result,
            is_async: handler_signature.asyncness.is_some(),
            is_query: receiver.mutability.is_none(),
        }
    }
}

fn params(handler_signature: &syn::Signature) -> impl Iterator<Item = (&Ident, &Type)> {
    handler_signature.inputs.iter().skip(1).map(|arg| {
        if let syn::FnArg::Typed(arg) = arg {
            let arg_ident = if let syn::Pat::Ident(arg_ident) = arg.pat.as_ref() {
                &arg_ident.ident
            } else {
                abort!(arg.span(), "Unnamed arguments are not supported");
            };
            (arg_ident, arg.ty.as_ref())
        } else {
            abort!(arg.span(), "Arguments of the Self type are not supported");
        }
    })
}

fn result(handler_signature: &Signature) -> &Type {
    if let syn::ReturnType::Type(_, ty) = &handler_signature.output {
        ty.as_ref()
    } else {
        abort!(
            handler_signature.output.span(),
            "Failed to parse return type"
        );
    }
}

struct HandlerGenerator<'a> {
    service_type: &'a TypePath,
    handler: Handler<'a>,
}

impl<'a> HandlerGenerator<'a> {
    fn from(service_type: &'a TypePath, handler: Handler<'a>) -> Self {
        Self {
            service_type,
            handler,
        }
    }

    fn params_struct_ident(&self) -> Ident {
        syn::Ident::new(
            &format!(
                "{}Params",
                self.handler.func.to_string().to_case(Case::Pascal)
            ),
            proc_macro2::Span::call_site(),
        )
    }

    fn result_type(&self) -> Type {
        self.handler.result.clone()
    }

    fn handler_func_ident(&self) -> Ident {
        self.handler.func.clone()
    }

    fn invocation_func_ident(&self) -> Ident {
        self.handler_func_ident()
    }

    fn is_query(&self) -> bool {
        self.handler.is_query
    }

    fn params_struct(&self) -> TokenStream2 {
        let params_struct_ident = self.params_struct_ident();
        let params_struct_members = self.handler.params.iter().map(|item| {
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
        let service_ref = self.handler.receiver.reference.as_ref().map(|r| r.0);
        let service_lifetime = self.handler.receiver.reference.as_ref().map(|r| &r.1);
        let service_mut = self.handler.receiver.mutability;
        let service_type = self.service_type;
        let params_struct_ident = self.params_struct_ident();
        let handler_func_ident = self.handler_func_ident();
        let handler_func_params = self.handler.params.iter().map(|item| {
            let param_ident = item.0;
            quote!(request.#param_ident)
        });

        let await_token = if self.handler.is_async {
            quote!(.await)
        } else {
            quote!()
        };

        quote!(
            async fn #invocation_func_ident(service: #service_ref #service_lifetime #service_mut #service_type, mut input: &[u8]) -> Vec<u8> {
                let request = #params_struct_ident::decode(&mut input).expect("Failed to decode request");
                let result = service.#handler_func_ident(#(#handler_func_params),*)#await_token;
                return result.encode();
            }
        )
    }
}

/// Generates a processor function with requests it can process and responses it can return.
/// The processor function essentially acts as a router for the requests on their way to the
/// handlers.
pub(super) fn generate(
    mod_tokens: TokenStream2,
    request_enum_name: &str,
    response_enum_name: &str,
) -> TokenStream2 {
    let handlers_mod = syn::parse2::<syn::ItemMod>(mod_tokens)
        .unwrap_or_else(|err| abort!(err.span(), "Failed to parse handlers module: {}", err));
    let handlers_mod_ident = &handlers_mod.ident;
    let handlers_mod_visibility = &handlers_mod.vis;
    let (handlers_mod_funcs, handlers_mod_non_funcs) = split_handlers_mod(&handlers_mod);

    let request_enum_ident = syn::Ident::new(request_enum_name, proc_macro2::Span::call_site());
    let response_enum_ident = syn::Ident::new(response_enum_name, proc_macro2::Span::call_site());
    let function_ident = get_processor_function_ident(request_enum_name);

    if handlers_mod_funcs.is_empty() {
        abort!(
            handlers_mod,
            "No handlers found. Please either define one or remove the macro usage"
        );
    }

    let processor_tokens = ProcessorTokens::from(
        &handlers_mod_funcs,
        &request_enum_ident,
        &response_enum_ident,
        &function_ident,
    );

    let request_enum = processor_tokens.request_enum;
    let response_enum = processor_tokens.response_enum;
    let function = processor_tokens.function;

    let scale_codec_crate_ident = get_scale_codec_crate_ident(request_enum_name);
    let scale_info_crate_ident = get_scale_info_crate_ident(request_enum_name);

    quote!(
        #handlers_mod_visibility mod #handlers_mod_ident {
            extern crate parity_scale_codec as #scale_codec_crate_ident;
            extern crate scale_info as #scale_info_crate_ident;

            #[derive(#scale_codec_crate_ident::Encode, #scale_codec_crate_ident::Decode, #scale_info_crate_ident::TypeInfo)]
            #request_enum

            #[derive(#scale_codec_crate_ident::Encode, #scale_codec_crate_ident::Decode, #scale_info_crate_ident::TypeInfo)]
            #response_enum

            #(#handlers_mod_non_funcs)*

            #[cfg(feature = "handlers")] // TODO: Make this configurable?
            pub mod handlers {
                use super::*;

                #function

                #(#handlers_mod_funcs)*
            }
        }
    )
}

struct ProcessorTokens {
    request_enum: TokenStream2,
    response_enum: TokenStream2,
    function: TokenStream2,
}

impl ProcessorTokens {
    fn from(
        handlers_mod_funcs: &[&syn::ItemFn],
        request_enum_ident: &syn::Ident,
        response_enum_ident: &syn::Ident,
        function_ident: &syn::Ident,
    ) -> ProcessorTokens {
        let handlers_signatures = handlers_mod_funcs.iter().map(|item_fn| &item_fn.sig);

        let handlers_tokens = handlers_signatures
            .map(|handler_signature| {
                HandlerTokens::from(request_enum_ident, response_enum_ident, handler_signature)
            })
            .collect::<Vec<_>>();

        let request_enum_variants = handlers_tokens
            .iter()
            .map(|handler_tokens| &handler_tokens.request_enum_variant);

        let response_enum_variants = handlers_tokens
            .iter()
            .map(|handler_tokens| &handler_tokens.response_enum_variant);

        let call_match_arms = handlers_tokens
            .iter()
            .map(|handler_tokens| &handler_tokens.call_match_arm);

        let has_async_handler = handlers_tokens
            .iter()
            .any(|handler_tokens| handler_tokens.is_async);

        let fn_signature = if has_async_handler {
            quote!(async fn #function_ident(request: #request_enum_ident) -> (#response_enum_ident, bool))
        } else {
            quote!(fn #function_ident(request: #request_enum_ident) -> (#response_enum_ident, bool))
        };

        ProcessorTokens {
            request_enum: quote!(
                pub enum #request_enum_ident {
                    #(#request_enum_variants)*
                }
            ),
            response_enum: quote!(
                pub enum #response_enum_ident {
                    #(#response_enum_variants)*
                }
            ),
            function: quote!(
                pub #fn_signature {
                    match request {
                        #(#call_match_arms)*
                    }
                }
            ),
        }
    }
}

struct HandlerTokens {
    request_enum_variant: TokenStream2,
    response_enum_variant: TokenStream2,
    call_match_arm: TokenStream2,
    is_async: bool,
}

impl HandlerTokens {
    fn from(
        request_enum_ident: &syn::Ident,
        response_enum_ident: &syn::Ident,
        handler_signature: &syn::Signature,
    ) -> Self {
        let enum_variant_name = syn::Ident::new(
            &handler_signature.ident.to_string().to_case(Case::Pascal),
            proc_macro2::Span::call_site(),
        );

        let response_enum_variant = {
            let response_type = Self::response_type(handler_signature);
            quote!(
                #enum_variant_name(#response_type),
            )
        };

        let (arg_types, arg_types_count) = Self::arg_types(handler_signature);

        let request_enum_variant = quote!(
             #enum_variant_name(#(#arg_types,)*),
        );

        let call_match_arm = {
            let call_param_idents = (0..arg_types_count)
                .map(|idx| syn::Ident::new(&format!("v{}", idx), proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            let call_ident = &handler_signature.ident;
            let call = if handler_signature.asyncness.is_some() {
                quote!(#call_ident(#(#call_param_idents),*).await)
            } else {
                quote!(#call_ident(#(#call_param_idents),*))
            };
            quote!(
                #request_enum_ident::#enum_variant_name(#(#call_param_idents),*) => {
                    let result: Result<_, _> = #call;
                    let is_error = result.is_err();
                    (#response_enum_ident::#enum_variant_name(result), is_error)
                }
            )
        };

        Self {
            request_enum_variant,
            response_enum_variant,
            call_match_arm,
            is_async: handler_signature.asyncness.is_some(),
        }
    }

    fn arg_types(
        handler_signature: &syn::Signature,
    ) -> (impl Iterator<Item = &syn::Type> + '_, usize) {
        (
            handler_signature.inputs.iter().map(Self::arg_type),
            handler_signature.inputs.len(),
        )
    }

    fn response_type(handler_signature: &syn::Signature) -> syn::Type {
        Self::return_type(&handler_signature.output)
    }

    fn arg_type(arg: &syn::FnArg) -> &syn::Type {
        if let syn::FnArg::Typed(arg) = arg {
            arg.ty.as_ref()
        } else {
            abort!(arg.span(), "Arguments of the Self type are not supported");
        }
    }

    fn return_type(output: &syn::ReturnType) -> syn::Type {
        if let syn::ReturnType::Type(_, ty) = output {
            ty.as_ref().clone()
        } else {
            syn::parse2::<syn::Type>(quote!(()))
                .unwrap_or_else(|err| abort!(err.span(), "Failed to parse return type: {}", err))
        }
    }
}

fn split_handlers_mod(handlers_mod: &syn::ItemMod) -> (Vec<&syn::ItemFn>, Vec<&syn::Item>) {
    let (handlers_mod_funcs, handlers_mod_non_funcs): (Vec<&syn::Item>, Vec<&syn::Item>) =
        handlers_mod
            .content
            .as_ref()
            .unwrap_or_else(|| abort!(handlers_mod, "Handlers module must be inline"))
            .1
            .iter()
            .partition(|item| matches!(item, syn::Item::Fn(_)));
    let handlers_mod_funcs = handlers_mod_funcs
        .iter()
        .filter_map(|item_fn| match item_fn {
            syn::Item::Fn(item_fn) => Some(item_fn),
            _ => None,
        })
        .collect();
    (handlers_mod_funcs, handlers_mod_non_funcs)
}

fn get_scale_codec_crate_ident(prefix: &str) -> syn::Ident {
    syn::Ident::new(
        format!("{}_scale_codec", prefix.to_case(Case::Snake)).as_str(),
        proc_macro2::Span::call_site(),
    )
}

fn get_scale_info_crate_ident(prefix: &str) -> syn::Ident {
    syn::Ident::new(
        format!("{}_scale_info", prefix.to_case(Case::Snake)).as_str(),
        proc_macro2::Span::call_site(),
    )
}

fn get_processor_function_ident(suffix: &str) -> syn::Ident {
    syn::Ident::new(
        format!("process_{}", suffix.to_case(Case::Snake)).as_str(),
        proc_macro2::Span::call_site(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_tokens_work_for_func_with_default_return_type() {
        let signature = syn::parse2::<syn::Signature>(quote! {
            fn do_this(p1: u32, p2: String)
        })
        .unwrap();

        let handler_tokens = HandlerTokens::from(
            &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
            &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
            &signature,
        );

        assert_eq!(
            quote!(DoThis(u32, String,),).to_string(),
            handler_tokens.request_enum_variant.to_string()
        );
        assert_eq!(
            quote!(DoThis(()),).to_string(),
            handler_tokens.response_enum_variant.to_string()
        );
        assert_eq!(
            quote!(
                Commands::DoThis(v0, v1) => {
                    let result: Result<_, _> = do_this(v0, v1);
                    let is_error = result.is_err();
                    (CommandResponses::DoThis(result), is_error)
                }
            )
            .to_string(),
            handler_tokens.call_match_arm.to_string()
        );
        assert!(!handler_tokens.is_async);
    }

    #[test]
    fn handler_tokens_work_for_func_without_args() {
        let signature = syn::parse2::<syn::Signature>(quote! {
            fn do_this()
        })
        .unwrap();

        let handler_tokens = HandlerTokens::from(
            &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
            &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
            &signature,
        );

        assert_eq!(
            quote!(DoThis(),).to_string(),
            handler_tokens.request_enum_variant.to_string()
        );
        assert_eq!(
            quote!(DoThis(()),).to_string(),
            handler_tokens.response_enum_variant.to_string()
        );
        assert_eq!(
            quote!(
                Commands::DoThis() => {
                    let result: Result<_, _> = do_this();
                    let is_error = result.is_err();
                    (CommandResponses::DoThis(result), is_error)
                }
            )
            .to_string(),
            handler_tokens.call_match_arm.to_string()
        );
        assert!(!handler_tokens.is_async);
    }

    #[test]
    fn handler_tokens_work_for_async_func() {
        let signature = syn::parse2::<syn::Signature>(quote! {
            async fn do_this(p1: (u32, u8))
        })
        .unwrap();

        let handler_tokens = HandlerTokens::from(
            &syn::Ident::new("Commands", proc_macro2::Span::call_site()),
            &syn::Ident::new("CommandResponses", proc_macro2::Span::call_site()),
            &signature,
        );

        assert_eq!(
            quote!(DoThis((u32, u8),),).to_string(),
            handler_tokens.request_enum_variant.to_string()
        );
        assert_eq!(
            quote!(DoThis(()),).to_string(),
            handler_tokens.response_enum_variant.to_string()
        );
        assert_eq!(
            quote!(
                Commands::DoThis(v0) => {
                    let result: Result<_, _> = do_this(v0).await;
                    let is_error = result.is_err();
                    (CommandResponses::DoThis(result), is_error)
                }
            )
            .to_string(),
            handler_tokens.call_match_arm.to_string()
        );
        assert!(handler_tokens.is_async);
    }
}
