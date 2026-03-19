mod error;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, GenericParam, Generics, Ident, parse_macro_input};

#[proc_macro_derive(TypeInfo, attributes(type_info))]
pub fn type_info_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut context = DeriveContext::new(&input);
    match process_derive(&mut context, input.data, &input.attrs) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.into_compile_error()),
    }
}

fn process_derive(
    context: &mut DeriveContext,
    data: Data,
    attrs: &[syn::Attribute],
) -> syn::Result<proc_macro2::TokenStream> {
    context.add_trait_bounds();

    let registry = &context.registry;
    let name = &context.name;
    let name_str = name.to_string();

    let type_annotations = context.extract_annotations(attrs)?;
    let type_docs = context.extract_docs(attrs);

    let (impl_generics, ty_generics, where_clause) = context.generics.split_for_impl();

    let def_tokens = context.generate_def_tokens(data)?;
    let type_params_builder_tokens = context.generate_type_params_tokens();

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics #registry::TypeInfo for #name #ty_generics #where_clause {
            type Identity = Self;
            fn type_info(registry: &mut #registry::Registry) -> #registry::ty::Type {
                let mut type_builder = #registry::builder::TypeBuilder::new()
                    .module_path(::core::module_path!())
                    .name(#name_str)
                    .docs(#type_docs);

                #(#type_params_builder_tokens)*

                #(type_builder = type_builder.annotation(#type_annotations);)*

                #def_tokens
            }
        }
    };

    Ok(expanded)
}

struct DeriveContext {
    registry: proc_macro2::TokenStream,
    name: Ident,
    generics: Generics,
}

impl DeriveContext {
    fn new(input: &DeriveInput) -> Self {
        let registry = Self::resolve_registry_path(input);
        Self {
            registry,
            name: input.ident.clone(),
            generics: input.generics.clone(),
        }
    }

    fn resolve_registry_path(input: &DeriveInput) -> proc_macro2::TokenStream {
        for attr in &input.attrs {
            if attr.path().is_ident("type_info") {
                let mut registry_path = None;
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("crate") {
                        registry_path = Some(meta.value()?.parse::<syn::Path>()?);
                    }
                    Ok(())
                });
                if let Some(path) = registry_path {
                    return quote!(#path);
                }
            }
        }

