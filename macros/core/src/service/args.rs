use proc_macro_error::abort;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Path, Result as SynResult, Token,
};

#[derive(PartialEq, Debug)]
pub(super) struct ServiceArgs {
    base_types: Vec<Path>,
    events_type: Option<Path>,
}

impl ServiceArgs {
    pub fn base_types(&self) -> &[Path] {
        &self.base_types
    }

    pub fn events_type(&self) -> &Option<Path> {
        &self.events_type
    }
}

impl Parse for ServiceArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let items = input.parse_terminated(ServiceArg::parse, Token![,])?;
        let base_types = items
            .iter()
            .filter_map(|arg| match arg {
                ServiceArg::Extends(paths) => Some(paths.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        let mut events_types = items.iter().filter_map(|arg| match arg {
            ServiceArg::Events(path) => Some(path.clone()),
            _ => None,
        });
        let events_type = events_types.next();
        if let Some(path) = events_types.next() {
            abort!(path, "only one `events` argument is allowed")
        }
        Ok(Self {
            base_types,
            events_type,
        })
    }
}

#[derive(Debug)]
enum ServiceArg {
    Extends(Vec<Path>),
    Events(Path),
}

impl Parse for ServiceArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        match ident.to_string().as_str() {
            "extends" => {
                if let Ok(path) = input.parse::<Path>() {
                    // Check path_expr.attrs is empty and qself is none
                    return Ok(Self::Extends(vec![path]));
                } else if let Ok(paths) = input.parse::<PathVec>() {
                    return Ok(Self::Extends(paths.0));
                }
                abort!(ident, "unexpected value for `extends` argument: {}", input)
            }
            "events" => {
                if let Ok(path) = input.parse::<Path>() {
                    return Ok(Self::Events(path));
                }
                abort!(ident, "unexpected value for `events` argument: {}", input)
            }
            _ => abort!(ident, "unknown argument: {}", ident),
        }
    }
}

struct PathVec(Vec<Path>);

impl Parse for PathVec {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let content;
        let _bracket = bracketed!(content in input);
        let punctuated: Punctuated<Path, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(PathVec(punctuated.into_iter().collect::<Vec<_>>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::quote;
    use syn::{
        punctuated::Punctuated, AngleBracketedGenericArguments, GenericArgument, Lifetime,
        PathArguments, PathSegment, Token,
    };

    #[test]
    fn gservice_parse_empty() {
        // arrange
        let input = quote!();

        let expected = ServiceArgs {
            base_types: vec![],
            events_type: None,
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
            args: args,
            gt_token: Token![>](Span::call_site()),
        };

        let expected = ServiceArgs {
            base_types: vec![PathSegment {
                ident: Ident::new("SomeService", Span::call_site()),
                arguments: PathArguments::AngleBracketed(arguments),
            }
            .into()],
            events_type: None,
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
            args: args,
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
        };

        // act
        let args = syn::parse2::<ServiceArgs>(input).unwrap();

        // arrange
        assert_eq!(expected, args);
    }
}
