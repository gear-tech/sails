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
    entry_id: Option<u16>,
}

impl ExportArgs {
    pub fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }

    pub fn unwrap_result(&self) -> bool {
        self.unwrap_result
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
            unwrap_result: false,
            entry_id: None,
        };
        for arg in punctuated {
            match arg {
                ImportArg::Route(route) => {
                    args.route = Some(route);
                }
                ImportArg::UnwrapResult(unwrap_result) => {
                    args.unwrap_result = unwrap_result;
                }
                ImportArg::EntryId(entry_id) => {
                    if args.entry_id.replace(entry_id).is_some() {
                        abort!(input.span(), "duplicate `entry_id` argument");
                    }
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
            "unwrap_result" => {
                if input.parse::<Token![=]>().is_ok()
                    && let Ok(val) = input.parse::<LitBool>()
                {
                    return Ok(Self::UnwrapResult(val.value()));
                }
                Ok(Self::UnwrapResult(true))
            }
            "entry_id" => {
                input.parse::<Token![=]>()?;
                let lit = input.parse::<syn::LitInt>().unwrap_or_else(|err| {
                    abort!(ident, "unexpected value for `entry_id` argument: {}", err)
                });
                let value = lit.base10_parse::<u32>().unwrap_or_else(|err| {
                    abort!(lit.span(), "`entry_id` must be a positive integer: {}", err)
                });
                if value > u16::MAX as u32 {
                    abort!(lit.span(), "`entry_id` value exceeds u16 range");
                }
                Ok(Self::EntryId(value as u16))
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
        let input = quote!(route = "CallMe", unwrap_result);
        let expected = ExportArgs {
            route: Some("CallMe".to_owned()),
            unwrap_result: true,
            entry_id: None,
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
            entry_id: None,
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
            entry_id: None,
        };

        // act
        let args = syn::parse2::<ExportArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn export_parse_args_with_entry_id() {
        let input = quote!(route = "CallMe", entry_id = 7);
        let expected = ExportArgs {
            route: Some("CallMe".to_owned()),
            unwrap_result: false,
            entry_id: Some(7),
        };

        let args = syn::parse2::<ExportArgs>(input).unwrap();

        assert_eq!(expected, args);
    }
}
