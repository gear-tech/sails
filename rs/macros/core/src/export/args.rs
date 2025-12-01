use proc_macro_error::abort;
use syn::{
    Ident, LitBool, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(PartialEq, Debug, Default)]
pub(crate) struct ExportArgs {
    route: Option<String>,
    unwrap_result: bool,
    payable: bool,
}

impl ExportArgs {
    pub fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }

    pub fn unwrap_result(&self) -> bool {
        self.unwrap_result
    }

    pub fn payable(&self) -> bool {
        self.payable
    }
}

impl Parse for ExportArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let punctuated: Punctuated<ImportArg, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut args = Self {
            route: None,
            unwrap_result: false,
            payable: false,
        };
        for arg in punctuated {
            match arg {
                ImportArg::Route(route) => {
                    args.route = Some(route);
                }
                ImportArg::UnwrapResult(unwrap_result) => {
                    args.unwrap_result = unwrap_result;
                }
                ImportArg::Payable => {
                    args.payable = true;
                }
            }
        }
        Ok(args)
    }
}

#[derive(Debug)]
enum ImportArg {
    Route(String),
    UnwrapResult(bool),
    Payable,
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
            "unwrap_result" => {
                if input.parse::<Token![=]>().is_ok()
                    && let Ok(val) = input.parse::<LitBool>()
                {
                    return Ok(Self::UnwrapResult(val.value()));
                }
                Ok(Self::UnwrapResult(true))
            }
            "payable" => Ok(Self::Payable),
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
            payable: false,
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
            payable: false,
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
            payable: false,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn export_parse_args_payable() {
        // arrange
        let input = quote!(payable);
        let expected = ExportArgs {
            route: None,
            unwrap_result: false,
            payable: true,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }
}
