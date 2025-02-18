use syn::{parse_quote, Path};

pub const SAILS: &str = "sails_rs";

pub(crate) fn sails_path_or_default(sails_custom_path: Option<syn::Path>) -> syn::Path {
    sails_custom_path.unwrap_or_else(|| syn::parse_str(SAILS).unwrap())
}

pub(crate) fn scale_codec_path(sails_path: &Path) -> syn::Path {
    parse_quote!(#sails_path::scale_codec)
}

pub(crate) fn scale_info_path(sails_path: &Path) -> syn::Path {
    parse_quote!(#sails_path::scale_info)
}
