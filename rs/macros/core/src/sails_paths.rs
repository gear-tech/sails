use syn::parse_quote;

const SAILS: &str = "sails_rs";

pub(crate) trait SailsPath {
    fn sails_custom_path(&self) -> Option<syn::Path>;

    fn sails_path(&self) -> syn::Path {
        self.sails_custom_path()
            .unwrap_or_else(|| syn::parse_str(SAILS).unwrap())
    }

    fn scale_codec_path(&self) -> syn::Path {
        let sails_path = self.sails_path();
        parse_quote!(#sails_path::scale_codec)
    }

    fn scale_info_path(&self) -> syn::Path {
        let sails_path = self.sails_path();
        parse_quote!(#sails_path::scale_info)
    }
}
