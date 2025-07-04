use crate::sails_paths;
use proc_macro_error::abort;
use syn::{
    LitBool, Path, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(Debug, PartialEq)]
pub(super) struct ProgramArgs {
    handle_signal: Option<Path>,
    sails_path: Option<Path>,
    payable: bool,
    default_sails_path: Path,
}

impl ProgramArgs {
    pub fn handle_signal(&self) -> Option<&Path> {
        self.handle_signal.as_ref()
    }

    pub fn sails_path(&self) -> &syn::Path {
        self.sails_path.as_ref().unwrap_or(&self.default_sails_path)
    }

    pub fn payable(&self) -> bool {
        self.payable
    }
}

impl Parse for ProgramArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let punctuated: Punctuated<ProgramArg, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut attrs = ProgramArgs {
            handle_signal: None,
            sails_path: None,
            payable: false,
            default_sails_path: syn::parse_str(sails_paths::SAILS).unwrap(),
        };

        for arg in punctuated {
            match arg {
                ProgramArg::HandleSignal(path) => {
                    attrs.handle_signal = Some(path);
                }
                ProgramArg::SailsPath(path) => {
                    attrs.sails_path = Some(path);
                }
                ProgramArg::AcceptTransfer(val) => {
                    attrs.payable = val;
                }
            }
        }

        Ok(attrs)
    }
}

enum ProgramArg {
    HandleSignal(Path),
    SailsPath(Path),
    AcceptTransfer(bool),
}

impl Parse for ProgramArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path: Path = input.parse()?;
        let ident = path.get_ident().unwrap();
        match ident.to_string().as_str() {
            "handle_signal" => {
                if cfg!(feature = "ethexe") {
                    abort!(
                        ident,
                        "`handle_signal` is not supported in this build. \
                        Disable the `ethexe` feature to use it.",
                    );
                }
                input.parse::<Token![=]>()?;
                let path: Path = input.parse()?;
                Ok(Self::HandleSignal(path))
            }
            "crate" => {
                input.parse::<Token![=]>()?;
                let path: Path = input.parse()?;
                Ok(Self::SailsPath(path))
            }
            "payable" => {
                if input.parse::<Token![=]>().is_ok() {
                    if let Ok(val) = input.parse::<LitBool>() {
                        return Ok(Self::AcceptTransfer(val.value()));
                    }
                }
                Ok(Self::AcceptTransfer(true))
            }
            _ => abort!(
                ident,
                "`program` attribute can only contain `handle_signal`, `crate`, `payable` parameters",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::quote;
    use syn::{Ident, PathSegment};

    #[test]
    fn gprogram_parse_attrs() {
        // arrange
        let input = quote!(handle_signal = my_handle_signal,);
        let expected = ProgramArgs {
            handle_signal: Some(
                PathSegment::from(Ident::new("my_handle_signal", Span::call_site())).into(),
            ),
            sails_path: None,
            payable: false,
            default_sails_path: syn::parse_str(sails_paths::SAILS).unwrap(),
        };

        // act
        let args = syn::parse2::<ProgramArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn gprogram_parse_crate() {
        // arrange
        let input = quote!(crate = sails_rename,);
        let expected = ProgramArgs {
            handle_signal: None,
            sails_path: Some(
                PathSegment::from(Ident::new("sails_rename", Span::call_site())).into(),
            ),
            payable: false,
            default_sails_path: syn::parse_str(sails_paths::SAILS).unwrap(),
        };

        // act
        let args = syn::parse2::<ProgramArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn program_parse_payable() {
        // arrange
        let input = quote!(payable,);
        let expected = ProgramArgs {
            handle_signal: None,
            sails_path: None,
            payable: true,
            default_sails_path: syn::parse_str(sails_paths::SAILS).unwrap(),
        };

        // act
        let args = syn::parse2::<ProgramArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }
}
