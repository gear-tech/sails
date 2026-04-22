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

fn process_derive(mut input: DeriveInput) -> syn::Result<TokenStream2> {
    let registry = resolve_registry_path(&input)?;
    let name = &input.ident;
    let name_str = name.to_string();

    let type_docs = extract_docs(&input.attrs);
    let type_annotations = extract_annotations(&input.attrs)?;

    let type_param_idents: Vec<Ident> = input
        .generics
        .type_params()
        .map(|p| p.ident.clone())
        .collect();
    let type_param_defaults: Vec<Option<Type>> = input
        .generics
        .type_params()
        .map(|p| p.default.clone())
        .collect();
    let const_param_idents: Vec<Ident> = input
        .generics
        .const_params()
        .map(|p| p.ident.clone())
        .collect();
    let type_param_names: Vec<String> = type_param_idents.iter().map(|i| i.to_string()).collect();
    let const_param_names: Vec<String> = const_param_idents.iter().map(|i| i.to_string()).collect();

    let lower = LowerContext {
        registry: &registry,
        type_params: &type_param_names,
        const_params: &const_param_names,
    };

    for param in &mut input.generics.params {
        if let GenericParam::Type(tp) = param {
            tp.bounds.push(syn::parse_quote!(#registry::TypeInfo));
            tp.default = None;
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let base_name_expr = build_base_name_expr(&registry, &name_str, &const_param_idents);
    let decl_generics_expr = build_decl_generics_expr(&registry, &type_param_idents);
    let dep_hook = build_dep_hook(&input.data, &registry)?;

    let has_nominal_lookup = type_def_uses_registry(&input.data, &lower)
        || type_param_defaults
            .iter()
            .flatten()
            .any(|ty| field_uses_registry(ty, &lower));
    let type_def_param = if has_nominal_lookup {
        quote!(registry)
    } else {
        quote!(_registry)
    };

    let type_def_ctx = TypeDefContext {
        registry: &registry,
        base_name_expr: &base_name_expr,
        type_param_names: &type_param_names,
        type_param_defaults: &type_param_defaults,
        type_docs: &type_docs,
        type_annotations: &type_annotations,
        lower: &lower,
        span: name.span(),
    };
    let type_def_body = build_type_def_body(&input.data, &type_def_ctx)?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #registry::TypeInfo for #name #ty_generics #where_clause {
            type Identity = Self;

            fn module_path() -> &'static str {
                ::core::module_path!()
            }

            fn type_decl(
                registry: &mut #registry::Registry,
            ) -> #registry::ast::TypeDecl {
                let base_name = #base_name_expr;
                let generics = #decl_generics_expr;
                registry.register_named_type(
                    <Self as #registry::TypeInfo>::META,
                    base_name,
                    generics,
                    |registry| {
                        #dep_hook
                    },
                )
            }

            fn type_def(
                #type_def_param: &mut #registry::Registry,
            ) -> ::core::option::Option<#registry::ast::Type> {
                #type_def_body
            }
        }
    })
}

