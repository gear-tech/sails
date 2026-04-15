use proc_macro_error::abort;
use syn::{
    Ident, LitBool, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(PartialEq, Debug)]
pub(crate) struct ExportArgs {
    route: Option<String>,
    unwrap_result: bool,
    #[cfg(feature = "ethexe")]
    payable: bool,
    overrides: Option<Path>,
    entry_id: Option<u16>,
    scale: bool,
    #[cfg(feature = "ethexe")]
    ethabi: bool,
}

impl Default for ExportArgs {
    fn default() -> Self {
        Self {
            route: None,
            unwrap_result: false,
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: None,
            entry_id: None,
            scale: true,
            #[cfg(feature = "ethexe")]
            ethabi: true,
        }
    }
}

impl ExportArgs {
    pub fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }

    pub fn unwrap_result(&self) -> bool {
        self.unwrap_result
    }

    #[cfg(feature = "ethexe")]
    pub fn payable(&self) -> bool {
        self.payable
    }

    pub fn overrides(&self) -> Option<&Path> {
        self.overrides.as_ref()
    }

    pub fn entry_id(&self) -> Option<u16> {
        self.entry_id
    }

    pub fn scale(&self) -> bool {
        self.scale
    }

    #[cfg(feature = "ethexe")]
    pub fn ethabi(&self) -> bool {
        self.ethabi
    }
}

impl Parse for ExportArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let punctuated: Punctuated<ImportArg, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut args = Self {
            route: None,
            unwrap_result: false,
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: None,
            entry_id: None,
            scale: false,
            #[cfg(feature = "ethexe")]
            ethabi: false,
        };
        let mut any_transport_flag_seen = false;
        let mut scale_seen = false;
        #[cfg(feature = "ethexe")]
        let mut ethabi_seen = false;
        #[cfg(feature = "ethexe")]
        let mut payable_span: Option<proc_macro2::Span> = None;

        for arg in punctuated {
            match arg {
                ImportArg::Route(route) => {
                    args.route = Some(route);
                }
                ImportArg::UnwrapResult(unwrap_result) => {
                    args.unwrap_result = unwrap_result;
                }
                #[cfg(feature = "ethexe")]
                ImportArg::Payable(span) => {
                    args.payable = true;
                    payable_span = Some(span);
                }
                ImportArg::Overrides(path) => {
                    args.overrides = Some(path);
                }
                ImportArg::EntryId(entry_id) => {
                    args.entry_id = Some(entry_id);
                }
                ImportArg::Scale(span) => {
                    if scale_seen {
                        return Err(syn::Error::new(
                            span,
                            "duplicate `scale` flag in `#[export]`",
                        ));
                    }
                    scale_seen = true;
                    any_transport_flag_seen = true;
                    args.scale = true;
                }
                #[cfg(feature = "ethexe")]
                ImportArg::Ethabi(span) => {
                    if ethabi_seen {
                        return Err(syn::Error::new(
                            span,
                            "duplicate `ethabi` flag in `#[export]`",
                        ));
                    }
                    ethabi_seen = true;
                    any_transport_flag_seen = true;
                    args.ethabi = true;
                }
            }
        }

        if !any_transport_flag_seen {
            args.scale = true;
            #[cfg(feature = "ethexe")]
            {
                args.ethabi = true;
            }
        }

        #[cfg(feature = "ethexe")]
        if let Some(span) = payable_span
            && !args.ethabi
        {
            return Err(syn::Error::new(
                span,
                "`payable` requires `ethabi` transport; write `#[export(ethabi, payable)]` or `#[export(scale, ethabi, payable)]`",
            ));
        }

        Ok(args)
    }
}

#[derive(Debug)]
enum ImportArg {
    Route(String),
    UnwrapResult(bool),
    #[cfg(feature = "ethexe")]
    Payable(proc_macro2::Span),
    Overrides(Path),
    EntryId(u16),
    Scale(proc_macro2::Span),
    #[cfg(feature = "ethexe")]
    Ethabi(proc_macro2::Span),
}

