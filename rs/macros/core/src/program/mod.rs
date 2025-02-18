use crate::{
    export, sails_paths,
    shared::{self, FnBuilder, Func},
};
use args::ProgramArgs;
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::quote;
use std::{
    collections::BTreeMap,
    env,
    ops::{Deref, DerefMut},
};
use syn::{
    parse_quote, spanned::Spanned, Attribute, Generics, Ident, ImplItem, ImplItemFn, ItemImpl,
    Path, PathArguments, Receiver, ReturnType, Type, TypePath, Visibility, WhereClause,
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
}

impl ProgramBuilder {
    fn wire_up_service_exposure(&mut self, program_ident: &Ident) -> (TokenStream2, TokenStream2) {
        let mut services_route = Vec::new();
        let mut services_meta = Vec::new();
        let mut invocation_dispatches = Vec::new();
        let mut routes = BTreeMap::new();
        #[cfg(feature = "ethexe")]
        let mut solidity_dispatchers = Vec::new();

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

        let sails_path = &self.sails_path();
        let (program_type_path, _program_type_args, _) = self.impl_type();
        let (generics, program_type_constraints) = self.impl_constraints();

        let program_meta = quote! {
            #(#services_route)*

            impl #generics #sails_path::meta::ProgramMeta for #program_type_path #program_type_constraints {
                type ConstructorsMeta = meta_in_program::ConstructorsMeta;

                fn services() -> impl Iterator<Item = (&'static str, #sails_path::meta::AnyServiceMeta)> {
                    [
                        #(#services_meta),*
                    ].into_iter()
                }
            }
        };