fn resolve_registry_path(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let mut path = None;
    for attr in &input.attrs {
        if attr.path().is_ident("type_info") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    path = Some(meta.value()?.parse::<syn::Path>()?);
                }
                Ok(())
            })?;
            if let p @ Some(_) = path {
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
    for attr in attrs.iter().filter(|a| a.path().is_ident("annotate")) {
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| meta.error("expected identifier"))?;
            let ident_str = ident.to_string();

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

fn build_base_name_expr(
    registry: &TokenStream2,
    name_str: &str,
    const_param_idents: &[Ident],
) -> TokenStream2 {
    if const_param_idents.is_empty() {
        quote!(#registry::alloc::string::String::from(#name_str))
    } else {
        let entries = const_param_idents.iter().map(|ident| {
            let ident_str = ident.to_string();
            quote!((
                #registry::alloc::string::String::from(#ident_str),
                #registry::alloc::format!("{}", #ident),
            ))
        });
        quote!(#registry::const_suffixed_name(
            #name_str,
            #registry::alloc::vec![#(#entries),*],
        ))
    }
}

fn build_decl_generics_expr(registry: &TokenStream2, type_param_idents: &[Ident]) -> TokenStream2 {
    if type_param_idents.is_empty() {
        quote!(#registry::alloc::vec::Vec::new())
    } else {
        let calls = type_param_idents
            .iter()
            .map(|tp| quote!(<#tp as #registry::TypeInfo>::type_decl(registry)));
        quote!(#registry::alloc::vec![#(#calls),*])
    }
}

fn build_dep_hook(data: &Data, registry: &TokenStream2) -> syn::Result<TokenStream2> {
    let calls: Vec<TokenStream2> = match data {
        Data::Struct(s) => s
            .fields
            .iter()
            .map(|f| {
                let ty = &f.ty;
                quote!(let _ = <#ty as #registry::TypeInfo>::type_decl(registry);)
            })
            .collect(),
        Data::Enum(e) => e
            .variants
            .iter()
            .flat_map(|v| {
                v.fields.iter().map(|f| {
                    let ty = &f.ty;
                    quote!(let _ = <#ty as #registry::TypeInfo>::type_decl(registry);)
                })
            })
            .collect(),
        Data::Union(u) => {
            return Err(syn::Error::new(
                u.union_token.span,
                "Unions are not supported by SailsTypeRegistry",
            ));
        }
    };

    if calls.is_empty() {
        Ok(quote!(let _ = registry;))
    } else {
        Ok(quote!(#(#calls)*))
    }
}

struct LowerContext<'a> {
    registry: &'a TokenStream2,
    type_params: &'a [String],
    const_params: &'a [String],
}

struct TypeDefContext<'a> {
    registry: &'a TokenStream2,
    base_name_expr: &'a TokenStream2,
    type_param_names: &'a [String],
    type_param_defaults: &'a [Option<Type>],
    type_docs: &'a [TokenStream2],
    type_annotations: &'a [TokenStream2],
    lower: &'a LowerContext<'a>,
    span: Span,
}

enum KnownPath<'a> {
    Primitive(&'static str),
    Named {
        name: &'static str,
        generics: Vec<&'a Type>,
    },
    Slice(&'a Type),
    Map(&'a Type, &'a Type),
    Transparent(&'a Type),
}

impl LowerContext<'_> {
    fn is_const_param_ident(&self, ty: &Type) -> bool {
        if let Type::Path(tp) = ty
            && let Some(ident) = tp.path.get_ident()
        {
            let name = ident.to_string();
            return self.const_params.iter().any(|c| c == &name);
        }
        false
    }

    fn lower(&self, ty: &Type) -> TokenStream2 {
        let registry = self.registry;
        match ty {
            Type::Tuple(tt) if tt.elems.is_empty() => quote! {
                #registry::ast::TypeDecl::Primitive(#registry::ast::PrimitiveType::Void)
            },
            Type::Tuple(tt) => {
                let elems = tt.elems.iter().map(|e| self.lower(e));
                quote! {
                    #registry::ast::TypeDecl::Tuple {
                        types: #registry::alloc::vec![#(#elems),*],
                    }
                }
            }
            Type::Array(ta) => {
                let inner = self.lower(&ta.elem);
                let len = &ta.len;
                quote! {
                    #registry::ast::TypeDecl::Array {
                        item: #registry::alloc::boxed::Box::new(#inner),
                        len: (#len) as u32,
                    }
                }
            }
            Type::Slice(ts) => {
                let inner = self.lower(&ts.elem);
                quote! {
                    #registry::ast::TypeDecl::Slice {
                        item: #registry::alloc::boxed::Box::new(#inner),
                    }
                }
            }
            Type::Reference(tr) => {
                if let Type::Slice(slice) = &*tr.elem {
                    let inner = self.lower(&slice.elem);
                    quote! {
                        #registry::ast::TypeDecl::Slice {
                            item: #registry::alloc::boxed::Box::new(#inner),
                        }
                    }
                } else {
                    self.lower(&tr.elem)
                }
            }
            Type::Path(tp) => self.lower_path(ty, tp),
            _ => self.lower_user_nominal(ty, None),
        }
    }

    fn lower_path(&self, ty: &Type, tp: &syn::TypePath) -> TokenStream2 {
        let registry = self.registry;

        if let Some(ident) = tp.path.get_ident() {
            let name = ident.to_string();
            if self.type_params.iter().any(|p| p == &name) {
                return quote! {
                    #registry::ast::TypeDecl::Named {
                        name: #registry::alloc::string::String::from(#name),
                        generics: #registry::alloc::vec::Vec::new(),
                    }
                };
            }
        }

        if let Some(known) = classify_known_path(tp) {
            return self.lower_known_path(known);
        }

        self.lower_user_nominal(ty, Some(tp))
    }

    fn lower_known_path(&self, known: KnownPath<'_>) -> TokenStream2 {
        let registry = self.registry;
        match known {
            KnownPath::Primitive(prim) => {
                let prim_ident = Ident::new(prim, Span::call_site());
                quote! {
                    #registry::ast::TypeDecl::Primitive(#registry::ast::PrimitiveType::#prim_ident)
                }
            }
            KnownPath::Named { name, generics } => {
                let generics = generics.iter().map(|ty| self.lower(ty));
                quote! {
                    #registry::ast::TypeDecl::Named {
                        name: #registry::alloc::string::String::from(#name),
                        generics: #registry::alloc::vec![#(#generics),*],
                    }
                }
            }
            KnownPath::Slice(inner_ty) => {
                let inner = self.lower(inner_ty);
                quote! {
                    #registry::ast::TypeDecl::Slice {
                        item: #registry::alloc::boxed::Box::new(#inner),
                    }
                }
            }
            KnownPath::Map(k_ty, v_ty) => {
                let k = self.lower(k_ty);
                let v = self.lower(v_ty);
                quote! {
                    #registry::ast::TypeDecl::Slice {
                        item: #registry::alloc::boxed::Box::new(
                            #registry::ast::TypeDecl::Tuple {
                                types: #registry::alloc::vec![#k, #v],
                            },
                        ),
                    }
                }
            }
            KnownPath::Transparent(inner_ty) => self.lower(inner_ty),
        }
    }

    fn lower_user_nominal(&self, ty: &Type, tp: Option<&syn::TypePath>) -> TokenStream2 {
        let registry = self.registry;

        let abstract_generics: Vec<TokenStream2> = tp
            .and_then(|tp| tp.path.segments.last())
            .map(|seg| {
                angle_bracketed_types(&seg.arguments)
                    .filter(|t| !self.is_const_param_ident(t))
                    .map(|t| self.lower(t))
                    .collect()
            })
            .unwrap_or_default();

        let generics_expr = if abstract_generics.is_empty() {
            quote!(#registry::alloc::vec::Vec::new())
        } else {
            quote!(#registry::alloc::vec![#(#abstract_generics),*])
        };

        quote! {
            {
                if let ::core::option::Option::Some(__registered) = registry.get_registered::<#ty>() {
                    let __name = registry
                        .get_type(__registered.type_ref)
                        .expect("registry entry exists")
                        .name
                        .clone();
                    #registry::ast::TypeDecl::Named {
                        name: __name,
                        generics: #generics_expr,
                    }
                } else {
                    <#ty as #registry::TypeInfo>::type_decl(registry)
                }
            }
        }
    }
}

