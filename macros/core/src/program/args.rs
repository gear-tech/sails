use std::collections::BTreeSet;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Path, Token,
};

#[derive(Debug, Default, PartialEq)]
pub(super) struct ProgramArgs {
    handle_reply: Option<Path>,
    handle_signal: Option<Path>,
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
        };
        let mut existing_attrs = BTreeSet::new();

        for ProgramArg { name, path, .. } in punctuated {
            let name = name.to_string();
            if existing_attrs.contains(&name) {
                return Err(syn::Error::new_spanned(name, "parameter already defined"));
            }

            match &*name {
                "handle_reply" => {
                    attrs.handle_reply = Some(path);
                }
                "handle_signal" => {
                    attrs.handle_signal = Some(path);
                }
                _ => return Err(syn::Error::new_spanned(name, "unknown parameter")),
            }

            existing_attrs.insert(name);
        }

        Ok(attrs)
    }
}

struct ProgramArg {
    name: Ident,
    path: Path,
}

impl Parse for ProgramArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let path: Path = input.parse()?;

        Ok(Self { name, path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::quote;
    use syn::PathSegment;

    #[test]
    fn gprogram_parse_attrs() {
        // arrange
        let input = quote!(
            handle_reply = my_handle_reply,
            handle_signal = my_handle_signal
        );
        let expected = ProgramArgs {
            handle_reply: Some(
                PathSegment::from(Ident::new("my_handle_reply", Span::call_site())).into(),
            ),
            handle_signal: Some(
                PathSegment::from(Ident::new("my_handle_signal", Span::call_site())).into(),
            ),
        };

        // act
        let args = syn::parse2::<ProgramArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn gprogram_parse_attrs_unknown_parameter() {
        // arrange
        let input = quote!(
            _handle_reply = my_handle_reply,
            handle_signal = my_handle_signal
        );

        // act
        let result = syn::parse2::<ProgramArgs>(input);

        // arrange
        assert!(result.is_err());
    }
}
