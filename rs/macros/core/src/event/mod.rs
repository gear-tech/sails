use crate::sails_paths::sails_path_or_default;
use args::{CratePathAttr, SAILS_PATH};
use parity_scale_codec::Encode;
use proc_macro_error::abort;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::collections::BTreeSet;
use syn::{Attribute, Expr, ExprLit, Fields, ItemEnum, Meta, Path, parse::Parse};

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

    let event_entry_ids = extract_event_entry_ids(&mut input);
    let event_impl = generate_sails_event_impl(&input, sails_path);
    let enum_ident = &input.ident;
    let event_entry_ids_impl =
        generate_event_entry_ids_impl(enum_ident, &event_entry_ids, sails_path);

    #[cfg(feature = "ethexe")]
    let eth_event_impl = ethexe::generate_eth_event_impl(&input, sails_path);
    #[cfg(feature = "ethexe")]
    ethexe::process_indexed(&mut input);
    #[cfg(not(feature = "ethexe"))]
    let eth_event_impl = quote!();

    quote! {
        #input

        #event_impl

        #event_entry_ids_impl

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

fn extract_event_entry_ids(input: &mut ItemEnum) -> Vec<u16> {
    // Collect variant info: (original_index, name, explicit_code)
    let mut variant_info: Vec<(usize, String, Option<u16>)> = Vec::new();

    for (idx, variant) in input.variants.iter_mut().enumerate() {
        let mut code_attr = None;
        let mut retained_attrs = Vec::new();

        for attr in variant.attrs.drain(..) {
            if attr.path().is_ident("event_code") {
                if code_attr.is_some() {
                    abort!(attr, "duplicate `event_code` attribute");
                }
                code_attr = Some(parse_event_code(&attr));
            } else {
                retained_attrs.push(attr);
            }
        }

        variant.attrs = retained_attrs;
        variant_info.push((idx, variant.ident.to_string(), code_attr));
    }

    // Sort by name (lexicographic)
    variant_info.sort_by(|a, b| a.1.cmp(&b.1));

    // Assign entry IDs in sorted order
    let mut codes_by_index = vec![0u16; variant_info.len()];
    let mut seen = BTreeSet::new();
    let mut next: u16 = 1;

    // First, collect all explicitly assigned codes
    for (_, _, code_attr) in &variant_info {
        if let Some(code) = code_attr {
            if !seen.insert(*code) {
                // This will be caught in the second loop with proper error location
            }
        }
    }

    // Now assign codes in sorted name order
    for (original_idx, _name, code_attr) in variant_info {
        let code = if let Some(explicit_code) = code_attr {
            // Check for duplicates
            if seen.iter().filter(|&&c| c == explicit_code).count() > 1 {
                abort!(
                    input.variants[original_idx],
                    "duplicate `event_code` value `{explicit_code}` within event enum"
                );
            }
            explicit_code
        } else {
            // Auto-assign next available ID
            while seen.contains(&next) {
                next = next.wrapping_add(1);
            }
            let value = next;
            seen.insert(value);
            next = next.wrapping_add(1);
            value
        };

        codes_by_index[original_idx] = code;
    }

    codes_by_index
}

fn parse_event_code(attr: &Attribute) -> u16 {
    let meta = attr.meta.clone();
    let Meta::NameValue(name_value) = meta else {
        abort!(
            attr,
            "`event_code` must be in the form `#[event_code = <u16>]`"
        );
    };
    let Expr::Lit(ExprLit { lit, .. }) = name_value.value else {
        abort!(name_value.value, "`event_code` must be an integer literal");
    };
    let syn::Lit::Int(lit_int) = lit else {
        abort!(lit, "`event_code` must be an integer literal");
    };
    let value = lit_int
        .base10_parse::<u32>()
        .unwrap_or_else(|err| abort!(lit_int.span(), "failed to parse `event_code`: {}", err));
    if value > u16::MAX as u32 {
        abort!(lit_int.span(), "`event_code` value exceeds u16 range");
    }
    value as u16
}

fn generate_event_entry_ids_impl(
    enum_ident: &syn::Ident,
    entry_ids: &[u16],
    sails_path: &Path,
) -> TokenStream {
    let push_statements = entry_ids.iter().map(|entry_id| {
        let literal = Literal::u16_unsuffixed(*entry_id);
        quote!(ids.push(#literal);)
    });

    quote! {
        impl #sails_path::meta::EventEntryIdMeta for #enum_ident {
            fn event_entry_ids() -> #sails_path::Vec<u16> {
                let mut ids = #sails_path::Vec::new();
                #( #push_statements )*
                ids
            }
        }
    }
}
