use crate::sails_paths;
use proc_macro_error::abort;
use syn::{
    LitBool, Path, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(Debug, PartialEq)]
pub(super) struct ProgramArgs {
    handle_reply: Option<Path>,
    handle_signal: Option<Path>,
    sails_path: Option<Path>,
    accept_transfers: bool,
    default_sails_path: Path,
}

impl ProgramArgs {
    pub fn handle_reply(&self) -> Option<&Path> {
        self.handle_reply.as_ref()
    }

    pub fn handle_signal(&self) -> Option<&Path> {
        self.handle_signal.as_ref()
    }

    pub fn sails_path(&self) -> &syn::Path {
        self.sails_path.as_ref().unwrap_or(&self.default_sails_path)
    }

    pub fn accept_transfers(&self) -> bool {
        self.accept_transfers
    }
}

impl Parse for ProgramArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let punctuated: Punctuated<ProgramArg, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut attrs = ProgramArgs {
            handle_reply: None,
            handle_signal: None,
            sails_path: None,
            accept_transfers: false,
            default_sails_path: syn::parse_str(sails_paths::SAILS).unwrap(),
        };

        for arg in punctuated {
            match arg {
                ProgramArg::HandleReply(path) => {
                    attrs.handle_reply = Some(path);
                }
                ProgramArg::HandleSignal(path) => {
                    attrs.handle_signal = Some(path);
                }
                ProgramArg::SailsPath(path) => {
                    attrs.sails_path = Some(path);
                }
                ProgramArg::AcceptTransfer(val) => {
                    attrs.accept_transfers = val;
                }
            }
        }

        Ok(attrs)
    }
}

enum ProgramArg {
    HandleReply(Path),
    HandleSignal(Path),
    SailsPath(Path),
    AcceptTransfer(bool),
}

impl Parse for ProgramArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path: Path = input.parse()?;
        let ident = path.get_ident().unwrap();
        match ident.to_string().as_str() {
            "handle_reply" => {
                input.parse::<Token![=]>()?;
                let path: Path = input.parse()?;
                Ok(Self::HandleReply(path))
            }
            "handle_signal" => {
                input.parse::<Token![=]>()?;
                let path: Path = input.parse()?;
                Ok(Self::HandleSignal(path))
            }
            "crate" => {
                input.parse::<Token![=]>()?;
                let path: Path = input.parse()?;
                Ok(Self::SailsPath(path))
            }
            "accept_transfers" => {
                if input.parse::<Token![=]>().is_ok() {
                    if let Ok(val) = input.parse::<LitBool>() {
                        return Ok(Self::AcceptTransfer(val.value()));
                    }
                }
                Ok(Self::AcceptTransfer(true))
            }
            _ => abort!(
                ident,
                "`program` attribute can only contain `handle_reply`, `handle_signal`, `crate`, `accept_transfers` parameters",
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
        let input = quote!(
            handle_reply = my_handle_reply,
            handle_signal = my_handle_signal,
        );
        let expected = ProgramArgs {
            handle_reply: Some(
                PathSegment::from(Ident::new("my_handle_reply", Span::call_site())).into(),
            ),
            handle_signal: Some(
                PathSegment::from(Ident::new("my_handle_signal", Span::call_site())).into(),
            ),
            sails_path: None,
            accept_transfers: false,
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
            handle_reply: None,
            handle_signal: None,
            sails_path: Some(
                PathSegment::from(Ident::new("sails_rename", Span::call_site())).into(),
            ),
            accept_transfers: false,
            default_sails_path: syn::parse_str(sails_paths::SAILS).unwrap(),
        };

        // act
        let args = syn::parse2::<ProgramArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn program_parse_accept_transfers() {
        // arrange
        let input = quote!(accept_transfers,);
        let expected = ProgramArgs {
            handle_reply: None,
            handle_signal: None,
            sails_path: None,
            accept_transfers: true,
            default_sails_path: syn::parse_str(sails_paths::SAILS).unwrap(),
        };

        // act
        let args = syn::parse2::<ProgramArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }
}
