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

#[derive(Debug, PartialEq)]
pub(super) struct EventArgs {
    pub crate_path: Option<Path>,
    pub scale: bool,
    #[cfg(feature = "ethexe")]
    pub ethabi: bool,
}

impl EventArgs {
    pub fn scale(&self) -> bool {
        self.scale
    }

    #[cfg(feature = "ethexe")]
    pub fn ethabi(&self) -> bool {
        self.ethabi
    }
}

impl Parse for EventArgs {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let mut crate_path: Option<Path> = None;
        let mut scale = false;
        #[cfg(feature = "ethexe")]
        let mut ethabi = false;
        let mut any_transport_flag_seen = false;
        let mut scale_seen = false;
        #[cfg(feature = "ethexe")]
        let mut ethabi_seen = false;

        if input.is_empty() {
            return Ok(EventArgs {
                crate_path,
                scale: true,
                #[cfg(feature = "ethexe")]
                ethabi: true,
            });
        }

        loop {
            if input.is_empty() {
                break;
            }

            let lookahead = input.lookahead1();
            if lookahead.peek(Token![crate]) {
                input.parse::<Token![crate]>()?;
                input.parse::<Token![=]>()?;
                let path = input.parse::<Path>()?;
                if crate_path.is_some() {
                    return Err(syn::Error::new_spanned(
                        &path,
                        "duplicate `crate` argument in `#[event]`",
                    ));
                }
                crate_path = Some(path);
            } else if lookahead.peek(syn::Ident) {
                let ident: syn::Ident = input.parse()?;
                match ident.to_string().as_str() {
                    "scale" => {
                        if scale_seen {
                            return Err(syn::Error::new_spanned(
                                &ident,
                                "duplicate `scale` flag in `#[event]`",
                            ));
                        }
                        scale_seen = true;
                        any_transport_flag_seen = true;
                        scale = true;
                    }
                    #[cfg(feature = "ethexe")]
                    "ethabi" => {
                        if ethabi_seen {
                            return Err(syn::Error::new_spanned(
                                &ident,
                                "duplicate `ethabi` flag in `#[event]`",
                            ));
                        }
                        ethabi_seen = true;
                        any_transport_flag_seen = true;
                        ethabi = true;
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            &ident,
                            format!("unknown `event` argument `{other}`"),
                        ));
                    }
                }
            } else {
                return Err(lookahead.error());
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        if !any_transport_flag_seen {
            scale = true;
            #[cfg(feature = "ethexe")]
            {
                ethabi = true;
            }
        }

        Ok(EventArgs {
            crate_path,
            scale,
            #[cfg(feature = "ethexe")]
            ethabi,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn event_args_default() {
        let args = syn::parse2::<EventArgs>(quote!()).unwrap();
        assert!(args.scale());
        #[cfg(feature = "ethexe")]
        assert!(args.ethabi());
    }

    #[test]
    fn event_args_crate_path() {
        let args = syn::parse2::<EventArgs>(quote!(crate = sails_rename)).unwrap();
        assert!(args.scale());
        assert!(args.crate_path.is_some());
    }

    #[test]
    fn event_args_crate_and_scale() {
        let args = syn::parse2::<EventArgs>(quote!(crate = sails_rename, scale)).unwrap();
        assert!(args.scale());
        #[cfg(feature = "ethexe")]
        assert!(!args.ethabi());
        assert!(args.crate_path.is_some());
    }

    #[test]
    fn event_args_crate_path_allows_trailing_comma() {
        let args = syn::parse2::<EventArgs>(quote!(crate = sails_rename,)).unwrap();
        assert!(args.scale());
        assert!(args.crate_path.is_some());
    }

    #[test]
    fn event_args_scale_only() {
        let args = syn::parse2::<EventArgs>(quote!(scale)).unwrap();
        assert!(args.scale());
        #[cfg(feature = "ethexe")]
        assert!(!args.ethabi());
    }

    #[test]
    fn event_args_scale_allows_trailing_comma() {
        let args = syn::parse2::<EventArgs>(quote!(scale,)).unwrap();
        assert!(args.scale());
        #[cfg(feature = "ethexe")]
        assert!(!args.ethabi());
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn event_args_ethabi_only() {
        let args = syn::parse2::<EventArgs>(quote!(ethabi)).unwrap();
        assert!(!args.scale());
        assert!(args.ethabi());
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn event_args_scale_and_ethabi() {
        let args = syn::parse2::<EventArgs>(quote!(scale, ethabi)).unwrap();
        assert!(args.scale());
        assert!(args.ethabi());
    }

    #[test]
    fn event_args_duplicate_crate_errors() {
        let err = syn::parse2::<EventArgs>(quote!(crate = a, crate = b)).unwrap_err();
        assert!(err.to_string().contains("duplicate `crate`"));
    }

    #[test]
    fn event_args_duplicate_scale_errors() {
        let err = syn::parse2::<EventArgs>(quote!(scale, scale)).unwrap_err();
        assert!(err.to_string().contains("duplicate `scale` flag"));
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn event_args_duplicate_ethabi_errors() {
        let err = syn::parse2::<EventArgs>(quote!(ethabi, ethabi)).unwrap_err();
        assert!(err.to_string().contains("duplicate `ethabi` flag"));
    }

    #[test]
    fn event_args_unknown_argument_errors() {
        let err = syn::parse2::<EventArgs>(quote!(foobar)).unwrap_err();
        assert!(err.to_string().contains("unknown `event` argument"));
    }
}
