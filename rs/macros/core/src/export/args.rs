use proc_macro_error::abort;
use syn::{
    Ident, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(PartialEq, Debug, Default)]
pub(crate) struct ExportArgs {
    route: Option<String>,
    #[cfg(feature = "ethexe")]
    payable: bool,
    overrides: Option<Path>,
    entry_id: Option<u16>,
}

impl ExportArgs {
    pub fn route(&self) -> Option<&str> {
        self.route.as_deref()
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
}

impl Parse for ExportArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let punctuated: Punctuated<ImportArg, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut args = Self {
            route: None,
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: None,
            entry_id: None,
        };
        for arg in punctuated {
            match arg {
                ImportArg::Route(route) => {
                    args.route = Some(route);
                }
                #[cfg(feature = "ethexe")]
                ImportArg::Payable => {
                    args.payable = true;
                }
                ImportArg::Overrides(path) => {
                    args.overrides = Some(path);
                }
                ImportArg::EntryId(entry_id) => {
                    args.entry_id = Some(entry_id);
                }
            }
        }
        Ok(args)
    }
}

#[derive(Debug)]
enum ImportArg {
    Route(String),
    #[cfg(feature = "ethexe")]
    Payable,
    Overrides(Path),
    EntryId(u16),
}

impl Parse for ImportArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<Path>()?;
        let ident = path.get_ident().unwrap();
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
            #[cfg(feature = "ethexe")]
            "payable" => Ok(Self::Payable),
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
        let input = quote!(route = "CallMe");
        let expected = ExportArgs {
            route: Some("CallMe".to_owned()),
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: None,
            entry_id: None,
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
            payable: true,
            overrides: None,
            entry_id: None,
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
            #[cfg(feature = "ethexe")]
            payable: false,
            overrides: Some(expected_path),
            entry_id: Some(42),
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }
}
