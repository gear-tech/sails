use crate::{
    export, sails_paths,
    shared::{self, FnBuilder},
};
use args::ProgramArgs;
use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::{
    collections::BTreeMap,
    env,
    ops::{Deref, DerefMut},
};
use syn::{
    Generics, Ident, ImplItem, ImplItemFn, ItemImpl, Path, PathArguments, Receiver, ReturnType,
    Type, TypePath, Visibility, WhereClause, parse_quote, spanned::Spanned,
};

mod args;
#[cfg(feature = "ethexe")]
mod ethexe;

/// Static Spans of Program `impl` block
static mut PROGRAM_SPANS: BTreeMap<String, Span> = BTreeMap::new();

pub fn gprogram(args: TokenStream2, program_impl_tokens: TokenStream2) -> TokenStream2 {
    let program_impl = parse_gprogram_impl(program_impl_tokens);
    ensure_single_gprogram(&program_impl);
    let args = parse_args(args);
    gen_gprogram_impl(program_impl, args)
}

#[doc(hidden)]
pub fn __gprogram_internal(args: TokenStream2, program_impl_tokens: TokenStream2) -> TokenStream2 {
    let program_impl = parse_gprogram_impl(program_impl_tokens);
    let args = parse_args(args);
    gen_gprogram_impl(program_impl, args)
}

fn parse_args(args: TokenStream2) -> ProgramArgs {
    syn::parse2(args).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "failed to parse `program` attribute arguments: {}",
            err
        )
    })
}

fn parse_gprogram_impl(program_impl_tokens: TokenStream2) -> ItemImpl {
    syn::parse2(program_impl_tokens).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`program` attribute can be applied to impls only: {}",
            err
        )
    })
}

#[allow(static_mut_refs)]
fn ensure_single_gprogram(program_impl: &ItemImpl) {
    let crate_name = env::var("CARGO_CRATE_NAME").unwrap_or("crate".to_string());
    if unsafe { PROGRAM_SPANS.get(&crate_name) }.is_some() {
        abort!(
            program_impl,
            "multiple `program` attributes are not allowed"
        )
    }
    unsafe { PROGRAM_SPANS.insert(crate_name, program_impl.span()) };
}

struct ProgramBuilder {
    program_impl: ItemImpl,
    program_args: ProgramArgs,
    type_constraints: Option<WhereClause>,
}

impl ProgramBuilder {
    fn new(program_impl: ItemImpl, program_args: ProgramArgs) -> Self {
        let mut program_impl = program_impl;
        let type_constraints = program_impl.generics.where_clause.take();
        ensure_default_program_ctor(&mut program_impl);

        Self {
            program_impl,
            program_args,
            type_constraints,
        }
    }

    fn sails_path(&self) -> &Path {
        self.program_args.sails_path()
    }

    fn impl_type(&self) -> (&TypePath, &PathArguments, &Ident) {
        shared::impl_type_refs(self.program_impl.self_ty.as_ref())
    }

    fn impl_constraints(&self) -> (&Generics, Option<&WhereClause>) {
        (&self.program_impl.generics, self.type_constraints.as_ref())
    }

    fn program_ctors(&self) -> Vec<FnBuilder<'_>> {
        discover_program_ctors(&self.program_impl)
            .into_iter()
            .map(|(route, (fn_impl, _idx, unwrap_result))| {
                FnBuilder::from(route, fn_impl, unwrap_result, self.sails_path())
            })
            .collect::<Vec<_>>()
    }

    fn handle_reply_fn(&mut self) -> Option<&mut ImplItemFn> {
        let mut fn_iter = self.program_impl.items.iter_mut().filter_map(|item| {
            if let ImplItem::Fn(fn_item) = item {
                if has_handle_reply_attr(fn_item) {
                    fn_item
                        .attrs
                        .retain(|attr| !attr.path().is_ident("handle_reply"));
                    if handle_reply_predicate(fn_item) {
                        return Some(fn_item);
                    } else {
                        abort!(
                            fn_item,
                            "`handle_reply` function must have a single `&self` argument and no return type"
                        );
                    }
                }
            }
            None
        });
        let handle_reply_fn = fn_iter.next();
        if let Some(duplicate) = fn_iter.next() {
            abort!(duplicate, "only one `handle_reply` function is allowed");
        }
        handle_reply_fn
    }

    #[cfg(feature = "ethexe")]
    fn service_ctors(&self) -> Vec<FnBuilder<'_>> {
        shared::discover_invocation_targets(self, service_ctor_predicate)
            .into_iter()
            .map(|(route, (fn_impl, _idx, unwrap_result))| {
                FnBuilder::from(route, fn_impl, unwrap_result, self.sails_path())
            })
            .collect::<Vec<_>>()
    }
}