impl Parse for ImportArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<Path>()?;
        let ident = path.get_ident().unwrap();
        let ident_span = ident.span();
        match ident.to_string().as_str() {
            "route" => {
                input.parse::<Token![=]>()?;
                if let Ok(lit) = input.parse::<LitStr>() {
                    let route = lit.value();
                    _ = syn::parse_str::<Ident>(&route).map_err(|err| {
                        abort!(
                            lit.span(),
                            "`route` argument requires a literal with a valid Rust identifier: {}",
                            err
                        )
                    });
                    return Ok(Self::Route(route));
                }
                abort!(ident, "unexpected value for `route` argument: {}", input)
            }
            "unwrap_result" => {
                if input.parse::<Token![=]>().is_ok()
                    && let Ok(val) = input.parse::<LitBool>()
                {
                    return Ok(Self::UnwrapResult(val.value()));
                }
                Ok(Self::UnwrapResult(true))
            }
            #[cfg(feature = "ethexe")]
            "payable" => Ok(Self::Payable(ident_span)),
            "overrides" => {
                input.parse::<Token![=]>()?;
                let path = input.parse::<Path>()?;
                Ok(Self::Overrides(path))
            }
            "entry_id" => {
                input.parse::<Token![=]>()?;
                let lit = input.parse::<syn::LitInt>()?;
                let entry_id = lit.base10_parse::<u16>()?;
                Ok(Self::EntryId(entry_id))
            }
            "scale" => Ok(Self::Scale(ident_span)),
            #[cfg(feature = "ethexe")]
            "ethabi" => Ok(Self::Ethabi(ident_span)),
            _ => abort!(ident, "unknown argument: {}", ident),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn export_parse_args() {
        // arrange
        let input = quote!(route = "CallMe", unwrap_result);
        let expected = ExportArgs {
            route: Some("CallMe".to_owned()),
            unwrap_result: true,
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: None,
            entry_id: None,
            scale: true,
            #[cfg(feature = "ethexe")]
            ethabi: true,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn export_parse_args_unwrap_result() {
        // arrange
        let input = quote!(unwrap_result);
        let expected = ExportArgs {
            route: None,
            unwrap_result: true,
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: None,
            entry_id: None,
            scale: true,
            #[cfg(feature = "ethexe")]
            ethabi: true,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn export_parse_args_unwrap_result_eq_false() {
        // arrange
        let input = quote!(unwrap_result = false);
        let expected = ExportArgs {
            route: None,
            unwrap_result: false,
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: None,
            entry_id: None,
            scale: true,
            #[cfg(feature = "ethexe")]
            ethabi: true,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn export_parse_args_payable() {
        // arrange
        let input = quote!(payable);
        let expected = ExportArgs {
            route: None,
            unwrap_result: false,
            payable: true,
            overrides: None,
            entry_id: None,
            scale: true,
            ethabi: true,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn export_parse_args_overrides() {
        // arrange
        let input = quote!(overrides = BaseService, entry_id = 42);
        let expected_path: Path = syn::parse2(quote!(BaseService)).unwrap();
        let expected = ExportArgs {
            route: None,
            unwrap_result: false,
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: Some(expected_path),
            entry_id: Some(42),
            scale: true,
            #[cfg(feature = "ethexe")]
            ethabi: true,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn export_parse_args_scale_only() {
        let input = quote!(scale);
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        assert!(args.scale());
        #[cfg(feature = "ethexe")]
        assert!(!args.ethabi());
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn export_parse_args_ethabi_only() {
        let input = quote!(ethabi);
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        assert!(!args.scale());
        assert!(args.ethabi());
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn export_parse_args_scale_and_ethabi() {
        let input = quote!(scale, ethabi);
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        assert!(args.scale());
        assert!(args.ethabi());
    }

    #[test]
    fn export_parse_args_default_is_both() {
        let input = quote!();
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        assert!(args.scale());
        #[cfg(feature = "ethexe")]
        assert!(args.ethabi());
    }

    #[test]
    fn export_parse_args_duplicate_scale_errors() {
        let input = quote!(scale, scale);
        let err = syn::parse2::<ExportArgs>(input).unwrap_err();

        assert!(err.to_string().contains("duplicate `scale` flag"));
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn export_parse_args_duplicate_ethabi_errors() {
        let input = quote!(ethabi, ethabi);
        let err = syn::parse2::<ExportArgs>(input).unwrap_err();

        assert!(err.to_string().contains("duplicate `ethabi` flag"));
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn export_parse_args_payable_requires_ethabi() {
        let input = quote!(scale, payable);
        let err = syn::parse2::<ExportArgs>(input).unwrap_err();

        assert!(
            err.to_string()
                .contains("`payable` requires `ethabi` transport")
        );
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn export_parse_args_payable_with_ethabi_ok() {
        let input = quote!(ethabi, payable);
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        assert!(!args.scale());
        assert!(args.ethabi());
        assert!(args.payable());
    }

    #[cfg(feature = "ethexe")]
    #[test]
    fn export_parse_args_plain_payable_ok_under_ethexe() {
        let input = quote!(payable);
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        assert!(args.scale());
        assert!(args.ethabi());
        assert!(args.payable());
    }
}
