use crate::sails_paths;
use proc_macro_error::abort;
use syn::{
    Path, Result as SynResult, Token, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

#[derive(PartialEq, Debug)]
pub(super) struct ServiceArgs {
    base_types: Vec<Path>,
    events_type: Option<Path>,
    sails_path: Option<Path>,
}

impl ServiceArgs {
    pub fn base_types(&self) -> &[Path] {
        self.base_types.as_slice()
    }

    pub fn events_type(&self) -> Option<&Path> {
        self.events_type.as_ref()
    }

    pub fn sails_path(&self) -> syn::Path {
        sails_paths::sails_path_or_default(self.sails_path.clone())
    }
}

impl Parse for ServiceArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let items = input.parse_terminated(ServiceArg::parse, Token![,])?;
        let mut base_types: Vec<Path> = items
            .iter()
            .filter_map(|arg| match arg {
                ServiceArg::Extends(paths) => Some(paths.clone()),
                _ => None,
            })
            .flatten()
            .collect();

        base_types.sort_by_cached_key(|path| {
            path.segments
                .last()
                .expect("path has at least one segment")
                .ident
                .to_string()
                .to_lowercase()
        });

        let mut events_types = items.iter().filter_map(|arg| match arg {
            ServiceArg::Events(path) => Some(path.clone()),
            _ => None,
        });
        let events_type = events_types.next();
        if let Some(path) = events_types.next() {
            abort!(path, "only one `events` argument is allowed")
        }
        let sails_path = items.iter().find_map(|arg| match arg {
            ServiceArg::SailsPath(path) => Some(path.clone()),
            _ => None,
        });
        Ok(Self {
            base_types,
            events_type,
            sails_path,
        })
    }
}

#[derive(Debug)]
enum ServiceArg {
    Extends(Vec<Path>),
    Events(Path),
    SailsPath(Path),
}

impl Parse for ServiceArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let path = input.parse::<Path>()?;
        let ident = path.get_ident().unwrap();
        input.parse::<Token![=]>()?;
        match ident.to_string().as_str() {
            "extends" => {
                if let Ok(path) = input.parse::<Path>() {
                    // Check path_expr.attrs is empty and qself is none
                    return Ok(Self::Extends(vec![path]));
                } else if input.peek(token::Bracket) {
                    let content;
                    let _bracket = bracketed!(content in input);
                    let punctuated: Punctuated<Path, Token![,]> =
                        Punctuated::parse_terminated(&content)?;
                    return Ok(Self::Extends(punctuated.into_iter().collect::<Vec<_>>()));
                }
                abort!(ident, "unexpected value for `extends` argument: {}", input)
            }
            "events" => {
                if let Ok(path) = input.parse::<Path>() {
                    return Ok(Self::Events(path));
                }
                abort!(ident, "unexpected value for `events` argument: {}", input)
            }
            "crate" => {
                if let Ok(path) = input.parse::<Path>() {
                    return Ok(Self::SailsPath(path));
                }
                abort!(ident, "unexpected value for `crate` argument: {}", input)
            }
            _ => abort!(ident, "unknown argument: {}", ident),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::quote;
    use syn::{
        AngleBracketedGenericArguments, GenericArgument, Ident, Lifetime, PathArguments,
        PathSegment, Token, punctuated::Punctuated,
    };

    #[test]
    fn gservice_parse_empty() {
        // arrange
        let input = quote!();

        let expected = ServiceArgs {
            base_types: vec![],
            events_type: None,
            sails_path: None,
        };

        // act
        let args = syn::parse2::<ServiceArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn gservice_parse_extends_path() {
        // arrange
        let input = quote!(extends = SomeService);

        let expected = ServiceArgs {
            base_types: vec![
                PathSegment::from(Ident::new("SomeService", Span::call_site())).into(),
            ],
            events_type: None,
            sails_path: None,
        };

        // act
        let args = syn::parse2::<ServiceArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn gservice_parse_extends_array_path() {
        // arrange
        let input = quote!(extends = [SomeService, AnotherService]);

        let expected = ServiceArgs {
            base_types: vec![
                PathSegment::from(Ident::new("SomeService", Span::call_site())).into(),
                PathSegment::from(Ident::new("AnotherService", Span::call_site())).into(),
            ],
            events_type: None,
            sails_path: None,
        };

        // act
        let args = syn::parse2::<ServiceArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn gservice_parse_extends_path_with_args() {
        // arrange
        let input = quote!(extends = SomeService<'a>);

        let lt = Lifetime::new("'a", Span::call_site());
        let mut args = Punctuated::new();
        args.push(GenericArgument::Lifetime(lt));
        let arguments = AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Token![<](Span::call_site()),
            args,
            gt_token: Token![>](Span::call_site()),
        };

        let expected = ServiceArgs {
            base_types: vec![
                PathSegment {
                    ident: Ident::new("SomeService", Span::call_site()),
                    arguments: PathArguments::AngleBracketed(arguments),
                }
                .into(),
            ],
            events_type: None,
            sails_path: None,
        };

        // act
        let args = syn::parse2::<ServiceArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }

    #[test]
    fn gservice_parse_extends_array_path_with_args() {
        // arrange
        let input = quote!(extends = [BaseService, SomeService<'a>]);

        let lt = Lifetime::new("'a", Span::call_site());
        let mut args = Punctuated::new();
        args.push(GenericArgument::Lifetime(lt));
        let arguments = AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Token![<](Span::call_site()),
            args,
            gt_token: Token![>](Span::call_site()),
        };

        let expected = ServiceArgs {
            base_types: vec![
                PathSegment::from(Ident::new("BaseService", Span::call_site())).into(),
                PathSegment {
                    ident: Ident::new("SomeService", Span::call_site()),
                    arguments: PathArguments::AngleBracketed(arguments),
                }
                .into(),
            ],
            events_type: None,
            sails_path: None,
        };

        // act
        let args = syn::parse2::<ServiceArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }
}
