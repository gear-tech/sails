//! Procedural macros for deriving `ReflectHash`.
//!
//! This crate provides the `#[derive(ReflectHash)]` macro which generates
//! compile-time structural hashing for Rust types using Keccak256.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Path, parse_macro_input, token};

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

fn derive_reflect_hash_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // Find the actual crate name - first check for #[reflect_hash(crate = ...)] attribute,
    // then fall back to proc-macro-crate detection
    let crate_name = reflect_hash_crate_path(&input.attrs)?;
    
    let hash_computation = match &input.data {
        Data::Struct(data_struct) => {
            generate_struct_hash(name, &data_struct.fields, &crate_name)?
        }
        Data::Enum(data_enum) => {
            generate_enum_hash(&data_enum.variants, &crate_name)?
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "ReflectHash cannot be derived for unions"
            ));
        }
    };
    
    Ok(quote! {
        impl #impl_generics ReflectHash for #name #ty_generics #where_clause {
            const HASH: [u8; 32] = #hash_computation;
        }
    })
}

/// Helper struct to parse `crate = path::to::crate` attribute
struct CratePath {
    _crate_token: token::Crate,
    _eq_token: token::Eq,
    path: Path,
}

impl syn::parse::Parse for CratePath {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(CratePath {
            _crate_token: input.parse()?,
            _eq_token: input.parse()?,
            path: input.parse()?,
        })
    }
}

/// Look for `#[reflect_hash(crate = ...)]` attribute and return the path.
/// If not found, use proc-macro-crate to detect the crate name.
fn reflect_hash_crate_path(attrs: &[Attribute]) -> syn::Result<TokenStream2> {
    // First check for explicit crate path attribute
    for attr in attrs {
        if attr.path().is_ident("reflect_hash") {
            if let Ok(crate_path) = attr.parse_args::<CratePath>() {
                let path = crate_path.path;
                // Check if path starts with :: - if not, it's a local path
                // Use the path as-is since syn::Path handles absolute vs relative correctly
                return Ok(quote!(#path));
            }
        }
    }
    
    // Fall back to proc-macro-crate detection
    match proc_macro_crate::crate_name("sails-reflect-hash") {
        Ok(proc_macro_crate::FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote!(::#ident))
        }
        Err(_) => Ok(quote!(::sails_reflect_hash)),
    }
}

/// Generates hash computation for a struct.
fn generate_struct_hash(name: &syn::Ident, fields: &Fields, crate_name: &TokenStream2) -> syn::Result<TokenStream2> {
    let name_str = name.to_string();
    
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
            let field_hashes = fields_unnamed.unnamed.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    .update(&<#ty as ReflectHash>::HASH)
                }
            });
            
            Ok(quote! {
                #crate_name::keccak_const::Keccak256::new()
                    .update(#name_str.as_bytes())
                    #(#field_hashes)*
                    .finalize()
            })
        }
        Fields::Named(fields_named) => {
            // Named struct: hash(b"StructName" || T1::HASH || T2::HASH || ...)
            // Field names are NOT included in the hash (structural hashing only)
            let field_hashes = fields_named.named.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    .update(&<#ty as ReflectHash>::HASH)
                }
            });
            
            Ok(quote! {
                #crate_name::keccak_const::Keccak256::new()
                    .update(#name_str.as_bytes())
                    #(#field_hashes)*
                    .finalize()
            })
        }
    }
}

/// Generates hash computation for an enum.
fn generate_enum_hash(variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>, crate_name: &TokenStream2) -> syn::Result<TokenStream2> {
    let mut variant_hash_computations = Vec::new();
    
    for variant in variants {
        let variant_hash = generate_variant_hash(variant, crate_name)?;
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

/// Generates hash computation for a single enum variant.
fn generate_variant_hash(variant: &syn::Variant, crate_name: &TokenStream2) -> syn::Result<TokenStream2> {
    let variant_name = variant.ident.to_string();
    
    match &variant.fields {
        Fields::Unit => {
            // Unit variant: hash(b"VariantName")
            Ok(quote! {
                #crate_name::keccak_const::Keccak256::new()
                    .update(#variant_name.as_bytes())
                    .finalize()
            })
        }
        Fields::Unnamed(fields) => {
            // Tuple variant: hash(b"VariantName" || T1::HASH || T2::HASH || ...)
            let field_hashes = fields.unnamed.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    .update(&<#ty as ReflectHash>::HASH)
                }
            });
            
            Ok(quote! {
                #crate_name::keccak_const::Keccak256::new()
                    .update(#variant_name.as_bytes())
                    #(#field_hashes)*
                    .finalize()
            })
        }
        Fields::Named(fields) => {
            // Named variant: hash(b"VariantName" || T1::HASH || T2::HASH || ...)
            // Field names are NOT included in the hash (structural hashing only)
            let field_hashes = fields.named.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    .update(&<#ty as ReflectHash>::HASH)
                }
            });
            
            Ok(quote! {
                #crate_name::keccak_const::Keccak256::new()
                    .update(#variant_name.as_bytes())
                    #(#field_hashes)*
                    .finalize()
            })
        }
    }
}
