use proc_macro_error::abort;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, Path, Result as SynResult, Token,
};

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
        let values = input.parse::<Expr>()?;
        match ident.to_string().as_str() {
            "extends" => {
                if let Expr::Path(path_expr) = values {
                    // Check path_expr.attrs is empty and qself is none
                    return Ok(Self::Extends(vec![path_expr.path]));
                } else if let Expr::Array(array_expr) = values {
                    let mut paths = Vec::new();
                    for item_expr in array_expr.elems {
                        if let Expr::Path(path_expr) = item_expr {
                            paths.push(path_expr.path);
                        } else {
                            abort!(
                                item_expr,
                                "unexpected value for `extends` argument: {}",
                                item_expr.to_token_stream()
                            )
                        }
                    }
                    return Ok(Self::Extends(paths));
                }
                abort!(
                    ident,
                    "unexpected value for `extends` argument: {}",
                    values.to_token_stream()
                )
            }
            "events" => {
                if let Expr::Path(path_expr) = values {
                    return Ok(Self::Events(path_expr.path));
                }
                abort!(
                    ident,
                    "unexpected value for `events` argument: {}",
                    values.to_token_stream()
                )
            }
            _ => abort!(ident, "unknown argument: {}", ident),
        }
    }
}
