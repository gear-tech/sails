use crate::{
    sails_paths,
    shared::{self, Func},
};
use args::ProgramArgs;
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::quote;
use std::{collections::BTreeMap, env};
use syn::{
    parse_quote, spanned::Spanned, Attribute, Ident, ImplItem, ImplItemFn, ItemImpl, Path,
    Receiver, ReturnType, Type, TypePath, Visibility,
};

mod args;

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

fn gen_gprogram_impl(program_impl: ItemImpl, program_args: ProgramArgs) -> TokenStream2 {
    let sails_path = program_args.sails_path();
    let scale_codec_path = sails_paths::scale_codec_path(&sails_path);
    let scale_info_path = sails_paths::scale_info_path(&sails_path);

    let services_ctors = discover_services_ctors(&program_impl);

    let mut program_impl = program_impl.clone();

    let services_data = services_ctors
        .into_iter()
        .map(|(route, (ctor_fn, ctor_idx))| {
            let route_ident = Ident::new(
                &format!("__ROUTE_{}", route.to_ascii_uppercase()),
                Span::call_site(),
            );

            let route_static = {
                let ctor_route_bytes = route.encode();
                let ctor_route_len = ctor_route_bytes.len();
                quote!(
                    static #route_ident: [u8; #ctor_route_len] = [ #(#ctor_route_bytes),* ];
                )
            };

            let service_meta = {
                let service_type = shared::result_type(&ctor_fn.sig);
                quote!(
                    ( #route , #sails_path::meta::AnyServiceMeta::new::< #service_type >())
                )
            };

            wire_up_service_exposure(
                &mut program_impl,
                &route_ident,
                ctor_fn,
                ctor_idx,
                &sails_path,
            );

            (
                route_static,
                service_meta,
                ctor_fn.sig.ident.clone(),
                route_ident,
            )
        })
        .collect::<Vec<_>>();

    let program_ident = Ident::new("PROGRAM", Span::call_site());

    let handle_fn = generate_handle(
        &program_ident,
        services_data.iter().map(|item| (&item.3, &item.2)),
        program_args,
        &sails_path,
    );

    let services_meta = services_data.iter().map(|item| &item.1);

    let services_routes = services_data.iter().map(|item| &item.0);

    let (program_type_path, _program_type_args, _) = shared::impl_type(&program_impl);
    let (generics, program_type_constraints) = shared::impl_constraints(&program_impl);

    let (ctors_data, init_fn) = generate_init(
        &mut program_impl,
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

    quote!(
        #(#services_routes)*

        #program_impl

        impl #generics #sails_path::meta::ProgramMeta for #program_type_path #program_type_constraints {
            fn constructors() -> #scale_info_path::MetaType {
                #scale_info_path::MetaType::new::<meta_in_program::ConstructorsMeta>()
            }

            fn services() -> impl Iterator<Item = (&'static str, #sails_path::meta::AnyServiceMeta)> {
                [
                    #(#services_meta),*
                ].into_iter()
            }
        }

        use #sails_path ::Decode as __ProgramDecode;
        use #sails_path ::TypeInfo as __ProgramTypeInfo;

        #(
            #[derive(__ProgramDecode, __ProgramTypeInfo)]
            #[codec(crate = #scale_codec_path )]
            #[scale_info(crate = #scale_info_path )]
            #[allow(dead_code)]
            #ctors_params_structs
        )*

        mod meta_in_program {
            use super::*;

            #[derive(__ProgramTypeInfo)]
            #[scale_info(crate = #scale_info_path)]
            pub enum ConstructorsMeta {
                #(#ctors_meta_variants),*
            }
        }

        #[cfg(target_arch = "wasm32")]
        pub mod wasm {
            use super::*;
            use #sails_path::{gstd, hex, prelude::*};

            static mut #program_ident: Option<#program_type_path> = None;

            #init_fn

            #handle_fn
        }
    )
}

fn wire_up_service_exposure(
    program_impl: &mut ItemImpl,
    route_ident: &Ident,
    ctor_fn: &ImplItemFn,
    ctor_idx: usize,
    sails_path: &Path,
) {
    let service_type = shared::result_type(&ctor_fn.sig);

    let mut original_service_ctor_fn = ctor_fn.clone();
    let original_service_ctor_fn_ident = Ident::new(
        &format!("__{}", original_service_ctor_fn.sig.ident),
        original_service_ctor_fn.sig.ident.span(),
    );
    original_service_ctor_fn.attrs.clear();
    original_service_ctor_fn.vis = Visibility::Inherited;
    original_service_ctor_fn.sig.ident = original_service_ctor_fn_ident.clone();
    program_impl
        .items
        .push(ImplItem::Fn(original_service_ctor_fn));

    let mut wrapping_service_ctor_fn = ctor_fn.clone();
    wrapping_service_ctor_fn.sig.output = parse_quote!(
        -> < #service_type as #sails_path::gstd::services::Service>::Exposure
    );
    wrapping_service_ctor_fn.block = parse_quote!({
        let service = self. #original_service_ctor_fn_ident ();
        let exposure = < #service_type as #sails_path::gstd::services::Service>::expose(
            service,
            #sails_path::gstd::msg::id().into(),
            #route_ident .as_ref(),
        );
        exposure
    });
    program_impl.items[ctor_idx] = ImplItem::Fn(wrapping_service_ctor_fn);
}

