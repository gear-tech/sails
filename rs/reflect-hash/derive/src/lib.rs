// This file is part of Gear.

// Copyright (C) 2025 Gear Technologies Inc.
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

//! Procedural macros for deriving `ReflectHash`.
//!
//! This crate provides the `#[derive(ReflectHash)]` macro which generates
//! compile-time structural hashing for Rust types using Keccak256.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, Ident, Path, Result as SynResult, Variant,
    parse::ParseStream, parse_macro_input, punctuated::Punctuated, token,
};

/// Derives the `ReflectHash` trait for structs and enums.
///
/// # Hashing Rules
///
/// The hash is computed at compile time using Keccak256 with the following rules:
///
/// ## Enums
///
/// The final hash is `keccak256(variant_hash_0 || variant_hash_1 || ... || variant_hash_N)`
/// where variants are processed in declaration order.
///
/// - **Unit variant** `Transferred` → `keccak256(b"Transferred")`
/// - **Tuple variant** `Approved(ActorId, u128)` → `keccak256(b"Approved" || ActorId::HASH || u128::HASH)`
/// - **Named variant** `Paused { by: ActorId }` → `keccak256(b"Paused" || ActorId::HASH)`
///
/// ## Structs
///
/// - **Unit struct** `struct Empty;` → `keccak256(b"Empty")`
/// - **Tuple struct** `struct Point(u32, u32);` → `keccak256(b"Point" || u32::HASH || u32::HASH)`
/// - **Named struct** `struct User { id: u64, name: String }` → `keccak256(b"User" || u64::HASH || String::HASH)`
///
/// # Examples
///
/// ```ignore
/// use sails_reflect_hash::ReflectHash;
///
/// #[derive(ReflectHash)]
/// struct Transfer {
///     from: ActorId,
///     to: ActorId,
///     amount: u128,
/// }
///
/// #[derive(ReflectHash)]
/// enum Event {
///     Transferred { from: ActorId, to: ActorId },
///     Approved(ActorId, u128),
///     Paused,
/// }
/// ```
///
/// ## Custom crate path
///
/// If `sails-reflect-hash` is re-exported under a different name, you can specify
/// the crate path using the `#[reflect_hash(crate = path)]` attribute:
///
/// ```ignore
/// // Local re-export
/// use sails_reflect_hash as my_hash;
///
/// #[derive(ReflectHash)]
/// #[reflect_hash(crate = my_hash)]
/// struct MyType {
///     field: u32,
/// }
///
/// // Absolute path
/// #[derive(ReflectHash)]
/// #[reflect_hash(crate = ::some::other::path::to::hash)]
/// struct OtherType {
///     field: u64,
/// }
/// ```
#[proc_macro_derive(ReflectHash, attributes(reflect_hash))]
pub fn derive_reflect_hash(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_reflect_hash_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_reflect_hash_impl(input: DeriveInput) -> SynResult<TokenStream2> {
    let ty_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Find the actual crate name - first check for #[reflect_hash(crate = ...)] attribute,
    // then fall back to proc-macro-crate detection
    let crate_name = reflect_hash_crate_path(&input.attrs)?;

    let hash_computation = match &input.data {
        Data::Struct(data_struct) => {
            generate_struct_hash(ty_name, &data_struct.fields, &crate_name)?
        }
        Data::Enum(data_enum) => generate_enum_hash(&data_enum.variants, &crate_name)?,
        Data::Union(_) => {
            return Err(Error::new_spanned(
                ty_name,
                "ReflectHash cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics ReflectHash for #ty_name #ty_generics #where_clause {
            const HASH: [u8; 32] = #hash_computation;
        }
    })
}

// TODO: #1125
/// Look for `#[reflect_hash(crate = ...)]` attribute and return the path.
/// If not found, use proc-macro-crate to detect the crate name.
fn reflect_hash_crate_path(attrs: &[Attribute]) -> SynResult<TokenStream2> {
    // First check for explicit crate path attribute
    for attr in attrs {
        if attr.path().is_ident("reflect_hash") {
            // Parser closure for: crate = some::path
            let parser = |input: ParseStream| -> SynResult<Path> {
                // parse the `crate` keyword
                input.parse::<token::Crate>()?;
                // parse the `=` token
                input.parse::<token::Eq>()?;
                // parse the path after `=`
                input.parse::<Path>()
            };
            let path = attr.parse_args_with(parser)?;

            return Ok(quote!(#path));
        }
    }

    // Fall back to proc-macro-crate detection
    match proc_macro_crate::crate_name("sails-reflect-hash") {
        Ok(proc_macro_crate::FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote!(::#ident))
        }
        Err(e) => Err(Error::new(
            Span::call_site(),
            format!(
                "Could not detect sails-reflect-hash crate: {e}. Consider using #[reflect_hash(crate = path::to::crate)]"
            ),
        )),
    }
}

/// Generates hash computation for a struct.
fn generate_struct_hash(
    ty_name: &Ident,
    fields: &Fields,
    crate_name: &TokenStream2,
) -> SynResult<TokenStream2> {
    let name_str = ty_name.to_string();

    fn fields_hash<'a>(
        fields: impl Iterator<Item = &'a Field>,
        crate_name: &TokenStream2,
        name_str: String,
    ) -> TokenStream2 {
        let field_hashes = fields.map(|field| {
            let ty = &field.ty;
            quote! {
                .update(&<#ty as ReflectHash>::HASH)
            }
        });

        quote! {
            #crate_name::keccak_const::Keccak256::new()
                .update(#name_str.as_bytes())
                #(#field_hashes)*
                .finalize()
        }
    }

    match fields {
        Fields::Unit => {
            // Unit struct: hash(b"StructName")
            Ok(quote! {
                #crate_name::keccak_const::Keccak256::new()
                    .update(#name_str.as_bytes())
                    .finalize()
            })
        }
        Fields::Unnamed(fields_unnamed) => {
            // Tuple struct: hash(b"StructName" || T1::HASH || T2::HASH || ...)
            Ok(fields_hash(
                fields_unnamed.unnamed.iter(),
                crate_name,
                name_str,
            ))
        }
        Fields::Named(fields_named) => {
            // Named struct: hash(b"StructName" || T1::HASH || T2::HASH || ...)
            // Field names are NOT included in the hash (structural hashing only)
            Ok(fields_hash(fields_named.named.iter(), crate_name, name_str))
        }
    }
}

/// Generates hash computation for an enum.
fn generate_enum_hash(
    variants: &Punctuated<Variant, token::Comma>,
    crate_name: &TokenStream2,
) -> SynResult<TokenStream2> {
    let mut variant_hash_computations = Vec::new();

    for variant in variants {
        let variant_hash = generate_struct_hash(&variant.ident, &variant.fields, crate_name)?;
        variant_hash_computations.push(variant_hash);
    }

    Ok(quote! {
        {
            let mut final_hasher = #crate_name::keccak_const::Keccak256::new();
            #(
                {
                    let variant_hash = #variant_hash_computations;
                    final_hasher = final_hasher.update(&variant_hash);
                }
            )*
            final_hasher.finalize()
        }
    })
}
