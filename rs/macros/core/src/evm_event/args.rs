use syn::{
    Path, Token,
    parse::{Parse, ParseBuffer},
};

pub(super) const SAILS_PATH: &str = "sails_path";

#[derive(Debug, PartialEq)]
pub(super) struct CratePathAttr {
    path: Path,
}

impl CratePathAttr {
    pub fn path(self) -> syn::Path {
        self.path
    }
}

impl Parse for CratePathAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        input.parse::<Token![crate]>()?;
        input.parse::<Token![=]>()?;
        let path = input.parse::<syn::Path>()?;

        Ok(Self { path })
    }
}
