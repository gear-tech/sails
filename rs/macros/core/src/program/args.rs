use proc_macro2::Span;
use proc_macro_error::abort;
use std::collections::BTreeSet;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Path, Token,
};

use crate::sails_paths::SailsPath;

#[derive(Debug, Default, PartialEq)]
pub(super) struct ProgramArgs {
    handle_reply: Option<Path>,
    handle_signal: Option<Path>,
    sails_path: Option<Path>,
}

impl ProgramArgs {
    pub fn handle_reply(&self) -> &Option<Path> {
        &self.handle_reply
    }

    pub fn handle_signal(&self) -> &Option<Path> {
        &self.handle_signal
    }
}

impl Parse for ProgramArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let punctuated: Punctuated<ProgramArg, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut attrs = ProgramArgs {
            handle_reply: None,
            handle_signal: None,
            sails_path: None,
        };
        let mut existing_attrs = BTreeSet::new();

        for ProgramArg {
            name, path, span, ..
        } in punctuated
        {
            let name = name.get_ident().unwrap().to_string();
            if existing_attrs.contains(&name) {
                abort!(span, "parameter already defined");
            }

            match &*name {
                "handle_reply" => {
                    attrs.handle_reply = Some(path);
                }
                "handle_signal" => {
                    attrs.handle_signal = Some(path);
                }
                "crate" => {
                    attrs.sails_path = Some(path);
                }
                _ => abort!(
                    span,
                    "`program` attribute can only contain `handle_reply`, `handle_signal` and `crate` parameters",
                ),
            }

            existing_attrs.insert(name);
        }

        Ok(attrs)
    }
}

impl SailsPath for ProgramArgs {
    fn sails_custom_path(&self) -> Option<syn::Path> {
        self.sails_path.clone()
    }
}

struct ProgramArg {
    name: Path,
    path: Path,
    span: Span,
}

impl Parse for ProgramArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let span = input.span();
        let name: Path = input.parse()?;
        let _: Token![=] = input.parse()?;
        let path: Path = input.parse()?;

        Ok(Self { name, path, span })
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
        };

        // act
        let args = syn::parse2::<ProgramArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }
}
