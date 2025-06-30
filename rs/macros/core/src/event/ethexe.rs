use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Fields, Ident, ItemEnum, Path, Type, Variant};

pub(super) fn generate_eth_event_impl(input: &ItemEnum, sails_path: &Path) -> TokenStream {
    let enum_ident = &input.ident;

    // Vectors to collect match arms for topics and data.
    let mut sigs_const = Vec::new();
    let mut topics_match_arms = Vec::new();
    let mut data_match_arms = Vec::new();

    // Process each variant.
    for (idx, variant) in input.variants.iter().enumerate() {
        let variant_ident = &variant.ident;
        check_forbidden_event_idents(variant_ident);

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
                if unnamed.unnamed.iter().any(is_indexed) {
                    abort!(
                        unnamed,
                        "unnamed fields cannot be `#[indexed]`, use named fields instead"
                    );
                }

                let non_idx_exprs: Vec<TokenStream> = unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Ident::new(&format!("f{i}"), Span::call_site()))
                    .map(|field_ident| quote!(#field_ident))
                    .collect();
                // Build the pattern: Enum::Variant(f0, f1, ...)
                let pat = quote! {
                    #enum_ident::#variant_ident( #( #non_idx_exprs ),* )
                };

                (pat, vec![], non_idx_exprs)
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
                let mut topics = #sails_path::Vec::with_capacity( #cap );
                let (_, _, hash) = Self::SIGNATURES[ # idx ];
                topics.push(#sails_path::alloy_primitives::B256::new(hash));
                #( topics.push(#indexed_exprs); )*
                topics
            }
        };
        topics_match_arms.push(topics_arm);

        // Build the data match arm: non-indexed fields are ABI-encoded as a tuple.
        data_match_arms.push(quote! {
            #pattern => {
                Self::encode_sequence(&( #( #non_indexed_exprs, )* ))
            }
        });
    }

    // Generate the implementation for the EthEvent trait.
    quote! {
        impl #sails_path::EthEvent for #enum_ident {
            const SIGNATURES: &'static [#sails_path::gstd::EthEventExpo] = &[
                #( #sigs_const ),*
            ];

            #[allow(unused_variables)]
            fn topics(&self) -> #sails_path::Vec<#sails_path::alloy_primitives::B256> {
                match self {
                    #( #topics_match_arms ),*
                }
            }

            #[allow(unused_variables)]
            fn data(&self) -> #sails_path::Vec<u8> {
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
    let sol_types = quote! {
        <<( #( #field_types, )* ) as #sails_path::alloy_sol_types::SolValue>::SolType as #sails_path::alloy_sol_types::SolType>::SOL_NAME
    };

    quote! {
        (
            #variant_ident,
            #sol_types,
            #sails_path::keccak_const::Keccak256::new().update(#variant_ident .as_bytes()).update(#sol_types  .as_bytes()).finalize()
        )
    }
}

fn check_forbidden_event_idents(ident: &Ident) {
    const FORBIDDEN_IDENTS: &[&str] = &[
        "ExecutableBalanceTopUpRequested",
        "Message",
        "MessageQueueingRequested",
        "Reply",
        "ReplyQueueingRequested",
        "StateChanged",
        "ValueClaimed",
        "ValueClaimingRequested",
    ];
    if FORBIDDEN_IDENTS.contains(&ident.to_string().as_str()) {
        abort!(
            ident,
            "`{}` is not allowed as an identifier of EthEvent variant, not allowed identifiers are: {:?}",
            ident,
            FORBIDDEN_IDENTS
        );
    }
}

pub(super) fn process_indexed(input: &mut ItemEnum) {
    // Process each variant.
    for variant in input.variants.iter_mut() {
        match &mut variant.fields {
            // For named fields, use the field identifiers directly.
            Fields::Named(named) => {
                for field in named.named.iter_mut().filter(|f| is_indexed(f)) {
                    remove_indexed_and_add_comment(field);
                }
            }
            // For unnamed (tuple) fields, create synthetic identifiers.
            Fields::Unnamed(unnamed) => {
                for field in unnamed.unnamed.iter_mut().filter(|f| is_indexed(f)) {
                    remove_indexed_and_add_comment(field);
                }
            }
            // For unit variants, no fields exist.
            Fields::Unit => {}
        }
    }
}

/// remove indexed attribute from field and add comment
fn remove_indexed_and_add_comment(field: &mut syn::Field) {
    field.attrs.retain(|attr| !attr.path().is_ident("indexed"));
    field.attrs.push(syn::parse_quote! {
        #[doc = r" #[indexed]"]
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_process_indexed() {
        // arrange
        let input = quote! {
            pub enum SomeService {
                SomeEvent
                {
                    /// Some comment
                    #[indexed]
                    sender: u128
                },
            }
        };
        let mut input: ItemEnum = syn::parse2(input).unwrap();

        let expected = quote! {
            pub enum SomeService {
                SomeEvent
                {
                    /// Some comment
                    /// #[indexed]
                    sender: u128
                },
            }
        };
        let expected: ItemEnum = syn::parse2(expected).unwrap();

        // act
        process_indexed(&mut input);

        // arrange
        assert_eq!(expected, input);
    }
}