impl ProgramBuilder {
    fn wire_up_service_exposure(&mut self, program_ident: &Ident) -> (TokenStream2, TokenStream2) {
        let mut services_route = Vec::new();
        let mut services_meta = Vec::new();
        let mut invocation_dispatches = Vec::new();
        let mut routes = BTreeMap::new();
        // only used for ethexe
        #[allow(unused_mut)]
        let mut solidity_dispatchers: Vec<TokenStream2> = Vec::new();

        let item_impl = self
            .program_impl
            .items
            .iter()
            .enumerate()
            .filter_map(|(idx, impl_item)| {
                if let ImplItem::Fn(fn_item) = impl_item {
                    if service_ctor_predicate(fn_item) {
                        let (span, route, unwrap_result) = export::invocation_export(fn_item);
                        if let Some(duplicate) = routes.insert(route.clone(), fn_item.sig.ident.to_string()) {
                            abort!(
                                span,
                                "`export` or `route` attribute conflicts with one already assigned to '{}'",
                                duplicate
                            );
                        }
                        return Some((idx, route, fn_item, unwrap_result));
                    }
                }
                None
            }).map(|(idx, route, fn_item, unwrap_result)| {
                let fn_builder = FnBuilder::from(route, fn_item, unwrap_result, self.sails_path());
                let original_service_ctor_fn = fn_builder.original_service_ctor_fn();
                let wrapping_service_ctor_fn =
                    fn_builder.wrapping_service_ctor_fn(&original_service_ctor_fn.sig.ident);

                services_route.push(fn_builder.service_const_route());
                services_meta.push(fn_builder.service_meta());
                invocation_dispatches.push(fn_builder.service_invocation());
                #[cfg(feature = "ethexe")]
                solidity_dispatchers.push(fn_builder.sol_service_invocation());

                (idx, original_service_ctor_fn, wrapping_service_ctor_fn)

            })
            .collect::<Vec<_>>();

        // replace service ctor fn impls
        for (idx, original_service_ctor_fn, wrapping_service_ctor_fn, ..) in item_impl {
            self.program_impl.items[idx] = ImplItem::Fn(original_service_ctor_fn);
            self.program_impl
                .items
                .push(ImplItem::Fn(wrapping_service_ctor_fn));
        }

        let sails_path = self.sails_path();
        let (program_type_path, _program_type_args, _) = self.impl_type();
        let (generics, program_type_constraints) = self.impl_constraints();

        let program_meta_impl = quote! {
            #(#services_route)*

            impl #generics #sails_path::meta::ProgramMeta for #program_type_path #program_type_constraints {
                type ConstructorsMeta = meta_in_program::ConstructorsMeta;

                const SERVICES: &'static [(&'static str, #sails_path::meta::AnyServiceMetaFn)] = &[
                    #(#services_meta),*
                ];
            }
        };

        invocation_dispatches.push(quote! {
            { gstd::unknown_input_panic("Unexpected service", input) }
        });

        let handle_reply_fn = self.handle_reply_fn().map(|item_fn| {
            let handle_reply_fn_ident = &item_fn.sig.ident;
            quote! {
                fn __handle_reply() {
                    let program_ref = unsafe { #program_ident.as_mut() }.expect("Program not initialized");
                    program_ref.#handle_reply_fn_ident();
                }
            }
        });

        let mut args = Vec::with_capacity(2);
        if handle_reply_fn.is_some() {
            args.push(quote!(handle_reply = __handle_reply));
        }
        if let Some(handle_signal) = self.program_args.handle_signal() {
            args.push(quote!(handle_signal = #handle_signal));
        }
        let async_main_args = (!args.is_empty()).then_some(quote!((#(#args),*)));

        let solidity_main = self.sol_main(solidity_dispatchers.as_slice());

        let payable = self.program_args.payable().then(|| {
            quote! {
                if gstd::msg::value() > 0 && gstd::msg::size() == 0 {
                    return;
                }
            }
        });

        let main_fn = quote!(
            #[gstd::async_main #async_main_args]
            async fn main() {
                #payable

                let mut input: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
                let program_ref = unsafe { #program_ident.as_mut() }.expect("Program not initialized");

                #solidity_main

                #(#invocation_dispatches)else*;
            }

            #handle_reply_fn
        );

        (program_meta_impl, main_fn)
    }

    fn generate_init(&self, program_ident: &Ident) -> (TokenStream2, TokenStream2) {
        let sails_path = self.sails_path();
        let scale_codec_path = sails_paths::scale_codec_path(sails_path);
        let scale_info_path = sails_paths::scale_info_path(sails_path);

        let (program_type_path, ..) = self.impl_type();
        let input_ident = Ident::new("input", Span::call_site());

        let program_ctors = self.program_ctors();

        let mut ctor_dispatches = Vec::with_capacity(program_ctors.len() + 1);
        let mut ctor_params_structs = Vec::with_capacity(program_ctors.len());
        let mut ctor_meta_variants = Vec::with_capacity(program_ctors.len());

        for fn_builder in program_ctors {
            ctor_dispatches.push(fn_builder.ctor_branch_impl(program_type_path, &input_ident));
            ctor_params_structs
                .push(fn_builder.ctor_params_struct(&scale_codec_path, &scale_info_path));
            ctor_meta_variants.push(fn_builder.ctor_meta_variant());
        }

        ctor_dispatches.push(quote! {
            { gstd::unknown_input_panic("Unexpected ctor", input) }
        });

        let solidity_init = self.sol_init(&input_ident, program_ident);

        let init_fn = quote! {
            #[gstd::async_init]
            async fn init() {
                use gstd::InvocationIo;
                let mut #input_ident: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");

                #solidity_init

                let (program, invocation_route) = #(#ctor_dispatches)else*;
                unsafe {
                    #program_ident = Some(program);
                }
                gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
            }
        };

        let meta_in_program = quote! {
            mod meta_in_program {
                use super::*;

                #( #ctor_params_structs )*

                #[derive(#sails_path ::TypeInfo)]
                #[scale_info(crate = #scale_info_path)]
                pub enum ConstructorsMeta {
                    #( #ctor_meta_variants ),*
                }
            }
        };
        (meta_in_program, init_fn)
    }
}

// Empty ProgramBuilder Implementations without `ethexe` feature
#[cfg(not(feature = "ethexe"))]
impl ProgramBuilder {
    fn program_signature_impl(&self) -> TokenStream2 {
        quote!()
    }

    fn match_ctor_impl(&self) -> TokenStream2 {
        quote!()
    }

    fn program_const(&self) -> TokenStream2 {
        quote!()
    }

    fn sol_init(&self, _input_ident: &Ident, _program_ident: &Ident) -> TokenStream2 {
        quote!()
    }

    fn sol_main(&self, _solidity_dispatchers: &[TokenStream2]) -> TokenStream2 {
        quote!()
    }
}

impl Deref for ProgramBuilder {
    type Target = ItemImpl;

    fn deref(&self) -> &Self::Target {
        &self.program_impl
    }
}

impl DerefMut for ProgramBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.program_impl
    }
}

fn gen_gprogram_impl(program_impl: ItemImpl, program_args: ProgramArgs) -> TokenStream2 {
    let mut program_builder = ProgramBuilder::new(program_impl, program_args);

    let sails_path = program_builder.sails_path().clone();

    // Call this before `wire_up_service_exposure`
    let program_signature_impl = program_builder.program_signature_impl();
    let match_ctor_impl = program_builder.match_ctor_impl();
    let program_const = program_builder.program_const();

    let program_ident = Ident::new("PROGRAM", Span::call_site());

    let (program_meta_impl, main_fn) = program_builder.wire_up_service_exposure(&program_ident);

    let (meta_in_program, init_fn) = program_builder.generate_init(&program_ident);

    let (program_type_path, ..) = program_builder.impl_type();

    let program_impl = program_builder.deref();

    quote!(
        #program_impl

        #program_meta_impl

        #meta_in_program

        #program_signature_impl

        #program_const

        #[cfg(target_arch = "wasm32")]
        pub mod wasm {
            use super::*;
            use #sails_path::{gstd, hex, prelude::*};

            static mut #program_ident: Option<#program_type_path> = None;

            #init_fn

            #match_ctor_impl

            #main_fn
        }
    )
}

fn ensure_default_program_ctor(program_impl: &mut ItemImpl) {
    let self_type_path: TypePath = parse_quote!(Self);
    let (program_type_path, _, _) = shared::impl_type_refs(program_impl.self_ty.as_ref());

    if shared::discover_invocation_targets(program_impl, |fn_item| {
        program_ctor_predicate(fn_item, &self_type_path, program_type_path)
    })
    .is_empty()
    {
        program_impl.items.push(ImplItem::Fn(parse_quote!(
            pub fn create() -> Self {
                Self
            }
        )));
    }
}

fn discover_program_ctors(program_impl: &ItemImpl) -> BTreeMap<String, (&ImplItemFn, usize, bool)> {
    let self_type_path: TypePath = parse_quote!(Self);
    let (program_type_path, _, _) = shared::impl_type_refs(program_impl.self_ty.as_ref());
    shared::discover_invocation_targets(program_impl, |fn_item| {
        program_ctor_predicate(fn_item, &self_type_path, program_type_path)
    })
}

fn program_ctor_predicate(
    fn_item: &ImplItemFn,
    self_type_path: &TypePath,
    program_type_path: &TypePath,
) -> bool {
    if matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_none() {
        if let ReturnType::Type(_, output_type) = &fn_item.sig.output {
            if let Type::Path(output_type_path) = output_type.as_ref() {
                if output_type_path == self_type_path || output_type_path == program_type_path {
                    return true;
                }
                if let Some(Type::Path(output_type_path)) =
                    shared::extract_result_type(output_type_path)
                {
                    if output_type_path == self_type_path || output_type_path == program_type_path {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn service_ctor_predicate(fn_item: &ImplItemFn) -> bool {
    matches!(fn_item.vis, Visibility::Public(_))
        && matches!(
            fn_item.sig.receiver(),
            Some(Receiver {
                reference: Some(_),
                ..
            })
        )
        && fn_item.sig.inputs.len() == 1
        && !matches!(fn_item.sig.output, ReturnType::Default)
}

fn has_handle_reply_attr(fn_item: &ImplItemFn) -> bool {
    fn_item
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("handle_reply"))
}

fn handle_reply_predicate(fn_item: &ImplItemFn) -> bool {
    matches!(fn_item.vis, Visibility::Inherited)
        && matches!(
            fn_item.sig.receiver(),
            Some(Receiver {
                mutability: None,
                reference: Some(_),
                ..
            })
        )
        && fn_item.sig.inputs.len() == 1
        && matches!(fn_item.sig.output, ReturnType::Default)
}

impl FnBuilder<'_> {
    fn route_ident(&self) -> Ident {
        Ident::new(
            &format!("__ROUTE_{}", self.route.to_ascii_uppercase()),
            Span::call_site(),
        )
    }

    fn service_meta(&self) -> TokenStream2 {
        let sails_path = self.sails_path;
        let route = &self.route;
        let service_type = &self.result_type;
        quote!(
            ( #route , #sails_path::meta::AnyServiceMeta::new::< #service_type >)
        )
    }

    fn service_const_route(&self) -> TokenStream2 {
        let route_ident = &self.route_ident();
        let ctor_route_bytes = self.encoded_route.as_slice();
        let ctor_route_len = ctor_route_bytes.len();
        quote!(
            const #route_ident: [u8; #ctor_route_len] = [ #(#ctor_route_bytes),* ];
        )
    }

    fn service_invocation(&self) -> TokenStream2 {
        let route_ident = &self.route_ident();
        let service_ctor_ident = self.ident;
        quote! {
            if input.starts_with(& #route_ident) {
                let mut service = program_ref.#service_ctor_ident();
                let Some(is_async) = service.check_asyncness(&input[#route_ident .len()..]) else {
                    gstd::unknown_input_panic("Unknown call", &input[#route_ident .len()..])
                };
                service
                    .try_handle_async(&input[#route_ident .len()..], |encoded_result, value| {
                        gstd::msg::reply_bytes(encoded_result, value)
                            .expect("Failed to send output");
                    })
                    .await
                    .unwrap_or_else(|| {
                        gstd::unknown_input_panic("Unknown request", input)
                    });
            }
        }
    }

    fn original_service_ctor_fn(&self) -> ImplItemFn {
        let mut original_service_ctor_fn = self.impl_fn.clone();
        let original_service_ctor_fn_ident = Ident::new(
            &format!("__{}", original_service_ctor_fn.sig.ident),
            original_service_ctor_fn.sig.ident.span(),
        );
        original_service_ctor_fn.attrs.clear();
        original_service_ctor_fn.vis = Visibility::Inherited;
        original_service_ctor_fn.sig.ident = original_service_ctor_fn_ident;
        original_service_ctor_fn
    }

    fn wrapping_service_ctor_fn(&self, original_service_ctor_fn_ident: &Ident) -> ImplItemFn {
        let sails_path = self.sails_path;
        let service_type = &self.result_type;
        let route_ident = &self.route_ident();
        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));

        let mut wrapping_service_ctor_fn = self.impl_fn.clone();
        // Filter out `export  attribute
        wrapping_service_ctor_fn
            .attrs
            .retain(|attr| export::parse_attr(attr).is_none());
        wrapping_service_ctor_fn.sig.output = parse_quote!(
            -> < #service_type as #sails_path::gstd::services::Service>::Exposure
        );
        wrapping_service_ctor_fn.block = parse_quote!({
            let service = self. #original_service_ctor_fn_ident () #unwrap_token;
            let exposure = < #service_type as #sails_path::gstd::services::Service>::expose(
                service,
                #sails_path::gstd::Syscall::message_id(),
                #route_ident .as_ref(),
            );
            exposure
        });
        wrapping_service_ctor_fn
    }

    fn ctor_branch_impl(&self, program_type_path: &TypePath, input_ident: &Ident) -> TokenStream2 {
        let handler_ident = self.ident;
        let handler_await = self.is_async().then(|| quote!(.await));
        let unwrap_token = self.unwrap_result.then(|| quote!(.unwrap()));
        let handler_args = self
            .params_idents()
            .iter()
            .map(|ident| quote!(request.#ident));
        let params_struct_ident = &self.params_struct_ident;

        quote!(
            if let Ok(request) = meta_in_program::#params_struct_ident::decode_params( #input_ident) {
                let program = #program_type_path :: #handler_ident (#(#handler_args),*) #handler_await #unwrap_token;
                (program, meta_in_program::#params_struct_ident::ROUTE)
            }
        )
    }

    fn ctor_params_struct(&self, scale_codec_path: &Path, scale_info_path: &Path) -> TokenStream2 {
        let sails_path = self.sails_path;
        let params_struct_ident = &self.params_struct_ident;
        let params_struct_members = self.params().map(|(ident, ty)| quote!(#ident: #ty));
        let ctor_route_bytes = self.encoded_route.as_slice();
        let is_async = self.is_async();

        quote! {
            #[derive(#sails_path ::Decode, #sails_path ::TypeInfo)]
            #[codec(crate = #scale_codec_path )]
            #[scale_info(crate = #scale_info_path )]
            pub struct #params_struct_ident {
                #(pub(super) #params_struct_members,)*
            }

            impl #sails_path::gstd::InvocationIo for #params_struct_ident {
                const ROUTE: &'static [u8] = &[ #(#ctor_route_bytes),* ];
                type Params = Self;
                const ASYNC: bool = #is_async;
            }
        }
    }

    fn ctor_meta_variant(&self) -> TokenStream2 {
        let ctor_route = Ident::new(self.route.as_str(), Span::call_site());
        let ctor_docs_attrs = self
            .impl_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"));
        let params_struct_ident = &self.params_struct_ident;

        quote! {
            #( #ctor_docs_attrs )*
            #ctor_route(#params_struct_ident)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn gprogram_discovers_public_associated_functions_returning_self_or_the_type_as_ctors() {
        let program_impl = syn::parse2(quote!(
            impl MyProgram {
                fn non_public_associated_func_returning_self() -> Self {}
                fn non_public_associated_func_returning_type() -> MyProgram {}
                fn non_public_associated_func_returning_smth() -> u32 {}
                pub fn public_associated_func_returning_self() -> Self {}
                pub fn public_associated_func_returning_type() -> MyProgram {}
                pub fn public_associated_func_returning_smth() -> u32 {}
                fn non_public_method_returning_self(&self) -> Self {}
                fn non_public_method_returning_type(&self) -> MyProgram {}
                fn non_public_method_returning_smth(&self) -> u32 {}
                pub fn public_method_returning_self(&self) -> Self {}
                pub fn public_method_returning_type(&self) -> MyProgram {}
                pub fn public_method_returning_smth(&self) -> u32 {}
            }
        ))
        .unwrap();

        let discovered_ctors = discover_program_ctors(&program_impl)
            .iter()
            .map(|(.., (ctor_fn, ..))| ctor_fn.sig.ident.to_string())
            .collect::<Vec<_>>();

        assert_eq!(discovered_ctors.len(), 2);
        assert!(discovered_ctors.contains(&String::from("public_associated_func_returning_self")));
        assert!(discovered_ctors.contains(&String::from("public_associated_func_returning_type")));
    }

    #[test]
    fn gprogram_discovers_public_methods_with_self_ref_only_and_some_return_as_service_funcs() {
        let program_impl = syn::parse2(quote!(
            impl MyProgram {
                fn non_public_associated_func_returning_smth() -> u32 {}
                fn non_public_associated_func_returning_unit() {}
                pub fn public_associated_func_returning_smth() -> MyProgram {}
                pub fn public_associated_func_returning_unit() {}
                fn non_public_method_returning_smth(&self) -> u32 {}
                fn non_public_method_returning_unit(&self) {}
                pub fn public_method_returning_smth(&self) -> u32 {}
                pub fn public_method_returning_smth_with_other_params(&self, p1: u32) -> u32 {}
                pub fn public_methos_returning_smth_and_consuming_self(self) -> u32 {}
            }
        ))
        .unwrap();

        let discovered_services =
            shared::discover_invocation_targets(&program_impl, service_ctor_predicate)
                .iter()
                .map(|(_, (fn_impl, ..))| fn_impl.sig.ident.to_string())
                .collect::<Vec<_>>();

        assert_eq!(discovered_services.len(), 1);
        assert!(discovered_services.contains(&String::from("public_method_returning_smth")));
    }
}
