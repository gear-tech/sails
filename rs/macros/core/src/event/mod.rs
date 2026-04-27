use crate::sails_paths::sails_path_or_default;
use args::{CratePathAttr, EventArgs, SAILS_PATH};
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

    // Sort variants alphabetically to ensure deterministic order for hashing and routing
    let mut variants: Vec<_> = input.variants.into_iter().collect();
    variants.sort_by_key(|v| v.ident.to_string().to_lowercase());
    input.variants = variants.into_iter().collect();

    // Parse the attributes into a syntax tree.
    let args = syn::parse2::<EventArgs>(attrs)
        .unwrap_or_else(|err| abort!(err.span(), "invalid `event` arguments: {}", err));
    let sails_path = &sails_path_or_default(args.crate_path.clone());

    // Determine codec annotation for single-codec events.
    #[cfg(feature = "ethexe")]
    let codec_ann: Option<&str> = match (args.scale(), args.ethabi()) {
        (true, false) => Some("scale"),
        (false, true) => Some("ethabi"),
        _ => None,
    };
    #[cfg(not(feature = "ethexe"))]
    let codec_ann: Option<&str> = None;

    if let Some(codec) = codec_ann {
        annotate_variants_with_codec(&mut input, codec);
    }

    let event_impl = if args.scale() {
        generate_sails_event_impl(&input, sails_path)
    } else {
        quote!()
    };

    #[cfg(feature = "ethexe")]
    let eth_event_impl = if args.ethabi() {
        ethexe::generate_eth_event_impl(&input, sails_path)
    } else {
        quote!()
    };
    #[cfg(feature = "ethexe")]
    ethexe::process_indexed(&mut input);
    #[cfg(not(feature = "ethexe"))]
    let eth_event_impl = quote!();

    quote! {
        #input

        #event_impl

        #eth_event_impl
    }
}

fn annotate_variants_with_codec(input: &mut ItemEnum, codec: &str) {
    for variant in &mut input.variants {
        let span = variant.ident.span();
        variant.attrs.push(syn::parse_quote_spanned! { span =>
            #[annotate(codec = #codec)]
        });
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
    let mut entry_id_arms = Vec::new();

    for (idx, variant) in variants.iter().enumerate() {
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

        let idx = idx as u16;
        entry_id_arms.push(quote! {
            #pattern => #idx
        });
    }

    // Generate the impl block for `Event`
    quote! {
         impl #sails_path::SailsEvent for #enum_ident {
            fn encoded_event_name(&self) -> &'static [u8] {
                match self {
                    #( #match_arms ),*
                }
            }

            fn entry_id(&self) -> u16 {
                match self {
                    #( #entry_id_arms ),*
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
