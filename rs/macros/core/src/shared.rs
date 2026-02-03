use crate::export;
use convert_case::{Case, Casing};
use proc_macro_error::abort;
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::{
    FnArg, GenericArgument, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, Lifetime, Pat, Path,
    PathArguments, PathSegment, ReturnType, Signature, Token, Type, TypeImplTrait, TypeParamBound,
    TypePath, TypeReference, TypeTuple, WhereClause, punctuated::Punctuated, spanned::Spanned,
};

pub(crate) fn impl_type_refs(item_impl_type: &Type) -> (&TypePath, &PathArguments, &Ident) {
    let path = if let Type::Path(type_path) = item_impl_type {
        type_path
    } else {
        abort!(
            item_impl_type,
            "failed to parse impl type: {}",
            item_impl_type.to_token_stream()
        )
    };
    let segment = path.path.segments.last().unwrap();
    let args = &segment.arguments;
    let ident = &segment.ident;
    (path, args, ident)
}

pub(crate) fn impl_constraints(item_impl: &ItemImpl) -> (Generics, Option<WhereClause>) {
    let mut generics = item_impl.generics.clone();
    let where_clause = generics.where_clause.take();
    (generics, where_clause)
}

fn extract_params(handler_signature: &Signature) -> impl Iterator<Item = (&Ident, &Type)> {
    handler_signature.inputs.iter().filter_map(|arg| {
        if let FnArg::Typed(arg) = arg {
            let arg_ident = if let Pat::Ident(arg_ident) = arg.pat.as_ref() {
                &arg_ident.ident
            } else {
                abort!(arg.span(), "unnamed arguments are not supported");
            };
            return Some((arg_ident, arg.ty.as_ref()));
        }
        None
    })
}

pub(crate) fn result_type(handler_signature: &Signature) -> Type {
    match &handler_signature.output {
        ReturnType::Type(_, ty) => *ty.to_owned(),
        ReturnType::Default => Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Default::default(),
        }),
    }
}

pub(crate) fn unwrap_result_type(handler_signature: &Signature, unwrap_result: bool) -> Type {
    let result_type = result_type(handler_signature);
    // process result type if set unwrap result
    if unwrap_result {
        {
            extract_result_type_from_path(&result_type)
                .unwrap_or_else(|| {
                    abort!(
                        result_type.span(),
                        "`unwrap_result` can be applied to methods returns result only"
                    )
                })
                .clone()
        }
    } else {
        result_type
    }
}

pub(crate) struct InvocationExport {
    pub span: Span,
    pub route: String,
    pub unwrap_result: bool,
    pub export: bool,
    #[cfg(feature = "ethexe")]
    pub payable: bool,
}

pub(crate) fn invocation_export(fn_impl: &ImplItemFn) -> Option<InvocationExport> {
    export::parse_export_args(&fn_impl.attrs).map(|(args, span)| {
        let ident = &fn_impl.sig.ident;
        let unwrap_result = args.unwrap_result();
        #[cfg(feature = "ethexe")]
        let payable = args.payable();

        let route = args.route().map_or_else(
            || ident.to_string().to_case(Case::Pascal),
            |route| route.to_case(Case::Pascal),
        );
        InvocationExport {
            span,
            route,
            unwrap_result,
            export: true,
            #[cfg(feature = "ethexe")]
            payable,
        }
    })
}

pub(crate) fn invocation_export_or_default(fn_impl: &ImplItemFn) -> InvocationExport {
    invocation_export(fn_impl).unwrap_or_else(|| {
        let ident = &fn_impl.sig.ident;
        InvocationExport {
            span: ident.span(),
            route: ident.to_string().to_case(Case::Pascal),
            unwrap_result: false,
            export: false,
            #[cfg(feature = "ethexe")]
            payable: false,
        }
    })
}

pub(crate) fn discover_invocation_targets<'a>(
    item_impl: &'a ItemImpl,
    filter: impl Fn(&ImplItemFn) -> bool,
    sails_path: &'a Path,
) -> Vec<FnBuilder<'a>> {
    let mut routes = BTreeMap::<String, String>::new();
    let vec: Vec<FnBuilder<'a>> = item_impl
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(fn_item) = item
                && filter(fn_item)
            {
                let InvocationExport {
                    span,
                    route,
                    unwrap_result,
                    export,
                    #[cfg(feature = "ethexe")]
                    payable,
                } = invocation_export_or_default(fn_item);
                // `entry_id` in order of appearance
                let entry_id = routes.len() as u16;
                if let Some(duplicate) = routes.insert(route.clone(), fn_item.sig.ident.to_string())
                {
                    abort!(
                        span,
                        "`export` attribute conflicts with one already assigned to '{}'",
                        duplicate
                    );
                }
                let is_result = extract_result_type_from_path(&result_type(&fn_item.sig)).is_some();
                let fn_builder = FnBuilder::new(
                    route,
                    entry_id,
                    export,
                    fn_item,
                    unwrap_result || is_result,
                    sails_path,
                );
                #[cfg(feature = "ethexe")]
                let fn_builder = fn_builder.payable(payable);
                return Some(fn_builder);
            }
            None
        })
        .collect();
    vec
}

