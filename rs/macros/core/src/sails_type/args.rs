use syn::{
    Path, Token,
    parse::{Parse, ParseBuffer},
};

#[derive(Debug, Default, PartialEq)]
pub(super) struct SailsTypeArgs {
    pub path: Option<Path>,
    pub no_reflect_hash: bool,
}

impl Parse for SailsTypeArgs {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let mut out = SailsTypeArgs::default();
        if input.is_empty() {
            return Ok(out);
        }

        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![crate]) {
                input.parse::<Token![crate]>()?;
                input.parse::<Token![=]>()?;
                let path = input.parse::<Path>()?;
                if out.path.is_some() {
                    return Err(syn::Error::new_spanned(
                        &path,
                        "`crate` argument specified more than once",
                    ));
                }
                out.path = Some(path);
            } else if lookahead.peek(syn::Ident) {
                let ident: syn::Ident = input.parse()?;
                if ident == "no_reflect_hash" {
                    if out.no_reflect_hash {
                        return Err(syn::Error::new_spanned(
                            &ident,
                            "`no_reflect_hash` specified more than once",
                        ));
                    }
                    out.no_reflect_hash = true;
                } else {
                    return Err(syn::Error::new_spanned(
                        &ident,
                        format!(
                            "unknown `sails_type` argument `{ident}` (expected `crate = <path>` or `no_reflect_hash`)"
                        ),
                    ));
                }
            } else {
                return Err(lookahead.error());
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(out)
    }
}
