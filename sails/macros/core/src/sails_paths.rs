use const_format::concatcp;

const SAILS: &str = "sails";
const SAILS_SCALE_CODEC: &str = concatcp!(SAILS, "::scale_codec");
const SAILS_SCALE_INFO: &str = concatcp!(SAILS, "::scale_info");

pub(crate) fn scale_types_path() -> syn::Path {
    syn::parse_str(SAILS).unwrap()
}

pub(crate) fn scale_codec_path() -> syn::Path {
    syn::parse_str(SAILS_SCALE_CODEC).unwrap()
}

pub(crate) fn scale_info_path() -> syn::Path {
    syn::parse_str(SAILS_SCALE_INFO).unwrap()
}