pub(crate) fn replace_any_lifetime_with_static(ty: Type) -> Type {
    match ty {
        Type::Reference(r) => {
            if r.lifetime.is_some() {
                Type::Reference(TypeReference {
                    and_token: r.and_token,
                    lifetime: Some(Lifetime::new("'static", Span::call_site())),
                    mutability: r.mutability,
                    elem: r.elem,
                })
            } else {
                Type::Reference(r)
            }
        }
        Type::Path(p) => Type::Path(TypePath {
            path: replace_lifetime_with_static_in_path(p.path),
            qself: p.qself,
        }),
        _ => ty,
    }
}

fn replace_lifetime_with_static_in_path(path: Path) -> Path {
    let mut segments: Punctuated<PathSegment, Token![::]> = Punctuated::new();
    for s in path.segments {
        segments.push(PathSegment {
            ident: s.ident,
            arguments: replace_lifetime_with_static_in_path_args(s.arguments),
        });
    }
    Path {
        leading_colon: path.leading_colon,
        segments,
    }
}

fn replace_lifetime_with_static_in_path_args(path_args: PathArguments) -> PathArguments {
    if let PathArguments::AngleBracketed(mut type_args) = path_args {
        type_args.args.iter_mut().for_each(|a| match a {
            GenericArgument::Lifetime(lifetime) => {
                *lifetime = Lifetime::new("'static", Span::call_site());
            }
            GenericArgument::Type(ty) => *ty = replace_any_lifetime_with_static(ty.clone()),
            _ => {}
        });
        PathArguments::AngleBracketed(type_args)
    } else {
        path_args
    }
}

pub(crate) fn remove_lifetimes(path: &Path) -> Path {
    let mut segments: Punctuated<PathSegment, Token![::]> = Punctuated::new();
    for s in &path.segments {
        segments.push(PathSegment {
            ident: s.ident.clone(),
            arguments: PathArguments::None,
        });
    }
    Path {
        leading_colon: path.leading_colon,
        segments,
    }
}

/// Check if type is `CommandReply<T>` and extract inner type `T`
pub(crate) fn extract_reply_type_with_value(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(tp) => extract_reply_result_type(tp),
        Type::ImplTrait(imp) => extract_reply_result_type_from_impl_into(imp),
        _ => None,
    }
}

/// Extract `T` type from `CommandReply<T>`
fn extract_reply_result_type(tp: &TypePath) -> Option<&Type> {
    if let Some(last) = tp.path.segments.last() {
        if last.ident != "CommandReply" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &last.arguments
            && args.args.len() == 1
            && let Some(GenericArgument::Type(ty)) = args.args.first()
        {
            return Some(ty);
        }
    }
    None
}

/// Extract `T` type from `impl Into<CommandReply<T>>`
fn extract_reply_result_type_from_impl_into(tit: &TypeImplTrait) -> Option<&Type> {
    if let Some(TypeParamBound::Trait(tr)) = tit.bounds.first()
        && let Some(last) = tr.path.segments.last()
    {
        if last.ident != "Into" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &last.arguments
            && args.args.len() == 1
            && let Some(GenericArgument::Type(Type::Path(tp))) = args.args.first()
        {
            return extract_reply_result_type(tp);
        }
    }
    None
}

/// Check if type is `Result<T, E>` and extract inner type `T`
pub(crate) fn extract_result_type_from_path(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => extract_result_types(tp).map(|(ok_ty, _)| ok_ty),
        _ => None,
    }
}

/// Extract both `T` and `E` types from `Result<T, E>`
pub(crate) fn extract_result_types(tp: &TypePath) -> Option<(&Type, &Type)> {
    if let Some(last) = tp.path.segments.last() {
        // TODO: This currently only recognizes the literal name "Result" and exactly 2 generic arguments.
        // It fails to see through aliases like `type MyResult<T> = Result<T, Error>`, leading
        // to interface_id mismatches if idl-gen is smarter than this macro.
        if last.ident != "Result" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &last.arguments
            && args.args.len() == 2
            && let Some(GenericArgument::Type(ok_ty)) = args.args.first()
            && let Some(GenericArgument::Type(err_ty)) = args.args.last()
        {
            return Some((ok_ty, err_ty));
        }
    }
    None
}