fn recognize_primitive(path: &syn::Path) -> Option<&'static str> {
    let last = path.segments.last()?;
    let ident_str = last.ident.to_string();

    if path.segments.len() >= 2
        && let Some(first) = path.segments.first()
        && first.ident == "alloy_primitives"
    {
        return match ident_str.as_str() {
            "Address" => Some("H160"),
            "B256" => Some("H256"),
            _ => None,
        };
    }

    let unqualified = path.segments.len() == 1;
    let std_string = path_is_in(path, &["alloc", "std"])
        && path
            .segments
            .iter()
            .rev()
            .nth(1)
            .is_some_and(|segment| segment.ident == "string");
    let gprimitive = path_is_in(path, &["gprimitives"]);

    match ident_str.as_str() {
        "bool" if unqualified => Some("Bool"),
        "char" if unqualified => Some("Char"),
        "str" if unqualified => Some("String"),
        "String" if unqualified || std_string => Some("String"),
        "u8" if unqualified => Some("U8"),
        "u16" if unqualified => Some("U16"),
        "u32" if unqualified => Some("U32"),
        "u64" if unqualified => Some("U64"),
        "u128" if unqualified => Some("U128"),
        "i8" if unqualified => Some("I8"),
        "i16" if unqualified => Some("I16"),
        "i32" if unqualified => Some("I32"),
        "i64" if unqualified => Some("I64"),
        "i128" if unqualified => Some("I128"),
        "ActorId" if unqualified || gprimitive => Some("ActorId"),
        "CodeId" if unqualified || gprimitive => Some("CodeId"),
        "MessageId" if unqualified || gprimitive => Some("MessageId"),
        "H160" if unqualified || gprimitive => Some("H160"),
        "H256" if unqualified || gprimitive => Some("H256"),
        "U256" if unqualified || gprimitive => Some("U256"),
        _ => None,
    }
}

