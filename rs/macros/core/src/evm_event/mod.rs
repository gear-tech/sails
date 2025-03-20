use crate::sails_paths::sails_path_or_default;
use args::{CratePathAttr, SAILS_PATH};
use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Path, Type, Variant, parse::Parse};

mod args;

pub fn derive_evm_event(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse2(input).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "EvmEvent can only be derived for enums: {}",
            err
        )
    });

    // Ensure the input is an enum.
    let data_enum = match input.data {
        Data::Enum(data) => data,
        _ => abort!(input, "EvmEvent can only be derived for enums"),
    };

    let enum_ident = &input.ident;

    let sails_path_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(SAILS_PATH))
        .map(|attr| {
            attr.parse_args_with(CratePathAttr::parse)
                .unwrap_or_else(|_| abort!(attr, "unexpected value for `crate` argument",))
        });
    let sails_path = &sails_path_or_default(sails_path_attr.map(|attr| attr.path()));

    // Vectors to collect match arms for topics and data.
    let mut sigs_const = Vec::new();
    let mut topics_match_arms = Vec::new();
    let mut data_match_arms = Vec::new();

    // Process each variant.
    for (idx, variant) in data_enum.variants.iter().enumerate() {
        let variant_ident = &variant.ident;

        // Prepare pattern, and vectors for indexed and non-indexed field expressions.
        let (pattern, indexed_exprs, non_indexed_exprs): (
            TokenStream,
            Vec<TokenStream>,
            Vec<TokenStream>,
        ) = match &variant.fields {
            // For named fields, use the field identifiers directly.
            Fields::Named(named) => {
                let field_idents: Vec<&Ident> = named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                // Build the pattern: Enum::Variant { field1, field2, ... }
                let pat = quote! {
                    #enum_ident::#variant_ident { #( #field_idents ),* }
                };

                let mut idx_exprs = Vec::new();
                let mut non_idx_exprs = Vec::new();
                for field in named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    if is_indexed(field) {
                        idx_exprs.push(quote! {
                            Self::topic_hash(#field_ident)
                        });
                    } else {
                        non_idx_exprs.push(quote!(#field_ident));
                    }
                }
                if idx_exprs.len() > 3 {
                    abort!(
                        variant,
                        "too many indexed fields (max 3): {}",
                        idx_exprs.len()
                    );
                }
                (pat, idx_exprs, non_idx_exprs)
            }
            // For unnamed (tuple) fields, create synthetic identifiers.
            Fields::Unnamed(unnamed) => {
                let field_idents: Vec<Ident> = unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Ident::new(&format!("f{}", i), Span::call_site()))
                    .collect();
                // Build the pattern: Enum::Variant(f0, f1, ...)
                let pat = quote! {
                    #enum_ident::#variant_ident( #( #field_idents ),* )
                };

                let mut idx_exprs = Vec::new();
                let mut non_idx_exprs = Vec::new();
                for (i, field) in unnamed.unnamed.iter().enumerate() {
                    let field_ident = &field_idents[i];
                    if is_indexed(field) {
                        idx_exprs.push(quote! {
                            Self::topic_hash(#field_ident)
                        });
                    } else {
                        non_idx_exprs.push(quote!(#field_ident));
                    }
                }
                if idx_exprs.len() > 3 {
                    abort!(
                        variant,
                        "too many indexed fields (max 3): {}",
                        idx_exprs.len()
                    );
                }
                (pat, idx_exprs, non_idx_exprs)
            }
            // For unit variants, no fields exist.
            Fields::Unit => {
                let pat = quote! {
                    #enum_ident::#variant_ident
                };
                (pat, Vec::new(), Vec::new())
            }
        };

        // Build the signature constant.
        let sig = variant_signature(variant, sails_path);
        sigs_const.push(sig);

        // Build the topics match arm.
        let cap = 1 + indexed_exprs.len();
        let topics_arm = quote! {
            #pattern => {
                let mut topics = Vec::with_capacity( #cap );
                topics.push(#sails_path::alloy_primitives::keccak256( Self::SIGNATURES[ # idx ] ));
                #( topics.push(#indexed_exprs); )*
                topics
            }
        };
        topics_match_arms.push(topics_arm);

        // Build the data match arm: non-indexed fields are ABI-encoded as a tuple.
        let data_arm = quote! {
            #pattern => {
                Self::encode_sequence(&( #( #non_indexed_exprs, )* ))
            }
        };
        data_match_arms.push(data_arm);
    }

    // Generate the implementation for the EvmEvent trait.
    quote! {
        impl EvmEvent for #enum_ident {
            const SIGNATURES: &'static [&'static str] = &[
                #( #sigs_const ),*
            ];

            #[allow(unused_variables)]
            fn topics(&self) -> Vec<#sails_path::alloy_primitives::B256> {
                match self {
                    #( #topics_match_arms ),*
                }
            }

            #[allow(unused_variables)]
            fn data(&self) -> Vec<u8> {
                match self {
                    #( #data_match_arms ),*
                }
            }
        }
    }
}

fn is_indexed(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("indexed"))
}

fn variant_signature(variant: &Variant, sails_path: &Path) -> TokenStream {
    let variant_ident = &variant.ident.to_string();
    let field_types: Vec<&Type> = match &variant.fields {
        Fields::Named(named) => named.named.iter().map(|f| &f.ty).collect(),
        Fields::Unnamed(unnamed) => unnamed.unnamed.iter().map(|f| &f.ty).collect(),
        Fields::Unit => Vec::new(),
    };

    quote! {
        concatcp!(
            #variant_ident,
            <<( #( #field_types, )* ) as #sails_path::alloy_sol_types::SolValue>::SolType as #sails_path::alloy_sol_types::SolType>::SOL_NAME,
        )
    }
}
