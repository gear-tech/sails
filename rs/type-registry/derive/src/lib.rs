use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, Ident, Type, parse_macro_input};

#[proc_macro_derive(TypeInfo, attributes(type_info, annotate))]
pub fn type_info_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match process_derive(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn register_fallback(registry: &TokenStream2, ty: &Type) -> TokenStream2 {
    quote! {
        {
            let id = registry.register_type::<#ty>();
            registry.get_type_decl(id).cloned().unwrap_or(#registry::TypeDecl::named("<unknown>".into()))
        }
    }
}

fn process_derive(mut input: DeriveInput) -> syn::Result<TokenStream2> {
    let registry = resolve_registry_path(&input)?;
    let name = &input.ident;
    let name_str = name.to_string();

    let docs = extract_docs(&input.attrs);
    let annotations = extract_annotations(&input.attrs)?;

    for param in &mut input.generics.params {
        if let GenericParam::Type(tp) = param {
            tp.bounds.push(syn::parse_quote!(#registry::TypeInfo));
        }
    }

    let type_param_names: Vec<_> = input
        .generics
        .type_params()
        .map(|p| p.ident.to_string())
        .collect();
    let const_param_names: Vec<_> = input
        .generics
        .const_params()
        .map(|p| p.ident.to_string())
        .collect();

    let ctx = TypeTransformContext::new(&registry, &type_param_names, &const_param_names);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let def_tokens = generate_def_tokens(&input.data, name, &ctx)?;

    // Handle type declarations
    let type_decl_args = input
        .generics
        .type_params()
        .map(|tp| {
            let ty = &tp.ident;
            let ty_path: syn::Type = syn::parse_quote!(#ty);
            register_fallback(&registry, &ty_path)
        })
        .chain(input.generics.const_params().map(|cp| {
            let ident = &cp.ident;
            let name = ident.to_string();
            quote! {
                #registry::TypeDecl::Named {
                    name: #name.into(),
                    generics: #registry::prelude::alloc::vec![],
                    param: Some(#registry::NamedParam::Const {
                        value: #registry::prelude::alloc::format!("{}", #ident),
                    }),
                }
            }
        }));

    // Handle type builders
    let type_params_builder = input.generics.params.iter().filter_map(|p| match p {
        GenericParam::Type(tp) => {
            let name = tp.ident.to_string();
            Some(quote! { type_builder = type_builder.parameter(#name); })
        }
        GenericParam::Const(cp) => {
            let ident = &cp.ident;
            let name = ident.to_string();
            Some(quote! { type_builder = type_builder.const_parameter(#name, #registry::prelude::alloc::format!("{}", #ident)); })
        }
        _ => None,
    });

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #registry::TypeInfo for #name #ty_generics #where_clause {
            type Identity = Self;

            fn type_decl(registry: &mut #registry::Registry) -> #registry::TypeDecl {
                let generics = #registry::prelude::alloc::vec![#(#type_decl_args),*];
                #registry::TypeDecl::Named {
                    name: #name_str.into(),
                    generics,
                    param: None,
                }
            }

            fn type_def(registry: &mut #registry::Registry) -> core::option::Option<#registry::Type> {
                let mut type_builder = #registry::builder::TypeBuilder::new()
                    .module_path(::core::module_path!())
                    .name(#name_str)
                    #(.doc(#docs))*
                    #(#annotations)*;

                #(#type_params_builder)*

                core::option::Option::Some(#def_tokens)
            }
        }
    })
}

fn resolve_registry_path(input: &DeriveInput) -> syn::Result<TokenStream2> {
    for attr in &input.attrs {
        if attr.path().is_ident("type_info") {
            let mut path = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    path = Some(meta.value()?.parse::<syn::Path>()?);
                }
                Ok(())
            })?;
            if let Some(p) = path {
                return Ok(quote!(#p));
            }
        }
    }

    for crate_name in ["sails-type-registry", "sails-rs"] {
        if let Ok(found) = proc_macro_crate::crate_name(crate_name) {
            let ident = match found {
                proc_macro_crate::FoundCrate::Itself => quote!(crate),
                proc_macro_crate::FoundCrate::Name(n) => {
                    let i = Ident::new(&n, Span::call_site());
                    quote!(::#i)
                }
            };
            return Ok(if crate_name == "sails-rs" {
                quote!(#ident::type_info)
            } else {
                ident
            });
        }
    }
    Ok(quote!(::sails_type_registry))
}

fn extract_docs(attrs: &[syn::Attribute]) -> Vec<TokenStream2> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc")
                && let syn::Meta::NameValue(meta) = &attr.meta
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &meta.value
            {
                let clean = s.value();
                let clean = clean.strip_prefix(' ').unwrap_or(&clean);
                return Some(quote!(#clean));
            }
            None
        })
        .collect()
}

fn extract_annotations(attrs: &[syn::Attribute]) -> syn::Result<Vec<TokenStream2>> {
    let mut anns = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("annotate")) {
        attr.parse_nested_meta(|meta| {
            let ident_str = meta
                .path
                .get_ident()
                .ok_or_else(|| meta.error("expected identifier"))?
                .to_string();
            if meta.input.peek(syn::Token![=]) {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = meta.value()?.parse()?
                {
                    let lit_val = lit_str.value();
                    anns.push(quote! { .annotate(#ident_str).value(#lit_val) });
                    return Ok(());
                }
                return Err(meta.error("expected string literal"));
            }
            anns.push(quote! { .annotate(#ident_str) });
            Ok(())
        })?;
    }
    Ok(anns)
}

struct TypeTransformContext<'a> {
    registry: &'a TokenStream2,
    params: &'a [String],
    consts: &'a [String],
}

impl<'a> TypeTransformContext<'a> {
    fn new(registry: &'a TokenStream2, params: &'a [String], consts: &'a [String]) -> Self {
        Self {
            registry,
            params,
            consts,
        }
    }

    fn type_tokens(&self, ty: &Type) -> TokenStream2 {
        if !self.contains_generic_param(ty) {
            return register_fallback(self.registry, ty);
        }

        match ty {
            Type::Path(tp) => self.path_type_tokens(ty, tp),
            Type::Reference(tr) => {
                if let Type::Slice(slice) = &*tr.elem {
                    let inner = self.type_tokens(&slice.elem);
                    let reg = self.registry;
                    quote! { #reg::TypeDecl::Slice { item: #reg::prelude::alloc::boxed::Box::new(#inner) } }
                } else {
                    self.type_tokens(&tr.elem)
                }
            }
            Type::Array(ta) => {
                let inner = self.type_tokens(&ta.elem);
                let len = &ta.len;
                let reg = self.registry;
                quote! { #reg::TypeDecl::Array { item: #reg::prelude::alloc::boxed::Box::new(#inner), len: #len as u32 } }
            }
            Type::Tuple(tt) => {
                let elems = tt.elems.iter().map(|t| self.type_tokens(t));
                let reg = self.registry;
                quote! { #reg::TypeDecl::tuple(#reg::prelude::alloc::vec![#(#elems),*]) }
            }
            _ => register_fallback(self.registry, ty),
        }
    }

    fn path_type_tokens(&self, ty: &Type, tp: &syn::TypePath) -> TokenStream2 {
        let reg = self.registry;

        if let Some(inner) = self.transparent_wrapper_inner(tp) {
            return self.type_tokens(inner);
        }

        if let Some(ident) = tp
            .path
            .get_ident()
            .filter(|i| self.matches_param(Some(i), self.params))
        {
            let name = ident.to_string();
            return quote! {
                #reg::TypeDecl::Named {
                    name: #name.into(),
                    generics: #reg::prelude::alloc::vec![],
                    param: Some(#reg::NamedParam::Type),
                }
            };
        }

        let Some(seg) = tp.path.segments.last() else {
            return register_fallback(self.registry, ty);
        };
        let short_name = seg.ident.to_string();
        let args: Vec<_> = self.angle_bracketed_types(Some(seg)).collect();

        // Map standard containers directly to AST structures
        match (short_name.as_str(), args.as_slice()) {
            ("Vec" | "VecDeque" | "BTreeSet" | "BinaryHeap", [inner_ty]) => {
                let inner = self.type_tokens(inner_ty);
                return quote! { #reg::TypeDecl::Slice { item: #reg::prelude::alloc::boxed::Box::new(#inner) } };
            }
            ("Option", [inner_ty]) => {
                let inner = self.type_tokens(inner_ty);
                return quote! { #reg::TypeDecl::option(#inner) };
            }
            ("Result", [ok_ty, err_ty]) => {
                let ok = self.type_tokens(ok_ty);
                let err = self.type_tokens(err_ty);
                return quote! { #reg::TypeDecl::result(#ok, #err) };
            }
            ("BTreeMap", [k_ty, v_ty]) => {
                let k = self.type_tokens(k_ty);
                let v = self.type_tokens(v_ty);
                return quote! {
                    #reg::TypeDecl::Slice {
                        item: #reg::prelude::alloc::boxed::Box::new(#reg::TypeDecl::Tuple { types: #reg::prelude::alloc::vec![#k, #v] })
                    }
                };
            }
            _ => {}
        }

        let mut generics = Vec::new();

        // Process standard generics
        generics.extend(
            args.iter()
                .filter(|ty| !self.is_const_param(ty))
                .map(|ty| self.type_tokens(ty)),
        );

        // Process all const parameters and expressions uniformly
        let mut push_const = |name: TokenStream2, val: TokenStream2| {
            generics.push(quote! {
                #reg::TypeDecl::Named {
                    name: #name.into(),
                    generics: #reg::prelude::alloc::vec![],
                    param: Some(#reg::NamedParam::Const {
                        value: #reg::prelude::alloc::format!("{}", #val),
                    }),
                }
            });
        };

        for ty in args.iter().filter(|ty| self.is_const_param(ty)) {
            if let Type::Path(tp) = ty
                && let Some(ident) = tp.path.get_ident()
            {
                let name = ident.to_string();
                push_const(quote!(#name), quote!(#ident));
            }
        }

        for arg in self.angle_bracketed_args(Some(seg)) {
            if let syn::GenericArgument::Const(expr) = arg {
                if let syn::Expr::Path(ep) = &expr
                    && let Some(ident) = ep.path.get_ident()
                {
                    push_const(quote!(#ident), quote!(#expr));
                    continue;
                }
                push_const(
                    quote!(#reg::prelude::alloc::format!("{}", #expr)),
                    quote!(#expr),
                );
            }
        }

        if generics.is_empty() {
            register_fallback(self.registry, ty)
        } else {
            let type_name = tp
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            quote! {
                {
                    let _ = registry.register_type::<#ty>();
                    #reg::TypeDecl::Named {
                        name: #type_name.into(),
                        generics: #reg::prelude::alloc::vec![#(#generics),*],
                        param: None,
                    }
                }
            }
        }
    }

    fn transparent_wrapper_inner<'b>(&self, tp: &'b syn::TypePath) -> Option<&'b Type> {
        let last = tp.path.segments.last()?;
        let args: Vec<_> = self.angle_bracketed_types(Some(last)).collect();
        match last.ident.to_string().as_str() {
            "Cow" => args.last().copied(),
            "Box" | "Rc" | "Arc" if args.len() == 1 => args.first().copied(),
            _ => None,
        }
    }

    fn contains_generic_param(&self, ty: &Type) -> bool {
        if self.params.is_empty() && self.consts.is_empty() {
            return false;
        }
        match ty {
            Type::Path(tp) => {
                self.matches_param(tp.path.get_ident(), self.params)
                    || self
                        .angle_bracketed_types(tp.path.segments.last())
                        .any(|t| self.contains_generic_param(t))
            }
            Type::Reference(tr) => self.contains_generic_param(&tr.elem),
            Type::Array(ta) => {
                self.contains_generic_param(&ta.elem)
                    || matches!(&ta.len, syn::Expr::Path(ep) if self.matches_param(ep.path.get_ident(), self.consts))
            }
            Type::Tuple(tt) => tt.elems.iter().any(|t| self.contains_generic_param(t)),
            _ => false,
        }
    }

    fn angle_bracketed_types<'b>(
        &self,
        seg: Option<&'b syn::PathSegment>,
    ) -> impl Iterator<Item = &'b Type> {
        seg.and_then(|s| match &s.arguments {
            syn::PathArguments::AngleBracketed(args) => Some(args),
            _ => None,
        })
        .into_iter()
        .flat_map(|args| args.args.iter())
        .filter_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
    }

    fn angle_bracketed_args(&self, seg: Option<&syn::PathSegment>) -> Vec<syn::GenericArgument> {
        seg.and_then(|s| match &s.arguments {
            syn::PathArguments::AngleBracketed(args) => Some(args),
            _ => None,
        })
        .into_iter()
        .flat_map(|args| args.args.iter())
        .cloned()
        .collect()
    }

    fn is_const_param(&self, ty: &Type) -> bool {
        matches!(ty, Type::Path(tp) if self.matches_param(tp.path.get_ident(), self.consts))
    }

    fn matches_param(&self, ident: Option<&Ident>, names: &[String]) -> bool {
        ident.is_some_and(|i| names.contains(&i.to_string()))
    }
}

fn generate_fields(
    fields: &Fields,
    ctx: &TypeTransformContext<'_>,
    builder: &TokenStream2,
) -> syn::Result<TokenStream2> {
    fields.iter().map(|f| {
        let docs = extract_docs(&f.attrs);
        let anns = extract_annotations(&f.attrs)?;
        let (method, args) = f.ident.as_ref().map_or_else(
            || (quote!(unnamed), quote!()),
            |i| { let n = i.to_string(); (quote!(field), quote!(#n)) }
        );
        let ty_tokens = ctx.type_tokens(&f.ty);

        Ok(quote! {
            let #builder = #builder.#method(#args) #(.doc(#docs))* #(#anns)* .ty({ #ty_tokens });
        })
    }).collect()
}

fn generate_def_tokens(
    data: &Data,
    name: &Ident,
    ctx: &TypeTransformContext<'_>,
) -> syn::Result<TokenStream2> {
    match data {
        Data::Struct(s) => {
            let fields = generate_fields(&s.fields, ctx, &quote!(composite_builder))?;
            Ok(
                quote! { { let mut composite_builder = type_builder.composite(); #fields composite_builder.build() } },
            )
        }
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|v| {
                let vname = v.ident.to_string();
                let vdocs = extract_docs(&v.attrs);
                let vanns = extract_annotations(&v.attrs)?;
                let fields = generate_fields(&v.fields, ctx, &quote!(__v_builder))?;
                Ok(quote! {
                    __builder = { let __v_builder = __builder.add_variant(#vname) #(.doc(#vdocs))* #(#vanns)*; #fields __v_builder.finish_variant() };
                })
            }).collect::<syn::Result<Vec<_>>>()?;
            Ok(
                quote! { { let mut __builder = type_builder.variant(); #(#variants)* __builder.build() } },
            )
        }
        Data::Union(_) => Err(syn::Error::new(
            name.span(),
            "Unions are not supported by SailsTypeRegistry",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn assert_expansion(input: DeriveInput, snapshot_name: &str) {
        let expanded = process_derive(input).unwrap();
        let file = syn::parse2::<syn::File>(expanded).unwrap();
        let formatted = prettyplease::unparse(&file);
        insta::with_settings!({
            prepend_module_to_snapshot => false,
            snapshot_path => "../tests/snapshots",
        }, {
            insta::assert_snapshot!(snapshot_name, formatted);
        });
    }

    #[test]
    fn minimal() {
        assert_expansion(
            parse_quote!(
                struct Unit;
            ),
            "unit_struct",
        );
        assert_expansion(
            parse_quote!(
                enum Empty {}
            ),
            "empty_enum",
        );
    }

    #[test]
    fn basic_struct() {
        assert_expansion(
            parse_quote! {
                /// Basic struct with docs
                /// Second line of documentation
                struct Basic<T> {
                    /// Simple field
                    #[annotate(name = "custom")]
                    a: u32,
                    /// Direct parameter
                    b: T,
                }
            },
            "basic_struct",
        );
    }

    #[test]
    fn generics_and_containers() {
        assert_expansion(
            parse_quote! {
                struct Generics<T, const N: usize> {
                    /// Nested generics
                    matrix: Vec<Vec<T>>,
                    /// Array with const param
                    data: [T; N],
                    /// Complex path with generics
                    result: Result<Option<T>, String>,
                }
            },
            "generics_and_containers",
        );
    }

    #[test]
    fn complex_enum() {
        assert_expansion(
            parse_quote! {
                #[annotate(top = "val")]
                enum Complex {
                    /// Variant with named fields and annotations
                    #[annotate(v1)]
                    V1 {
                        #[annotate(f1 = "v")]
                        f: u32
                    },
                    /// Variant with unnamed fields
                    V2(u64, String),
                    /// Unit variant
                    V3,
                }
            },
            "complex_enum",
        );
    }

    #[test]
    fn aliases() {
        #[allow(dead_code)]
        type Inner<T> = (T, bool);
        #[allow(dead_code)]
        type Middle<T> = Vec<Inner<T>>;
        #[allow(dead_code)]
        type Outer<T> = Result<Middle<T>, String>;

        assert_expansion(
            parse_quote! {
                struct Aliases<T> {
                    /// Deeply nested aliases: Result<Vec<(T, bool)>, String>
                    field: Outer<T>,
                    /// Direct use of intermediate alias
                    direct: Middle<T>,
                }
            },
            "aliases",
        );
    }

    #[test]
    fn big_type() {
        assert_expansion(
            parse_quote! {
                /// The Container Type
                #[type_info(crate = sails_rs::type_info)]
                #[annotate(attr1 = "val1", attr2)]
                pub struct Container<T, U, const SIZE: usize>
                where T: Clone
                {
                    /// Recursive field
                    pub next: Option<Box<Container<T, U, SIZE>>>,
                    /// Field with many annotations
                    #[annotate(indexed, secret = "true", range = "0..100")]
                    pub data: [T; SIZE],
                    pub mapped: BTreeMap<String, U>,
                    /// Tuple field
                    pub meta: (u32, bool, String),
                }
            },
            "big_type",
        );
    }
}