fn path_is_unqualified_or_in(tp: &syn::TypePath, roots: &[&str]) -> bool {
    tp.path.segments.len() == 1 || path_is_in(&tp.path, roots)
}

fn path_is_in(path: &syn::Path, roots: &[&str]) -> bool {
    path.segments
        .first()
        .is_some_and(|segment| roots.iter().any(|root| segment.ident == root))
}

fn angle_bracketed_types(args: &syn::PathArguments) -> impl Iterator<Item = &Type> {
    let ab = match args {
        syn::PathArguments::AngleBracketed(ab) => Some(ab),
        _ => None,
    };
    ab.into_iter()
        .flat_map(|args| args.args.iter())
        .filter_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
}

fn classify_known_path(tp: &syn::TypePath) -> Option<KnownPath<'_>> {
    if let Some(prim) = recognize_primitive(&tp.path) {
        return Some(KnownPath::Primitive(prim));
    }

    let last = tp.path.segments.last()?;
    let name = last.ident.to_string();
    let type_args: Vec<&Type> = angle_bracketed_types(&last.arguments).collect();

    match (name.as_str(), type_args.as_slice()) {
        ("Option", [inner_ty]) if path_is_unqualified_or_in(tp, &["core", "std"]) => {
            Some(KnownPath::Named {
                name: "Option",
                generics: vec![*inner_ty],
            })
        }
        ("Result", [ok_ty, err_ty]) if path_is_unqualified_or_in(tp, &["core", "std"]) => {
            Some(KnownPath::Named {
                name: "Result",
                generics: vec![*ok_ty, *err_ty],
            })
        }
        ("Vec" | "VecDeque" | "BTreeSet" | "BinaryHeap", [inner_ty])
            if path_is_unqualified_or_in(tp, &["alloc", "std"]) =>
        {
            Some(KnownPath::Slice(inner_ty))
        }
        ("BTreeMap", [k_ty, v_ty]) if path_is_unqualified_or_in(tp, &["alloc", "std"]) => {
            Some(KnownPath::Map(k_ty, v_ty))
        }
        ("Box" | "Rc" | "Arc", [inner_ty]) if path_is_unqualified_or_in(tp, &["alloc", "std"]) => {
            Some(KnownPath::Transparent(inner_ty))
        }
        ("Cow", [owned_ty]) if path_is_unqualified_or_in(tp, &["alloc", "std"]) => {
            Some(KnownPath::Transparent(owned_ty))
        }
        ("PhantomData", [inner_ty]) if path_is_unqualified_or_in(tp, &["core", "std"]) => {
            Some(KnownPath::Named {
                name: "PhantomData",
                generics: vec![*inner_ty],
            })
        }
        ("Range", [inner_ty]) if path_is_unqualified_or_in(tp, &["core", "std"]) => {
            Some(KnownPath::Named {
                name: "Range",
                generics: vec![*inner_ty],
            })
        }
        ("RangeInclusive", [inner_ty]) if path_is_unqualified_or_in(tp, &["core", "std"]) => {
            Some(KnownPath::Named {
                name: "RangeInclusive",
                generics: vec![*inner_ty],
            })
        }
        (name, []) if path_is_unqualified_or_in(tp, &["core", "std", "gprimitives"]) => {
            known_zero_arg_name(name).map(|name| KnownPath::Named {
                name,
                generics: Vec::new(),
            })
        }
        _ => None,
    }
}

