use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, Ident, Type, parse_macro_input};

#[proc_macro_derive(TypeInfo, attributes(type_info))]
pub fn type_info_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match process_derive(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn process_derive(mut input: DeriveInput) -> syn::Result<TokenStream2> {
    let registry = resolve_registry_path(&input);
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

    let type_params_count = input.generics.type_params().count();
    let type_params_registration = input.generics.params.iter().filter_map(|p| match p {
        GenericParam::Type(tp) => {
            let ty = &tp.ident;
            let name = ty.to_string();
            Some(quote! {
                let arg_id = registry.register_type::<#ty>();
                args.push((#name, arg_id));
            })
        }
        _ => None,
    });

    let type_params_builder = input.generics.params.iter().filter_map(|p| match p {
        GenericParam::Type(_) => Some(quote! {
            if let Some((name, arg_id)) = args.next() {
                type_builder = type_builder.param(name).arg(arg_id);
            }
        }),
        GenericParam::Const(cp) => {
            let ident = &cp.ident;
            let name = ident.to_string();
            Some(quote! {
                type_builder = type_builder.param(#name).val(#registry::prelude::alloc::format!("{}", #ident));
            })
        }
        _ => None,
    });

    let type_params_prelude = if type_params_count > 0 {
        quote! {
            let mut args: #registry::prelude::alloc::vec::Vec<(&'static str, _)> = #registry::prelude::alloc::vec![];
            #(#type_params_registration)*
            let mut args = args.into_iter();
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #registry::TypeInfo for #name #ty_generics #where_clause {
            type Identity = Self;
            fn type_info(registry: &mut #registry::Registry) -> #registry::ty::Type {
                #type_params_prelude

                let mut type_builder = #registry::builder::TypeBuilder::new()
                    .module_path(::core::module_path!())
                    .name(#name_str)
                    #(.doc(#docs))*
                    #(#annotations)*;

                #(#type_params_builder)*

                #def_tokens
            }
        }
    })
}

fn resolve_registry_path(input: &DeriveInput) -> TokenStream2 {
    let mut path = None;
    for attr in &input.attrs {
        if attr.path().is_ident("type_info") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    path = Some(meta.value()?.parse::<syn::Path>()?);
                }
                Ok(())
            });
            if let p @ Some(_) = path {
                return quote!(#p);
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
            return if crate_name == "sails-rs" {
                quote!(#ident::type_info)
            } else {
                ident
            };
        }
    }
    quote!(::sails_type_registry)
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
                let doc = s.value();
                let clean = doc.strip_prefix(' ').unwrap_or(&doc);
                return Some(quote!(#clean));
            }
            None
        })
        .collect()
}

fn extract_annotations(attrs: &[syn::Attribute]) -> syn::Result<Vec<TokenStream2>> {
    let mut anns = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("type_info")) {
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| meta.error("expected identifier"))?;
            let ident_str = ident.to_string();
            if ident_str == "crate" {
                let _: syn::Expr = meta.value()?.parse()?;
                return Ok(());
            }

            if meta.input.peek(syn::Token![=]) {
                let value: syn::Expr = meta.value()?.parse()?;
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = value
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
            return self.register_type(ty);
        }

        match ty {
            Type::Path(tp) => self.path_type_tokens(ty, tp),
            Type::Reference(tr) => self.reference_type_tokens(tr),
            Type::Array(ta) => {
                let inner = self.type_tokens(&ta.elem);
                let len = &ta.len;
                let registry = self.registry;
                quote! {
                    { let inner_id = #inner; registry.register_type_def(#registry::ty::Type::builder().array(inner_id, #len as u32)) }
                }
            }
            Type::Tuple(tt) => {
                let ids = tt.elems.iter().map(|ty| self.type_tokens(ty));
                let registry = self.registry;
                quote! {
                    { let ids = #registry::prelude::alloc::vec![#(#ids),*]; registry.register_type_def(#registry::ty::Type::builder().tuple(ids)) }
                }
            }
            _ => self.register_type(ty),
        }
    }

    fn register_type(&self, ty: &Type) -> TokenStream2 {
        quote! { { registry.register_type::<#ty>() } }
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
                        .any(|ty| self.contains_generic_param(ty))
            }
            Type::Reference(tr) => self.contains_generic_param(&tr.elem),
            Type::Array(ta) => {
                self.contains_generic_param(&ta.elem)
                    || matches!(
                        &ta.len,
                        syn::Expr::Path(ep)
                            if self.matches_param(ep.path.get_ident(), self.consts)
                    )
            }
            Type::Tuple(tt) => tt.elems.iter().any(|ty| self.contains_generic_param(ty)),
            _ => false,
        }
    }

    fn path_type_tokens(&self, ty: &Type, tp: &syn::TypePath) -> TokenStream2 {
        if let Some(inner) = self.transparent_wrapper_inner_type(tp) {
            return self.type_tokens(inner);
        }

        if let Some(ident) = tp
            .path
            .get_ident()
            .filter(|ident| self.matches_param(Some(ident), self.params))
        {
            let name = ident.to_string();
            let registry = self.registry;
            return quote! {
                registry.register_type_def(#registry::ty::Type::builder().name(#name).parameter(#name))
            };
        }

        let args = self
            .angle_bracketed_types(tp.path.segments.last())
            .filter(|ty| !self.is_const_param_type(ty))
            .map(|ty| self.type_tokens(ty))
            .collect::<Vec<_>>();

        if args.is_empty() {
            self.register_type(ty)
        } else {
            let registry = self.registry;
            quote! {
                {
                    let base_id = registry.register_type::<#ty>();
                    let args = #registry::prelude::alloc::vec![#(#args),*];
                    registry.register_type_def(#registry::ty::Type::builder().applied(base_id, args))
                }
            }
        }
    }

    fn reference_type_tokens(&self, tr: &syn::TypeReference) -> TokenStream2 {
        if let Type::Slice(slice) = &*tr.elem {
            let inner = self.type_tokens(&slice.elem);
            let registry = self.registry;
            quote! {
                { let inner_id = #inner; registry.register_type_def(#registry::ty::Type::builder().sequence(inner_id)) }
            }
        } else {
            self.type_tokens(&tr.elem)
        }
    }

    fn transparent_wrapper_inner_type<'b>(&self, tp: &'b syn::TypePath) -> Option<&'b Type> {
        let last = tp.path.segments.last()?;
        let type_args = self.angle_bracketed_types(Some(last)).collect::<Vec<_>>();

        match last.ident.to_string().as_str() {
            "Cow" => type_args.last().copied(),
            "Box" | "Rc" | "Arc" if type_args.len() == 1 => type_args.first().copied(),
            _ => None,
        }
    }

    fn angle_bracketed_types<'b>(
        &self,
        segment: Option<&'b syn::PathSegment>,
    ) -> impl Iterator<Item = &'b Type> {
        segment
            .and_then(|segment| match &segment.arguments {
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

    fn is_const_param_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Path(tp) if self.matches_param(tp.path.get_ident(), self.consts))
    }

    fn matches_param(&self, ident: Option<&Ident>, names: &[String]) -> bool {
        ident.is_some_and(|ident| names.iter().any(|name| name == &ident.to_string()))
    }
}

fn generate_fields(fields: &Fields, ctx: &TypeTransformContext<'_>) -> syn::Result<TokenStream2> {
    fields
        .iter()
        .map(|f| {
            let docs = extract_docs(&f.attrs);
            let anns = extract_annotations(&f.attrs)?;
            let (method, args) = f.ident.as_ref().map_or_else(
                || (quote!(unnamed), quote!()),
                |i| {
                    let name = i.to_string();
                    (quote!(field), quote!(#name))
                },
            );
            let ty_tokens = ctx.type_tokens(&f.ty);

            Ok(quote! {
                {
                    let ty = #ty_tokens;
                    builder = builder.#method(#args) #(.doc(#docs))* #(#anns)* .ty(ty);
                }
            })
        })
        .collect::<syn::Result<TokenStream2>>()
}

fn generate_def_tokens(
    data: &Data,
    name: &Ident,
    ctx: &TypeTransformContext<'_>,
) -> syn::Result<TokenStream2> {
    match data {
        Data::Struct(s) => {
            let fields = generate_fields(&s.fields, ctx)?;
            Ok(quote! {
                let mut builder = type_builder.composite();
                #fields
                builder.build()
            })
        }
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|v| {
                let vname = v.ident.to_string();
                let vdocs = extract_docs(&v.attrs);
                let vanns = extract_annotations(&v.attrs)?;
                let fields = generate_fields(&v.fields, ctx)?;
                Ok(quote! {
                    {
                        let mut builder = builder.add_variant(#vname) #(.doc(#vdocs))* #(#vanns)*;
                        #fields
                        builder.finish_variant()
                    }
                })
            }).collect::<syn::Result<Vec<TokenStream2>>>()?;

            Ok(quote! {
                let mut builder = type_builder.variant();
                #(builder = #variants;)*
                builder.build()
            })
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
                    #[type_info(name = "custom")]
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
                #[type_info(top = "val")]
                enum Complex {
                    /// Variant with named fields and annotations
                    #[type_info(v1)]
                    V1 {
                        #[type_info(f1 = "v")]
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
                #[type_info(attr1 = "val1", attr2)]
                pub struct Container<T, U, const SIZE: usize>
                where T: Clone
                {
                    /// Recursive field
                    pub next: Option<Box<Container<T, U, SIZE>>>,
                    /// Field with many annotations
                    #[type_info(indexed, secret = "true", range = "0..100")]
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
