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
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let def_tokens = generate_def_tokens(
        &input.data,
        &registry,
        name,
        &type_param_names,
        &const_param_names,
    )?;
    let type_params_builder = input.generics.params.iter().filter_map(|p| match p {
        GenericParam::Type(tp) => {
            let ty = &tp.ident;
            let name = ty.to_string();
            Some(quote! { type_builder = type_builder.param(#name).arg({ registry.register_type::<#ty>() }); })
        }
        GenericParam::Const(cp) => {
            let ident = &cp.ident;
            let name = ident.to_string();
            Some(quote! { type_builder = type_builder.param(#name).val(#registry::prelude::alloc::format ! ("{}", #ident)); })
        }
        _ => None,
    });

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #registry::TypeInfo for #name #ty_generics #where_clause {
            type Identity = Self;
            fn type_info(registry: &mut #registry::Registry) -> #registry::ty::Type {
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

fn generate_field_type_tokens(
    ty: &Type,
    registry: &TokenStream2,
    params: &[String],
    consts: &[String],
) -> TokenStream2 {
    fn transform(
        ty: &Type,
        registry: &TokenStream2,
        params: &[String],
        consts: &[String],
    ) -> Option<TokenStream2> {
        match ty {
            Type::Path(tp) => {
                if let Some(ident) = tp
                    .path
                    .get_ident()
                    .filter(|i| params.contains(&i.to_string()))
                {
                    let id = ident.to_string();
                    return Some(
                        quote! { registry.register_type_def(#registry::ty::Type::builder().name(#id).parameter(#id)) },
                    );
                }
                if let Some(last) = tp.path.segments.last()
                    && let syn::PathArguments::AngleBracketed(args) = &last.arguments
                {
                    let mut has_gen = false;
                    let mut arg_tokens = Vec::new();
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            if let Some(toks) = transform(inner, registry, params, consts) {
                                has_gen = true;
                                arg_tokens.push(toks);
                            } else {
                                arg_tokens.push(quote! { { registry.register_type::<#inner>() } });
                            }
                        }
                    }
                    if has_gen {
                        return Some(quote! {
                            {
                                let base_id = registry.register_type::<#ty>();
                                let args = #registry::prelude::alloc::vec![#(#arg_tokens),*];
                                registry.register_type_def(#registry::ty::Type::builder().applied(base_id, args))
                            }
                        });
                    }
                }
                None
            }
            Type::Reference(tr) => {
                if let Type::Slice(ts) = &*tr.elem {
                    transform(&ts.elem, registry, params, consts).map(|inner| quote! {
                        { let inner_id = #inner; registry.register_type_def(#registry::ty::Type::builder().sequence(inner_id)) }
                    })
                } else {
                    transform(&tr.elem, registry, params, consts)
                }
            }
            Type::Array(ta) => {
                let is_const = matches!(&ta.len, syn::Expr::Path(ep) if ep.path.get_ident().is_some_and(|i| consts.contains(&i.to_string())));
                if let Some(inner) = transform(&ta.elem, registry, params, consts) {
                    let len = &ta.len;
                    Some(
                        quote! { { let inner_id = #inner; registry.register_type_def(#registry::ty::Type::builder().array(inner_id, #len as u32)) } },
                    )
                } else if is_const {
                    let elem = &ta.elem;
                    let inner = quote! { { registry.register_type::<#elem>() } };
                    let len = &ta.len;
                    Some(
                        quote! { { let inner_id = #inner; registry.register_type_def(#registry::ty::Type::builder().array(inner_id, #len as u32)) } },
                    )
                } else {
                    None
                }
            }
            Type::Tuple(tt) => {
                let mut has_gen = false;
                let mut elem_tokens = Vec::new();
                for inner in &tt.elems {
                    if let Some(toks) = transform(inner, registry, params, consts) {
                        has_gen = true;
                        elem_tokens.push(toks);
                    } else {
                        elem_tokens.push(quote! { { registry.register_type::<#inner>() } });
                    }
                }
                if has_gen {
                    Some(quote! {
                        { let ids = #registry::prelude::alloc::vec![#(#elem_tokens),*]; registry.register_type_def(#registry::ty::Type::builder().tuple(ids)) }
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    transform(ty, registry, params, consts)
        .unwrap_or_else(|| quote! { { registry.register_type::<#ty>() } })
}

fn generate_fields(
    fields: &Fields,
    registry: &TokenStream2,
    params: &[String],
    consts: &[String],
) -> syn::Result<TokenStream2> {
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
            let ty_tokens = generate_field_type_tokens(&f.ty, registry, params, consts);

            Ok(quote! {
                builder = builder.#method(#args) #(.doc(#docs))* #(#anns)* .ty(#ty_tokens);
            })
        })
        .collect::<syn::Result<TokenStream2>>()
}

fn generate_def_tokens(
    data: &Data,
    registry: &TokenStream2,
    name: &Ident,
    params: &[String],
    consts: &[String],
) -> syn::Result<TokenStream2> {
    match data {
        Data::Struct(s) => {
            let fields = generate_fields(&s.fields, registry, params, consts)?;
            Ok(quote! { let mut builder = type_builder.composite(); #fields builder.build() })
        }
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|v| {
                let vname = v.ident.to_string();
                let vdocs = extract_docs(&v.attrs);
                let vanns = extract_annotations(&v.attrs)?;
                let fields = generate_fields(&v.fields, registry, params, consts)?;
                Ok(quote! {
                    builder = {
                        let mut builder = builder.add_variant(#vname) #(.doc(#vdocs))* #(#vanns)*;
                        #fields
                        builder.finish_variant()
                    };
                })
            }).collect::<syn::Result<TokenStream2>>()?;

            Ok(quote! { let mut builder = type_builder.variant(); #variants builder.build() })
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