fn generate_init(
    program_impl: &mut ItemImpl,
    program_type_path: &TypePath,
    program_ident: &Ident,
    sails_path: &Path,
) -> (
    impl Iterator<Item = (String, Ident, TokenStream2, Vec<Attribute>)> + Clone,
    TokenStream2,
) {
    if discover_program_ctors(program_impl, program_type_path).is_empty() {
        program_impl.items.push(ImplItem::Fn(parse_quote!(
            pub fn default() -> Self {
                Self
            }
        )));
    }

    let program_ctors = discover_program_ctors(program_impl, program_type_path);

    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_dispatches = Vec::with_capacity(program_ctors.len());
    let mut invocation_params_structs = Vec::with_capacity(program_ctors.len());

    for (invocation_route, (program_ctor, ..)) in &program_ctors {
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
            let handler_args = handler.params().iter().map(|item| {
                let param_ident = item.0;
                quote!(request.#param_ident)
            });

            quote!(
                if #input_ident.starts_with(& [ #(#invocation_route_bytes),* ]) {
                    static INVOCATION_ROUTE: [u8; #invocation_route_len] = [ #(#invocation_route_bytes),* ];
                    let request = #invocation_params_struct_ident::decode(&mut &#input_ident[#invocation_route_len..]).expect("Failed to decode request");
                    let program = #program_type_path :: #handler_ident (#(#handler_args),*) #handler_await;
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

    invocation_dispatches.push(shared::generate_unexpected_input_panic(
        &input_ident,
        "Unexpected ctor",
        sails_path,
    ));

    let init = quote!(
        #[gstd::async_init]
        async fn init() {
            #sails_path::gstd::events::__enable_events();
            let mut #input_ident: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
            let (program, invocation_route) = #(#invocation_dispatches)else*;
            unsafe {
                #program_ident = Some(program);
            }
            gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
        }
    );

    (invocation_params_structs.into_iter(), init)
}

fn generate_handle<'a>(
    program_ident: &'a Ident,
    service_ctors: impl Iterator<Item = (&'a Ident, &'a Ident)>,
    program_args: ProgramArgs,
    sails_path: &Path,
) -> TokenStream2 {
    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_dispatches = Vec::new();

    for (service_route_ident, service_ctor_ident) in service_ctors {
        invocation_dispatches.push({
            quote!(
                if #input_ident.starts_with(& #service_route_ident) {
                    let program_ref = unsafe { #program_ident.as_ref() }.expect("Program not initialized");
                    let mut service = program_ref.#service_ctor_ident();
                    let (output, value) = service.handle(&#input_ident[#service_route_ident .len()..]).await;
                    ([#service_route_ident .as_ref(), &output].concat(), value)
                }
            )
        });
    }

    invocation_dispatches.push(shared::generate_unexpected_input_panic(
        &input_ident,
        "Unexpected service",
        sails_path,
    ));

    let mut args = Vec::with_capacity(2);
    if let Some(handle_reply) = program_args.handle_reply() {
        args.push(quote!(handle_reply = #handle_reply));
    }
    if let Some(handle_signal) = program_args.handle_signal() {
        args.push(quote!(handle_signal = #handle_signal));
    }
    let async_main_args = if args.is_empty() {
        quote!()
    } else {
        quote!((#(#args),*))
    };

    quote!(
        #[gstd::async_main #async_main_args]
        async fn main() {
            let mut #input_ident: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
            let (output, value): (Vec<u8>, ValueUnit) = #(#invocation_dispatches)else*;
            gstd::msg::reply_bytes(output, value).expect("Failed to send output");
        }
    )
}

fn discover_program_ctors<'a>(
    program_impl: &'a ItemImpl,
    program_type_path: &'a TypePath,
) -> BTreeMap<String, (&'a ImplItemFn, usize)> {
    let self_type_path = syn::parse_str::<TypePath>("Self").unwrap();
    shared::discover_invocation_targets(program_impl, |fn_item| {
        if matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_none() {
            if let ReturnType::Type(_, output_type) = &fn_item.sig.output {
                if let Type::Path(output_type_path) = output_type.as_ref() {
                    if output_type_path == &self_type_path || output_type_path == program_type_path
                    {
                        return true;
                    }
                }
            }
        }
        false
    })
}

fn discover_services_ctors(program_impl: &ItemImpl) -> BTreeMap<String, (&ImplItemFn, usize)> {
    shared::discover_invocation_targets(program_impl, |fn_item| {
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
    })
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
        let (program_type_path, ..) = shared::impl_type(&program_impl);

        let discovered_ctors = discover_program_ctors(&program_impl, &program_type_path)
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

        let discovered_services = discover_services_ctors(&program_impl)
            .iter()
            .map(|(_, (fn_impl, _))| fn_impl.sig.ident.to_string())
            .collect::<Vec<_>>();

        assert_eq!(discovered_services.len(), 1);
        assert!(discovered_services.contains(&String::from("public_method_returning_smth")));
    }
}