        if let Ok(found) = proc_macro_crate::crate_name("sails-type-registry") {
            match found {
                proc_macro_crate::FoundCrate::Itself => return quote!(crate),
                proc_macro_crate::FoundCrate::Name(name) => {
                    let ident = Ident::new(&name, Span::call_site());
                    return quote!(::#ident);
                }
            }
        }

        if let Ok(found) = proc_macro_crate::crate_name("sails-rs") {
            match found {
                proc_macro_crate::FoundCrate::Itself => return quote!(crate::type_info),
                proc_macro_crate::FoundCrate::Name(name) => {
                    let ident = Ident::new(&name, Span::call_site());
                    return quote!(::#ident::type_info);
                }
            }
        }

        quote!(::sails_type_registry)
    }

    fn add_trait_bounds(&mut self) {
        let registry = &self.registry;
        for param in &mut self.generics.params {
            if let GenericParam::Type(ref mut type_param) = *param {
                type_param
                    .bounds
                    .push(syn::parse_quote!(#registry::TypeInfo));
            }
        }
    }

    fn extract_docs(&self, attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
        let registry = &self.registry;
        let mut docs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("doc")
                && let syn::Meta::NameValue(meta) = &attr.meta
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
            {
                let doc = lit_str.value();
                let doc = doc.strip_prefix(' ').unwrap_or(&doc);
                docs.push(quote! { #doc.to_string() });
            }
        }
        quote! { #registry::prelude::alloc::vec![#(#docs),*] }
    }

    fn extract_annotations(
        &self,
        attrs: &[syn::Attribute],
    ) -> syn::Result<Vec<proc_macro2::TokenStream>> {
        let registry = &self.registry;
        let mut annotations = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("type_info") {
                attr.parse_nested_meta(|meta| {
                    let ident = meta
                        .path
                        .get_ident()
                        .ok_or_else(|| meta.error("expected identifier in type_info attribute"))?;
                    let ident_str = ident.to_string();

                    if ident_str == "crate" {
                        let _: syn::Expr = meta.value()?.parse()?;
                        return Ok(());
                    }

                    if meta.input.peek(syn::Token![=]) {
                        let value: syn::Expr = meta.value()?.parse()?;

                        if let syn::Expr::Lit(expr_lit) = value {
                            match expr_lit.lit {
                                syn::Lit::Str(lit_str) => {
                                    let lit_val = lit_str.value();
                                    annotations.push(quote! {
                                        #registry::ty::Annotation::new_with_value(#ident_str, #lit_val)
                                    });
                                }
                                _ => {
                                    return Err(crate::error::MacroError::UnsupportedLiteralType
                                        .into_syn_error(expr_lit.span()));
                                }
                            }
                        } else {
                            return Err(
                                meta.error("expected string literal after '=' in type_info attribute")
                            );
                        }
                    } else {
                        annotations.push(quote! {
                            #registry::ty::Annotation::new(#ident_str)
                        });
                    }

                    Ok(())
                })?;
            }
        }

        Ok(annotations)
    }

    fn contains_generic_param(
        &self,
        ty: &syn::Type,
        type_param_names: &[String],
        const_param_names: &[String],
    ) -> bool {
        if type_param_names.is_empty() && const_param_names.is_empty() {
            return false;
        }
        match ty {
            syn::Type::Path(tp) => {
                if let Some(ident) = tp.path.get_ident()
                    && type_param_names.contains(&ident.to_string())
                {
                    return true;
                }

                if let Some(last_segment) = tp.path.segments.last()
                    && let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg
                            && self.contains_generic_param(
                                inner_ty,
                                type_param_names,
                                const_param_names,
                            )
                        {
                            return true;
                        }
                    }
                }
                false
            }
            syn::Type::Array(ta) => {
                if self.contains_generic_param(&ta.elem, type_param_names, const_param_names) {
                    return true;
                }
                if let syn::Expr::Path(ep) = &ta.len
                    && let Some(ident) = ep.path.get_ident()
                    && const_param_names.contains(&ident.to_string())
                {
                    return true;
                }
                false
            }
            syn::Type::Tuple(tt) => tt
                .elems
                .iter()
                .any(|e| self.contains_generic_param(e, type_param_names, const_param_names)),
            _ => false,
        }
    }

    fn generate_type_params_tokens(&self) -> Vec<proc_macro2::TokenStream> {
        let registry = &self.registry;
        self.generics
            .params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Type(tp) => {
                    let ty = &tp.ident;
                    let name = ty.to_string();
                    quote! { type_builder = type_builder.type_param(#name, registry.register_type::<#ty>()); }
                }
                syn::GenericParam::Const(cp) => {
                    let ident = &cp.ident;
                    let name = ident.to_string();
                    quote! { type_builder = type_builder.const_param(#name, #registry::prelude::alloc::format!("{}", #ident)); }
                }
                syn::GenericParam::Lifetime(_) => quote! {},
            })
            .collect()
    }

    fn generate_field_type_tokens(
        &self,
        ty: &syn::Type,
        type_param_names: &[String],
        const_param_names: &[String],
    ) -> proc_macro2::TokenStream {
        let registry = &self.registry;

        match ty {
            syn::Type::Path(tp) => {
                if let Some(ident) = tp.path.get_ident()
                    && type_param_names.contains(&ident.to_string())
                {
                    let ident_str = ident.to_string();
                    return quote! { #registry::ty::FieldType::Parameter(#ident_str.to_string()) };
                }

                if self.contains_generic_param(ty, type_param_names, const_param_names)
                    && let Some(last_segment) = tp.path.segments.last()
                    && let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                {
                    let mut arg_tokens = Vec::new();
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            arg_tokens.push(self.generate_field_type_tokens(
                                inner_ty,
                                type_param_names,
                                const_param_names,
                            ));
                        }
                    }

                    if !arg_tokens.is_empty() {
                        return quote! {
                            {
                                let id = registry.register_type::<#ty>();
                                let args = #registry::prelude::alloc::vec![#(#arg_tokens),*];
                                let field_type = #registry::ty::FieldType::Parameterized { id, args };
                                registry.expand_aliases(&field_type)
                            }
                        };
                    }
                }

                quote! {
                    {
                        let id = registry.register_type::<#ty>();
                        let field_type = #registry::ty::FieldType::Id(id);
                        registry.expand_aliases(&field_type)
                    }
                }
            }
            syn::Type::Array(ta) => {
                let inner_ty = &ta.elem;
                let inner_tokens =
                    self.generate_field_type_tokens(inner_ty, type_param_names, const_param_names);

                let len = &ta.len;
                let mut len_tokens = quote! { #registry::ty::ArrayLen::Static(#len as u32) };

                if let syn::Expr::Path(ep) = len
                    && let Some(ident) = ep.path.get_ident()
                    && const_param_names.contains(&ident.to_string())
                {
                    let ident_str = ident.to_string();
                    len_tokens =
                        quote! { #registry::ty::ArrayLen::Parameter(#ident_str.to_string()) };
                }

                quote! {
                    {
                        let id = registry.register_type::<#ty>();
                        let elem = #registry::prelude::alloc::boxed::Box::new(#inner_tokens);
                        let field_type = #registry::ty::FieldType::Array { id, elem, len: #len_tokens };
                        registry.expand_aliases(&field_type)
                    }
                }
            }
            syn::Type::Tuple(tt) => {
                let mut elem_tokens = Vec::new();
                for inner_ty in &tt.elems {
                    elem_tokens.push(self.generate_field_type_tokens(
                        inner_ty,
                        type_param_names,
                        const_param_names,
                    ));
                }
                quote! {
                    {
                        let id = registry.register_type::<#ty>();
                        let elems = #registry::prelude::alloc::vec![#(#elem_tokens),*];
                        let field_type = #registry::ty::FieldType::Tuple { id, elems };
                        registry.expand_aliases(&field_type)
                    }
                }
            }
            _ => quote! {
                {
                    let id = registry.register_type::<#ty>();
                    let field_type = #registry::ty::FieldType::Id(id);
                    registry.expand_aliases(&field_type)
                }
            },
        }
    }

    fn generate_fields_tokens(
        &self,
        fields: &Fields,
        is_variant: bool,
    ) -> syn::Result<proc_macro2::TokenStream> {
        let registry = &self.registry;

        let type_param_names: Vec<_> = self
            .generics
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Type(tp) => Some(tp.ident.to_string()),
                _ => None,
            })
            .collect();

        let const_param_names: Vec<_> = self
            .generics
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Const(cp) => Some(cp.ident.to_string()),
                _ => None,
            })
            .collect();

        let field_tokens = fields
            .iter()
            .map(|f| {
                let field_ty = &f.ty;
                let field_type_name = quote! { #registry::prelude::alloc::string::String::from(::core::stringify!(#field_ty)) };
                let field_annotations = self.extract_annotations(&f.attrs)?;
                let field_docs = self.extract_docs(&f.attrs);

                let methods = FieldMethods::new(f.ident.is_some(), is_variant);
                let field_method = methods.field;
                let type_name_method = methods.type_name;
                let docs_method = methods.docs;
                let annotation_method = methods.annotation;

                let field_annotations_tokens = quote! {
                    #(.#annotation_method(#field_annotations))*
                };

                let field_type_tokens = self.generate_field_type_tokens(
                    field_ty,
                    &type_param_names,
                    &const_param_names,
                );

                let field_call = if let Some(ident) = &f.ident {
                    let name = ident.to_string();
                    quote! { .#field_method(#name, #field_type_tokens) }
                } else {
                    quote! { .#field_method(#field_type_tokens) }
                };

                Ok(quote! {
                    #field_call
                    .#type_name_method(#field_type_name)
                    .#docs_method(#field_docs)
                    #field_annotations_tokens
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(quote! { #(#field_tokens)* })
    }

    fn generate_def_tokens(&self, data: Data) -> syn::Result<proc_macro2::TokenStream> {
        match data {
            Data::Struct(data_struct) => {
                let fields_tokens = self.generate_fields_tokens(&data_struct.fields, false)?;

                Ok(quote! {
                    type_builder.composite()
                        #fields_tokens
                        .build()
                })
            }
            Data::Enum(data_enum) => {
                let variants = data_enum
                    .variants
                    .iter()
                    .map(|variant| {
                        let variant_name = variant.ident.to_string();
                        let variant_docs = self.extract_docs(&variant.attrs);
                        let variant_annotations = self.extract_annotations(&variant.attrs)?;
                        let fields_tokens = self.generate_fields_tokens(&variant.fields, true)?;

                        Ok(quote! {
                            .add_variant(#variant_name)
                                .docs(#variant_docs)
                                #(.annotation(#variant_annotations))*
                                #fields_tokens
                        })
                    })
                    .collect::<syn::Result<Vec<_>>>()?;

                Ok(quote! {
                    type_builder.variant()
                        #(#variants)*
                        .build()
                })
            }
            Data::Union(_) => {
                Err(crate::error::MacroError::UnsupportedUnion.into_syn_error(self.name.span()))
            }
        }
    }
}

struct FieldMethods {
    field: proc_macro2::TokenStream,
    type_name: proc_macro2::TokenStream,
    docs: proc_macro2::TokenStream,
    annotation: proc_macro2::TokenStream,
}

impl FieldMethods {
    fn new(is_named: bool, is_variant: bool) -> Self {
        if is_variant {
            Self {
                field: if is_named {
                    quote!(field)
                } else {
                    quote!(unnamed_field)
                },
                type_name: quote!(type_name),
                docs: quote!(field_docs),
                annotation: quote!(field_annotation),
            }
        } else {
            Self {
                field: if is_named {
                    quote!(field)
                } else {
                    quote!(unnamed_field)
                },
                type_name: quote!(type_name),
                docs: quote!(docs),
                annotation: quote!(annotation),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn assert_expansion(input: DeriveInput, snapshot_name: &str) {
        let mut context = DeriveContext::new(&input);
        let expanded = process_derive(&mut context, input.data, &input.attrs).unwrap();
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
    fn basic_struct() {
        let input = parse_quote! {
            /// Documentation for basic struct
            #[type_info(skip)]
            struct Basic {
                /// Field documentation
                #[type_info(name = "custom_field")]
                a: u32,
                b: String,
            }
        };
        assert_expansion(input, "basic_struct");
    }

    #[test]
    fn generic_struct() {
        let input = parse_quote! {
            struct Generic<T, const N: usize> {
                data: [T; N],
                other: Option<T>,
            }
        };
        assert_expansion(input, "generic_struct");
    }

    #[test]
    fn complex_enum() {
        let input = parse_quote! {
            /// Multiline
            /// Enum docs
            enum Complex {
                /// Variant 1
                V1 {
                    #[type_info(rename = "val")]
                    field1: u32,
                },
                /// Variant 2
                V2(u64, String),
                /// Unit variant
                #[type_info(unit)]
                V3,
            }
        };
        assert_expansion(input, "complex_enum");
    }

    #[test]
    fn big_type() {
        let input = parse_quote! {
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
        };
        assert_expansion(input, "big_type");
    }
}
