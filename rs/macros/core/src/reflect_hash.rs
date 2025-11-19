use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

// todo [sab] is it used?
pub fn derive_reflect_hash(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let sails_path = crate::sails_paths::sails_path_or_default(None);
    
    let hash_computation = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    let field_hashes = fields.named.iter().map(|field| {
                        let ty = &field.ty;
                        quote! {
                            .update(&<#ty as #sails_path::sails_reflect_hash::ReflectHash>::HASH)
                        }
                    });
                    quote! {
                        #sails_path::keccak_const::Keccak256::new()
                            .update(stringify!(#name).as_bytes())
                            #(#field_hashes)*
                            .finalize()
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_hashes = fields.unnamed.iter().map(|field| {
                        let ty = &field.ty;
                        quote! {
                            .update(&<#ty as #sails_path::sails_reflect_hash::ReflectHash>::HASH)
                        }
                    });
                    quote! {
                        #sails_path::keccak_const::Keccak256::new()
                            .update(stringify!(#name).as_bytes())
                            #(#field_hashes)*
                            .finalize()
                    }
                }
                Fields::Unit => {
                    quote! {
                        #sails_path::keccak_const::Keccak256::new()
                            .update(stringify!(#name).as_bytes())
                            .finalize()
                    }
                }
            }
        }
        Data::Enum(_) => {
            // For enums, the #[event] macro already handles ReflectHash generation
            // This derive should not be used on enums
            return quote! {
                compile_error!("ReflectHash cannot be derived for enums. Use #[event] macro instead for event enums, or implement manually for other enums.");
            };
        }
        Data::Union(_) => {
            return quote! {
                compile_error!("ReflectHash cannot be derived for unions");
            };
        }
    };

    quote! {
        impl #sails_path::sails_reflect_hash::ReflectHash for #name {
            const HASH: [u8; 32] = #hash_computation;
        }
    }
}
