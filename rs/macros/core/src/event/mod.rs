use crate::sails_paths::sails_path_or_default;
use args::{CratePathAttr, SAILS_PATH};
use parity_scale_codec::Encode;
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, ItemEnum, Path, parse::Parse};

mod args;
#[cfg(feature = "ethexe")]
mod ethexe;

pub fn event(attrs: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    #[cfg_attr(not(feature = "ethexe"), allow(unused_mut))]
    let mut input: ItemEnum = syn::parse2(input).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`event` attribute can be applied to enums only: {}",
            err
        )
    });

    // Parse the attributes into a syntax tree.
    let sails_path_attr = syn::parse2::<CratePathAttr>(attrs).ok();
    let sails_path = &sails_path_or_default(sails_path_attr.map(|attr| attr.path()));

    let event_impl = generate_sails_event_impl(&input, sails_path);
    let reflect_hash_impl = generate_reflect_hash_impl(&input, sails_path);

    #[cfg(feature = "ethexe")]
    let eth_event_impl = ethexe::generate_eth_event_impl(&input, sails_path);
    #[cfg(feature = "ethexe")]
    ethexe::process_indexed(&mut input);
    #[cfg(not(feature = "ethexe"))]
    let eth_event_impl = quote!();

    quote! {
        #input

        #event_impl

        #reflect_hash_impl

        #eth_event_impl
    }
}

fn generate_sails_event_impl(input: &ItemEnum, sails_path: &Path) -> TokenStream {
    // Parse the input enum
    let enum_ident = &input.ident;
    let variants = &input.variants;
    // Check that the enum has at most 256 variants
    if variants.len() > 256 {
        abort!(
            input,
            "`event` enum can have at most 256 variants, but found {}",
            variants.len()
        )
    }

    // Build match arms for each variant
    let mut match_arms = Vec::new();

    for variant in variants {
        let variant_ident = &variant.ident;
        // Determine the pattern to match this variant, ignoring its fields:
        let pattern = match &variant.fields {
            Fields::Unit => {
                // Unit variant: `Enum::Variant`
                quote! { #enum_ident::#variant_ident }
            }
            Fields::Unnamed(_) => {
                // Tuple variant: `Enum::Variant(..)`
                quote! { #enum_ident::#variant_ident ( .. ) }
            }
            Fields::Named(_) => {
                // Struct variant: `Enum::Variant { .. }`
                quote! { #enum_ident::#variant_ident { .. } }
            }
        };
        // Encode the variant identifier as a sequence of u8
        let encoded_name = variant_ident.to_string().encode();

        // Build the match arm: pattern => &[ ... ],
        let arm = quote! {
            #pattern => &[ #( #encoded_name ),* ]
        };
        match_arms.push(arm);
    }

    // Generate the impl block for `Event`
    quote! {
         impl #sails_path::SailsEvent for #enum_ident {
            fn encoded_event_name(&self) -> &'static [u8] {
                match self {
                    #( #match_arms ),*
                }
            }

            fn skip_bytes() -> usize {
                1 // The first byte is reserved for the index of the event enum variant
            }
        }
    }
}

fn generate_reflect_hash_impl(input: &ItemEnum, sails_path: &Path) -> TokenStream {
    let enum_ident = &input.ident;
    let variants = &input.variants;

    // Build the hash computation for each variant
    let mut variant_hash_computations = Vec::new();

    for variant in variants {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        let variant_hash = match &variant.fields {
            Fields::Unit => {
                // Unit variant: hash(b"VariantName")
                quote! {
                    #sails_path::keccak_const::Keccak256::new()
                        .update(#variant_name.as_bytes())
                        .finalize()
                }
            }
            Fields::Unnamed(fields) => {
                // Tuple variant: hash(b"VariantName" || T1::HASH || T2::HASH || ...)
                let field_hashes = fields.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote! {
                        .update(&<#ty as #sails_path::sails_reflect_hash::ReflectHash>::HASH)
                    }
                });
                quote! {
                    #sails_path::keccak_const::Keccak256::new()
                        .update(#variant_name.as_bytes())
                        #(#field_hashes)*
                        .finalize()
                }
            }
            Fields::Named(fields) => {
                // Named variant: hash(b"VariantName" || T1::HASH || T2::HASH || ...)
                // Field names are ignored for structural hashing
                let field_hashes = fields.named.iter().map(|field| {
                    let ty = &field.ty;
                    quote! {
                        .update(&<#ty as #sails_path::sails_reflect_hash::ReflectHash>::HASH)
                    }
                });
                quote! {
                    #sails_path::keccak_const::Keccak256::new()
                        .update(#variant_name.as_bytes())
                        #(#field_hashes)*
                        .finalize()
                }
            }
        };

        variant_hash_computations.push(quote! {
            let variant_hash = #variant_hash;
            hasher = hasher.update(&variant_hash);
        });
    }

    quote! {
        impl #sails_path::sails_reflect_hash::ReflectHash for #enum_ident {
            const HASH: [u8; 32] = {
                let mut hasher = #sails_path::keccak_const::Keccak256::new();
                #(#variant_hash_computations)*
                hasher.finalize()
            };
        }
    }
}

pub fn derive_sails_event(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: ItemEnum = syn::parse2(input).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`SailsEvent` can only be derived for enums: {}",
            err
        )
    });

    let sails_path_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(SAILS_PATH))
        .map(|attr| {
            attr.parse_args_with(CratePathAttr::parse)
                .unwrap_or_else(|_| abort!(attr, "unexpected value for `crate` argument",))
        });
    let sails_path = &sails_path_or_default(sails_path_attr.map(|attr| attr.path()));

    generate_sails_event_impl(&input, sails_path)
}
