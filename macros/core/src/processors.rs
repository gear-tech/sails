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

use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{
    self, spanned::Spanned, FnArg, Ident, ImplItem, ItemImpl, Pat, PathArguments, Receiver,
    ReturnType, Signature, Type, TypePath, Visibility, WhereClause,
};

pub(super) fn gservice(service_impl_tokens: TokenStream2) -> TokenStream2 {
    let service_impl = syn::parse2::<ItemImpl>(service_impl_tokens.clone())
        .unwrap_or_else(|err| abort!(err.span(), "Failed to parse handlers impl: {}", err));

    let handler_funcs = handler_funcs(&service_impl).collect::<Vec<_>>();

    if handler_funcs.is_empty() {
        abort!(
            service_impl,
            "No handlers found. Try either defining one or removing the macro usage"
        );
    }

    let service_type = ServiceType::new(&service_impl);
    let service_type_path = service_type.path;
    let service_type_args = service_type.args;
    let service_type_constraints = service_type.constraints;

    let mut params_structs = vec![];
    let mut invocation_funcs = vec![];
    let mut invocations = vec![];
    let mut commands_meta_variants = vec![];
    let mut queries_meta_variants = vec![];

    for handler_func in &handler_funcs {
        let handler = Handler::from(handler_func);
        let handler_generator = HandlerGenerator::from(&service_type, handler);
        let invocation_func_ident = handler_generator.invocation_func_ident();
        let invocation_path = invocation_func_ident.to_string().to_case(Case::Pascal);

        params_structs.push(handler_generator.params_struct());
        invocation_funcs.push(handler_generator.invocation_func());
        invocations.push(quote!(
            let invocation_path = #invocation_path.encode();
            if input.starts_with(&invocation_path) {
                let output = #invocation_func_ident(service, &input[invocation_path.len()..]).await;
                return [invocation_path, output].concat();
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
        #service_impl_tokens

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

            impl sails_service_meta::ServiceMeta for ServiceMeta {
                type Commands = CommandsMeta;
                type Queries = QueriesMeta;
            }
        }

        pub mod requests {
            use super::*;

            pub async fn process #service_type_args (service: &mut #service_type_path, mut input: &[u8]) -> Vec<u8>
                #service_type_constraints
            {
                #(#invocations)*
                panic!("Unknown request");
            }

            #(#invocation_funcs)*
        }
    )
}

fn handler_funcs(service_impl: &ItemImpl) -> impl Iterator<Item = &Signature> {
    service_impl.items.iter().filter_map(|item| {
        if let ImplItem::Fn(fn_item) = item {
            if matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some() {
                return Some(&fn_item.sig);
            }
        }
        None
    })
}

/// Represents parts of a handler function.
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
        let params = Self::params(handler_signature).collect();
        let result = Self::result(handler_signature);
        Self {
            func,
            receiver,
            params,
            result,
            is_async: handler_signature.asyncness.is_some(),
            is_query: receiver.mutability.is_none(),
        }
    }

    fn params(handler_signature: &Signature) -> impl Iterator<Item = (&Ident, &Type)> {
        handler_signature.inputs.iter().skip(1).map(|arg| {
            if let FnArg::Typed(arg) = arg {
                let arg_ident = if let Pat::Ident(arg_ident) = arg.pat.as_ref() {
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
        if let ReturnType::Type(_, ty) = &handler_signature.output {
            ty.as_ref()
        } else {
            abort!(
                handler_signature.output.span(),
                "Failed to parse return type"
            );
        }
    }
}

struct HandlerGenerator<'a> {
    service_type: &'a ServiceType<'a>,
    handler: Handler<'a>,
}

impl<'a> HandlerGenerator<'a> {
    fn from(service_type: &'a ServiceType, handler: Handler<'a>) -> Self {
        Self {
            service_type,
            handler,
        }
    }

    fn params_struct_ident(&self) -> Ident {
        Ident::new(
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
        let service_mut = self.handler.receiver.mutability;
        let service_type_path = self.service_type.path;
        let service_type_args = self.service_type.args;
        let service_type_constraints = self.service_type.constraints;
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
            async fn #invocation_func_ident #service_type_args (service: & #service_mut #service_type_path, mut input: &[u8]) -> Vec<u8>
                #service_type_constraints
            {
                let request = #params_struct_ident::decode(&mut input).expect("Failed to decode request");
                let result = service.#handler_func_ident(#(#handler_func_params),*)#await_token;
                return result.encode();
            }
        )
    }
}

struct ServiceType<'a> {
    path: &'a TypePath,
    args: &'a PathArguments,
    constraints: Option<&'a WhereClause>,
}

impl<'a> ServiceType<'a> {
    fn new(r#impl: &'a ItemImpl) -> Self {
        let service_type = r#impl.self_ty.as_ref();
        let path = if let Type::Path(type_path) = service_type {
            type_path
        } else {
            abort!(
                service_type.span(),
                "Failed to parse service type: {}",
                service_type.to_token_stream()
            )
        };
        let args = &path.path.segments.last().unwrap().arguments;
        let constraints = r#impl.generics.where_clause.as_ref();
        Self {
            path,
            args,
            constraints,
        }
    }
}