fn known_zero_arg_name(name: &str) -> Option<&'static str> {
    match name {
        "Duration" => Some("Duration"),
        "NonZeroI8" => Some("NonZeroI8"),
        "NonZeroI16" => Some("NonZeroI16"),
        "NonZeroI32" => Some("NonZeroI32"),
        "NonZeroI64" => Some("NonZeroI64"),
        "NonZeroI128" => Some("NonZeroI128"),
        "NonZeroU8" => Some("NonZeroU8"),
        "NonZeroU16" => Some("NonZeroU16"),
        "NonZeroU32" => Some("NonZeroU32"),
        "NonZeroU64" => Some("NonZeroU64"),
        "NonZeroU128" => Some("NonZeroU128"),
        "NonZeroU256" => Some("NonZeroU256"),
        _ => None,
    }
}

fn type_def_uses_registry(data: &Data, lower: &LowerContext<'_>) -> bool {
    match data {
        Data::Struct(s) => s.fields.iter().any(|f| field_uses_registry(&f.ty, lower)),
        Data::Enum(e) => e
            .variants
            .iter()
            .any(|v| v.fields.iter().any(|f| field_uses_registry(&f.ty, lower))),
        Data::Union(_) => false,
    }
}

fn field_uses_registry(ty: &Type, lower: &LowerContext<'_>) -> bool {
    match ty {
        Type::Tuple(tt) => tt.elems.iter().any(|e| field_uses_registry(e, lower)),
        Type::Array(ta) => field_uses_registry(&ta.elem, lower),
        Type::Slice(ts) => field_uses_registry(&ts.elem, lower),
        Type::Reference(tr) => {
            if let Type::Slice(s) = &*tr.elem {
                field_uses_registry(&s.elem, lower)
            } else {
                field_uses_registry(&tr.elem, lower)
            }
        }
        Type::Path(tp) => {
            if let Some(ident) = tp.path.get_ident() {
                let name = ident.to_string();
                if lower.type_params.iter().any(|p| p == &name) {
                    return false;
                }
                if lower.const_params.iter().any(|p| p == &name) {
                    return false;
                }
            }
            if let Some(known) = classify_known_path(tp) {
                return known_path_uses_registry(known, lower);
            }
            true
        }
        _ => true,
    }
}

fn known_path_uses_registry(known: KnownPath<'_>, lower: &LowerContext<'_>) -> bool {
    match known {
        KnownPath::Primitive(_) => false,
        KnownPath::Named { generics, .. } => generics
            .iter()
            .any(|generic| field_uses_registry(generic, lower)),
        KnownPath::Slice(inner) | KnownPath::Transparent(inner) => {
            field_uses_registry(inner, lower)
        }
        KnownPath::Map(key, value) => {
            field_uses_registry(key, lower) || field_uses_registry(value, lower)
        }
    }
}

