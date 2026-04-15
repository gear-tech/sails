use crate::sails_paths::sails_path_or_default;
use args::SailsTypeArgs;
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, Path};

mod args;

pub fn sails_type(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let args: SailsTypeArgs = syn::parse2(attrs)
        .unwrap_or_else(|err| abort!(err.span(), "invalid `sails_type` arguments: {}", err));

    let parsed: Item = syn::parse2(item).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`sails_type` can only be applied to structs or enums: {}",
            err
        )
    });

    match &parsed {
        Item::Struct(_) | Item::Enum(_) => {}
        other => abort!(
            other,
            "`sails_type` can only be applied to structs or enums"
        ),
    }

    let sails_path: Path = sails_path_or_default(args.path);
    let scale_codec = quote! { #sails_path::scale_codec };
    let type_info = quote! { #sails_path::type_info };

    let (derive_list, reflect_hash_attr) = if args.no_reflect_hash {
        (
            quote! { #scale_codec::Encode, #scale_codec::Decode, #type_info::TypeInfo },
            quote! {},
        )
    } else {
        (
            quote! {
                #scale_codec::Encode,
                #scale_codec::Decode,
                #type_info::TypeInfo,
                #sails_path::ReflectHash
            },
            quote! { #[reflect_hash(crate = #sails_path)] },
        )
    };

    quote! {
        #[derive(#derive_list)]
        #[codec(crate = #scale_codec)]
        #[type_info(crate = #type_info)]
        #reflect_hash_attr
        #parsed
    }
}
