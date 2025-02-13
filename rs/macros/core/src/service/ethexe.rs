// This file is part of Gear.

// Copyright (C) 2021-2025 Gear Technologies Inc.
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

use crate::{
    service::HandlerGenerator,
    shared::{self, Func},
};
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

pub fn service_signature_impl(service_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let (service_type_path, _, _) = shared::impl_type_refs(&service_impl);
    let (generics, service_type_constraints) = shared::impl_constraints(&service_impl);
    let service_handlers = shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    });
    let service_method_routes =
        service_handlers
            .into_iter()
            .map(|(handler_route, (handler_fn, _, _))| {
                handler_signature(handler_route, handler_fn, sails_path)
            });

    quote! {
        impl #generics #sails_path::solidity::ServiceSignature for #service_type_path #service_type_constraints {
            const METHODS: &'static [#sails_path::solidity::MethodRoute] = &[
                #( #service_method_routes )*
            ];
        }
    }
}

fn handler_signature(
    handler_route: String,
    handler_fn: &ImplItemFn,
    sails_path: &Path,
) -> TokenStream {
    let handler_route_bytes = handler_route.encode();
    let handler_name = handler_route.to_case(Case::Snake);
    let handler_func = Func::from(&handler_fn.sig);
    let handler_types = handler_func.params().iter().map(|item| {
        let param_type = item.1;
        quote!(#param_type,)
    });

    quote! {
        (
            #sails_path::concatcp!(
                #handler_name,
                <<(#(#handler_types)*) as #sails_path::alloy_sol_types::SolValue>::SolType as #sails_path::alloy_sol_types::SolType>::SOL_NAME,
            ),
            &[ #(#handler_route_bytes),* ] as &[u8],
        ),
    }
}

pub fn try_handle_impl(service_impl: &ItemImpl, sails_path: &Path) -> TokenStream {
    let service_handlers = shared::discover_invocation_targets(service_impl, |fn_item| {
        matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_some()
    });
    let service_method_branches =
        service_handlers
            .iter()
            .map(|(handler_route, (handler_fn, _, unwrap_result))| {
                try_handle_branch_impl(handler_route, handler_fn, *unwrap_result, sails_path)
            });

    quote! {
        pub async fn try_handle_solidity(
            &mut self,
            method: &[u8],
            input: &[u8],
        ) -> Option<(Vec<u8>, u128)> {
            #( #service_method_branches )*
            None
        }
    }
}

/// Generates code
/// ```rust
/// if method == &[24u8, 68u8, 111u8, 84u8, 104u8, 105u8, 115u8] {
///     // invocation
/// }
/// ```
fn try_handle_branch_impl(
    handler_route: &str,
    handler_fn: &ImplItemFn,
    unwrap_result: bool,
    sails_path: &Path,
) -> TokenStream {
    let handler_route_bytes = handler_route.encode();
    let handler_func = Func::from(&handler_fn.sig);
    let handler_generator = HandlerGenerator::from(handler_func.clone(), unwrap_result);
    let invocation = handler_generator.invocation_func_solidity(sails_path);

    quote! {
        if method == &[ #(#handler_route_bytes),* ] {
            #invocation
        }
    }
}