/// Represents parts of a handler function.
#[derive(Clone)]
pub(crate) struct FnBuilder<'a> {
    pub route: String,
    pub entry_id: u16,
    pub export: bool,
    pub payable: bool,
    pub impl_fn: &'a ImplItemFn,
    pub ident: &'a Ident,
    pub params_struct_ident: Ident,
    params_idents: Vec<&'a Ident>,
    params_types: Vec<&'a Type>,
    pub result_type: Type,
    pub unwrap_result: bool,
    pub sails_path: &'a Path,
}

impl<'a> FnBuilder<'a> {
    pub(crate) fn new(
        route: String,
        entry_id: u16,
        export: bool,
        impl_fn: &'a ImplItemFn,
        unwrap_result: bool,
        sails_path: &'a Path,
    ) -> Self {
        let signature = &impl_fn.sig;
        let ident = &signature.ident;
        let params_struct_ident = Ident::new(&format!("__{route}Params"), Span::call_site());
        let (params_idents, params_types): (Vec<_>, Vec<_>) = extract_params(signature).unzip();
        let result_type = unwrap_result_type(signature, unwrap_result);

        Self {
            route,
            entry_id,
            export,
            payable: false,
            impl_fn,
            ident,
            params_struct_ident,
            params_idents,
            params_types,
            result_type,
            unwrap_result,
            sails_path,
        }
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn payable(mut self, payable: bool) -> Self {
        self.payable = payable;
        self
    }
    pub(crate) fn is_async(&self) -> bool {
        self.impl_fn.sig.asyncness.is_some()
    }

    pub(crate) fn is_query(&self) -> bool {
        self.impl_fn
            .sig
            .receiver()
            .is_none_or(|r| r.mutability.is_none())
    }

    pub(crate) fn result_type_with_value(&self) -> (&Type, bool) {
        let result_type = &self.result_type;
        let (result_type, reply_with_value) = extract_reply_type_with_value(result_type)
            .map_or_else(|| (result_type, false), |ty| (ty, true));

        if reply_with_value && self.is_query() {
            abort!(
                self.result_type.span(),
                "using `CommandReply` type in a query is not allowed"
            );
        }
        (result_type, reply_with_value)
    }

    pub(crate) fn params(&self) -> impl Iterator<Item = (&&Ident, &&Type)> {
        self.params_idents.iter().zip(self.params_types.iter())
    }

    pub(crate) fn params_idents(&self) -> &[&Ident] {
        self.params_idents.as_slice()
    }

    pub(crate) fn params_types(&self) -> &[&Type] {
        self.params_types.as_slice()
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn route_camel_case(&self) -> String {
        use convert_case::{Boundary, Case, Casing};

        self.route
            .with_boundaries(&[Boundary::UNDERSCORE, Boundary::LOWER_UPPER])
            .to_case(Case::Camel)
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn payable_check(&self) -> proc_macro2::TokenStream {
        if !self.payable {
            let sails_path = self.sails_path;
            let msg = format!("'{}' accepts no value", self.ident);
            quote::quote! {
                #[cfg(target_arch = "wasm32")]
                if #sails_path::gstd::msg::value() > 0 {
                    core::panic!(#msg);
                }
            }
        } else {
            quote::quote!()
        }
    }
}

#[cfg(feature = "ethexe")]
pub mod validation {
    use proc_macro_error::abort;
    use proc_macro2::Span;

    // Source: https://github.com/argotorg/solidity/blob/develop/liblangutil/Token.h
    // docs:
    // https://docs.soliditylang.org/en/latest/types.html
    // https://docs.soliditylang.org/en/latest/units-and-global-variables.html#reserved-keywords
    const SOL_KEYWORDS: &[&str] = &[
        "abi",
        "abstract",
        "addmod",
        "address",
        "after",
        "alias",
        "anonymous",
        "apply",
        "as",
        "assembly",
        "assert",
        "auto",
        "block",
        "blockhash",
        "bool",
        "break",
        "byte",
        "bytes",
        "calldata",
        "case",
        "catch",
        "constant",
        "constructor",
        "continue",
        "contract",
        "copyof",
        "days",
        "default",
        "define",
        "delete",
        "do",
        "ecrecover",
        "else",
        "emit",
        "enum",
        "ether",
        "event",
        "external",
        "false",
        "final",
        "fixed",
        "for",
        "function",
        "gasleft",
        "gwei",
        "hex",
        "hours",
        "if",
        "immutable",
        "implements",
        "import",
        "in",
        "indexed",
        "inline",
        "int",
        "interface",
        "internal",
        "is",
        "keccak256",
        "let",
        "library",
        "macro",
        "mapping",
        "match",
        "memory",
        "minutes",
        "modifier",
        "msg",
        "mulmod",
        "mutable",
        "new",
        "null",
        "of",
        "override",
        "partial",
        "payable",
        "pragma",
        "private",
        "promise",
        "public",
        "pure",
        "reference",
        "relocatable",
        "require",
        "return",
        "returns",
        "revert",
        "ripemd160",
        "sealed",
        "seconds",
        "selfdestruct",
        "sha256",
        "sizeof",
        "static",
        "storage",
        "string",
        "struct",
        "super",
        "supports",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "tx",
        "type",
        "typedef",
        "typeof",
        "ufixed",
        "uint",
        "unchecked",
        "unicode",
        "using",
        "var",
        "view",
        "virtual",
        "weeks",
        "wei",
        "while",
        "years",
    ];

    fn is_reserved(s: &str) -> bool {
        let s = s.to_ascii_lowercase();

        if SOL_KEYWORDS.binary_search(&s.as_str()).is_ok() {
            return true;
        }

        // bytes<N>
        if let Some(num) = s.strip_prefix("bytes").and_then(|x| x.parse::<u8>().ok()) {
            return (1..=32).contains(&num);
        }

        // uint<N> | int<N>
        if let Some(rest) = s.strip_prefix("uint").or_else(|| s.strip_prefix("int"))
            && let Ok(n) = rest.parse::<u16>()
        {
            return n == 8 || (16..=256).contains(&n) && n % 8 == 0;
        }

        // ufixed<M>x<N> | fixed<M>x<N>
        if let Some(rest) = s.strip_prefix("ufixed").or_else(|| s.strip_prefix("fixed"))
            && let Some((m_str, n_str)) = rest.split_once('x')
            && let (Ok(m), Ok(n)) = (m_str.parse::<u16>(), n_str.parse::<u8>())
            && (8..=256).contains(&m)
            && m % 8 == 0
            && n <= 80
        {
            return true;
        }

        false
    }

    pub fn validate_identifier(name: &str, span: Span, type_of_ident: &str) {
        if is_reserved(name) {
            abort!(
                span,
                "The name '{}' cannot be used for a {} because it is a reserved keyword in Solidity.",
                name,
                type_of_ident;
                help = "Please rename this item to avoid compilation errors in the generated Solidity contract."
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn is_reserved_works() {
            // Keywords
            assert!(is_reserved("address"));
            assert!(is_reserved("contract"));
            assert!(is_reserved("function"));
            assert!(!is_reserved("myfunction"));

            // bytes<N>
            assert!(is_reserved("bytes1"));
            assert!(is_reserved("bytes16"));
            assert!(is_reserved("bytes32"));
            assert!(!is_reserved("bytes0"));
            assert!(!is_reserved("bytes33"));
            assert!(!is_reserved("sbytes1"));

            // uint<N> | int<N>
            assert!(is_reserved("uint8"));
            assert!(is_reserved("int8"));
            assert!(is_reserved("uint16"));
            assert!(is_reserved("int24"));
            assert!(is_reserved("uint256"));
            assert!(!is_reserved("uint9"));
            assert!(!is_reserved("int17"));
            assert!(!is_reserved("uint257"));
            assert!(!is_reserved("uint249"));
            assert!(!is_reserved("uint"));
            assert!(!is_reserved("int"));

            // ufixed<M>x<N> | fixed<M>x<N>
            assert!(is_reserved("fixed128x18"));
            assert!(is_reserved("ufixed256x80"));
            assert!(is_reserved("fixed8x1"));
            assert!(is_reserved("ufixed256x0"));
            assert!(!is_reserved("fixed"));
            assert!(!is_reserved("ufixed"));
            assert!(!is_reserved("fixed128x81")); // N > 80
            assert!(!is_reserved("fixed264x18")); // M > 256
            assert!(!is_reserved("fixed129x18")); // M not divisible by 8
            assert!(!is_reserved("fixed4x1")); // M < 8
            assert!(!is_reserved("fixed128"));
            assert!(!is_reserved("fixed128xN"));
        }
    }
}