        invocation_dispatches.push(quote! {
            { #sails_path::gstd::unknown_input_panic("Unexpected service", input) }
        });

        let mut args = Vec::with_capacity(2);
        if let Some(handle_reply) = self.program_args.handle_reply() {
            args.push(quote!(handle_reply = #handle_reply));
        }
        if let Some(handle_signal) = self.program_args.handle_signal() {
            args.push(quote!(handle_signal = #handle_signal));
        }
        let async_main_args = if args.is_empty() {
            quote!()
        } else {
            quote!((#(#args),*))
        };

        #[cfg(feature = "ethexe")]
        let solidity_main = {
            quote! {
                if let Ok(sig) = TryInto::<[u8; 4]>::try_into(&input[..4]) {
                    if let Some(idx) = __METHOD_SIGS.iter().position(|s| s == &sig) {
                        let (route, method) = __METHOD_ROUTES[idx];
                        #(#solidity_dispatchers)*
                    }
                }
            }
        };
        #[cfg(not(feature = "ethexe"))]
        let solidity_main = quote!();

        let main_fn = quote!(
            #[gstd::async_main #async_main_args]
            async fn main() {
                let mut input: &[u8] = &#sails_path::gstd::msg::load_bytes().expect("Failed to read input");
                let program_ref = unsafe { #program_ident.as_ref() }.expect("Program not initialized");

                #solidity_main

                let (output, value): (Vec<u8>, ValueUnit) = #(#invocation_dispatches)else*;
                #sails_path::gstd::msg::reply_bytes(output, value).expect("Failed to send output");
            }
        );

        (program_meta, main_fn)
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
    let scale_codec_path = sails_paths::scale_codec_path(&sails_path);
    let scale_info_path = sails_paths::scale_info_path(&sails_path);

    // Call this before `wire_up_service_exposure`
    #[cfg(feature = "ethexe")]
    let program_signature_impl = ethexe::program_signature_impl(&program_builder, &sails_path);
    #[cfg(not(feature = "ethexe"))]
    let program_signature_impl = quote!();

    #[cfg(feature = "ethexe")]
    let match_ctor_impl = ethexe::match_ctor_impl(&program_builder, &sails_path);
    #[cfg(not(feature = "ethexe"))]
    let match_ctor_impl = quote!();

    #[cfg(feature = "ethexe")]
    let program_const = ethexe::program_const(&program_builder.impl_type().0, &sails_path);
    #[cfg(not(feature = "ethexe"))]
    let program_const = quote!();

    let program_ident = Ident::new("PROGRAM", Span::call_site());

    let (service_tokens, main_fn) = program_builder.wire_up_service_exposure(&program_ident);

    let (program_type_path, _program_type_args, _) = program_builder.impl_type();

    let program_ctors = discover_program_ctors(&program_builder);
    let (ctors_data, init_fn) = generate_init(
        &program_ctors,
        &program_type_path,
        &program_ident,
        &sails_path,
    );

    let ctors_params_structs = ctors_data.clone().map(|item| item.2);

    let ctors_meta_variants = ctors_data.map(|item| {
        let ctor_route = Ident::new(&item.0, Span::call_site());
        let ctor_params_struct_ident = item.1;
        let ctor_docs_attrs = item.3;
        quote!(
            #( #ctor_docs_attrs )*
            #ctor_route(#ctor_params_struct_ident)
        )
    });

    let program_impl = program_builder.deref();

    quote!(
        #program_impl

        #service_tokens

        #(
            #[derive(#sails_path ::Decode, #sails_path ::TypeInfo)]
            #[codec(crate = #scale_codec_path )]
            #[scale_info(crate = #scale_info_path )]
            #[allow(dead_code)]
            #ctors_params_structs
        )*

        mod meta_in_program {
            use super::*;

            #[derive(#sails_path ::TypeInfo)]
            #[scale_info(crate = #scale_info_path)]
            pub enum ConstructorsMeta {
                #(#ctors_meta_variants),*
            }
        }

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

fn generate_init(
    program_ctors: &BTreeMap<String, (&ImplItemFn, usize, bool)>,
    program_type_path: &TypePath,
    program_ident: &Ident,
    sails_path: &Path,
) -> (
    impl Iterator<Item = (String, Ident, TokenStream2, Vec<Attribute>)> + Clone,
    TokenStream2,
) {
    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_dispatches = Vec::with_capacity(program_ctors.len() + 1);
    let mut invocation_params_structs = Vec::with_capacity(program_ctors.len());

    for (invocation_route, (program_ctor, _, unwrap_result)) in program_ctors {
        let ctor_docs_attrs: Vec<_> = program_ctor
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .cloned()
            .collect();

        let program_ctor = &program_ctor.sig;
        let handler = Func::from(program_ctor);

        let invocation_params_struct_ident =
            Ident::new(&format!("__{}Params", invocation_route), Span::call_site());

        invocation_dispatches.push({
            let invocation_route_bytes = invocation_route.encode();
            let invocation_route_len = invocation_route_bytes.len();
            let handler_ident = handler.ident();
            let handler_await = handler.is_async().then(|| quote!(.await));
            let unwrap_token = unwrap_result.then(|| quote!(.unwrap()));
            let handler_args = handler.params().iter().map(|item| {
                let param_ident = item.0;
                quote!(request.#param_ident)
            });

            quote!(
                if #input_ident.starts_with(& [ #(#invocation_route_bytes),* ]) {
                    static INVOCATION_ROUTE: [u8; #invocation_route_len] = [ #(#invocation_route_bytes),* ];
                    let request = #invocation_params_struct_ident::decode(&mut &#input_ident[#invocation_route_len..]).expect("Failed to decode request");
                    let program = #program_type_path :: #handler_ident (#(#handler_args),*) #handler_await #unwrap_token;
                    (program, INVOCATION_ROUTE.as_ref())
                }
            )
        });

        invocation_params_structs.push({
            let invocation_params_struct_members = handler.params().iter().map(|item| {
                let param_ident = item.0;
                let param_type = item.1;
                quote!(#param_ident: #param_type)
            });

            (
                invocation_route.clone(),
                invocation_params_struct_ident.clone(),
                quote!(
                    struct #invocation_params_struct_ident {
                        #(#invocation_params_struct_members),*
                    }
                ),
                ctor_docs_attrs,
            )
        });
    }

    invocation_dispatches.push(quote! {
        { #sails_path::gstd::unknown_input_panic("Unexpected ctor", input) }
    });

    #[cfg(feature = "ethexe")]
    let solidity_init = {
        quote! {
            if let Ok(sig) = TryInto::<[u8; 4]>::try_into(&#input_ident[..4]) {
                if let Some(idx) = __CTOR_SIGS.iter().position(|s| s == &sig) {
                    let (_, ctor_route) = <#program_type_path as #sails_path::solidity::ProgramSignature>::CTORS[idx];
                    unsafe {
                        #program_ident = match_ctor_solidity(ctor_route, &#input_ident[4..]).await;
                    }
                    if unsafe { #program_ident.is_some() } {
                        #sails_path::gstd::msg::reply_bytes(ctor_route, 0).expect("Failed to send output");
                        return;
                    }
                }
            }
        }
    };
    #[cfg(not(feature = "ethexe"))]
    let solidity_init = quote!();

    let init = quote!(
        #[gstd::async_init]
        async fn init() {
            #sails_path::gstd::events::__enable_events();
            let mut #input_ident: &[u8] = &#sails_path::gstd::msg::load_bytes().expect("Failed to read input");

            #solidity_init

            let (program, invocation_route) = #(#invocation_dispatches)else*;
            unsafe {
                #program_ident = Some(program);
            }
            #sails_path::gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
        }
    );

    (invocation_params_structs.into_iter(), init)
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
            pub fn default() -> Self {
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
                mutability: None,
                reference: Some(_),
                ..
            })
        )
        && fn_item.sig.inputs.len() == 1
        && !matches!(fn_item.sig.output, ReturnType::Default)
}

impl<'a> FnBuilder<'a> {
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
            ( #route , #sails_path::meta::AnyServiceMeta::new::< #service_type >())
        )
    }

    fn service_const_route(&self) -> TokenStream2 {
        let route_ident = self.route_ident();
        let ctor_route_bytes = self.encoded_route.as_slice();
        let ctor_route_len = ctor_route_bytes.len();
        quote!(
            const #route_ident: [u8; #ctor_route_len] = [ #(#ctor_route_bytes),* ];
        )
    }

    fn service_invocation(&self) -> TokenStream2 {
        let sails_path = self.sails_path;
        let route_ident = &self.route_ident();
        let service_ctor_ident = self.ident;
        quote! {
            if input.starts_with(& #route_ident) {
                let mut service = program_ref.#service_ctor_ident();
                let (output, value) = service.try_handle(&input[#route_ident .len()..]).await.unwrap_or_else(|| {
                    #sails_path::gstd::unknown_input_panic("Unknown request", input)
                });
                ([#route_ident .as_ref(), &output].concat(), value)
            }
        }
    }

    #[cfg(feature = "ethexe")]
    fn sol_service_invocation(&self) -> TokenStream2 {
        let sails_path = self.sails_path;
        let route_ident = &self.route_ident();
        let service_ctor_ident = self.ident;
        quote! {
            if route == & #route_ident {
                let mut service = program_ref.#service_ctor_ident();
                let (output, value) = service
                    .try_handle_solidity(method, &input[4..])
                    .await
                    .unwrap_or_else(|| {
                        #sails_path::gstd::unknown_input_panic("Unknown request", input)
                    });
                #sails_path::gstd::msg::reply_bytes(output, value).expect("Failed to send output");
                return;
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
                #sails_path::gstd::msg::id().into(),
                #route_ident .as_ref(),
            );
            exposure
        });
        wrapping_service_ctor_fn
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