fn build_type_def_body(data: &Data, ctx: &TypeDefContext<'_>) -> syn::Result<TokenStream2> {
    let registry = ctx.registry;
    let base_name_expr = ctx.base_name_expr;
    let type_docs = ctx.type_docs;
    let type_annotations = ctx.type_annotations;
    let lower = ctx.lower;

    let param_decls: Vec<_> = ctx
        .type_param_names
        .iter()
        .zip(ctx.type_param_defaults.iter())
        .map(|(name, default)| {
            if let Some(default) = default {
                let default_decl = lower.lower(default);
                quote!(.param_with_default(#name, #default_decl))
            } else {
                quote!(.param(#name))
            }
        })
        .collect();

    match data {
        Data::Struct(s) => {
            let field_block = build_composite_fields(&s.fields, lower)?;
            Ok(quote! {
                let base_name = #base_name_expr;
                ::core::option::Option::Some({
                    let __builder = #registry::builder::TypeBuilder::new()
                        .name(base_name)
                        #(.doc(#type_docs))*
                        #(#type_annotations)*
                        #(#param_decls)*;
                    let composite = __builder.composite();
                    #field_block
                })
            })
        }
        Data::Enum(e) => {
            let variant_blocks = e
                .variants
                .iter()
                .map(|v| {
                    let vname = v.ident.to_string();
                    let vdocs = extract_docs(&v.attrs);
                    let vanns = extract_annotations(&v.attrs)?;
                    let field_block = build_variant_fields(&v.fields, lower)?;
                    Ok(quote! {
                        {
                            let variant = variants.add_variant(#vname)
                                #(.doc(#vdocs))*
                                #(#vanns)*;
                            #field_block
                        }
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?;

            Ok(quote! {
                let base_name = #base_name_expr;
                ::core::option::Option::Some({
                    let __builder = #registry::builder::TypeBuilder::new()
                        .name(base_name)
                        #(.doc(#type_docs))*
                        #(#type_annotations)*
                        #(#param_decls)*;
                    let mut variants = __builder.variant();
                    #(variants = #variant_blocks;)*
                    variants.build()
                })
            })
        }
        Data::Union(_) => Err(syn::Error::new(
            ctx.span,
            "Unions are not supported by SailsTypeRegistry",
        )),
    }
}

fn build_composite_fields(fields: &Fields, lower: &LowerContext<'_>) -> syn::Result<TokenStream2> {
    if fields.is_empty() {
        return Ok(quote!(composite.build()));
    }

    let mut chain = quote!(composite);
    for f in fields.iter() {
        let docs = extract_docs(&f.attrs);
        let anns = extract_annotations(&f.attrs)?;
        let start = match f.ident.as_ref() {
            Some(ident) => {
                let n = ident.to_string();
                quote!(.field(#n))
            }
            None => quote!(.unnamed()),
        };
        let ty_tokens = lower.lower(&f.ty);
        chain = quote! {
            #chain
            #start
            #(.doc(#docs))*
            #(#anns)*
            .ty(#ty_tokens)
        };
    }
    Ok(quote!(#chain.build()))
}

fn build_variant_fields(fields: &Fields, lower: &LowerContext<'_>) -> syn::Result<TokenStream2> {
    if fields.is_empty() {
        return Ok(quote!(variant.finish_variant()));
    }

    let mut chain = quote!(variant);
    for f in fields.iter() {
        let docs = extract_docs(&f.attrs);
        let anns = extract_annotations(&f.attrs)?;
        let start = match f.ident.as_ref() {
            Some(ident) => {
                let n = ident.to_string();
                quote!(.field(#n))
            }
            None => quote!(.unnamed()),
        };
        let ty_tokens = lower.lower(&f.ty);
        chain = quote! {
            #chain
            #start
            #(.doc(#docs))*
            #(#anns)*
            .ty(#ty_tokens)
        };
    }
    Ok(quote!(#chain.finish_variant()))
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
