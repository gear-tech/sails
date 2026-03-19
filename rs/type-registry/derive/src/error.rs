use thiserror::Error;

#[derive(Error, Debug)]
pub enum MacroError {
    #[error("Unions are not supported by SailsTypeRegistry")]
    UnsupportedUnion,
    #[error(
        "Unsupported literal type. Only string literals are supported (e.g. `doc = \"text\"`)."
    )]
    UnsupportedLiteralType,
}

impl MacroError {
    pub fn into_syn_error(self, span: proc_macro2::Span) -> syn::Error {
        syn::Error::new(span, self.to_string())
    }
}
