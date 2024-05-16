use crate::{
    sails_paths,
    shared::{self, Func, ImplType},
};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::quote;
use std::collections::BTreeMap;
use syn::{
    parse_quote, Ident, ImplItem, ImplItemFn, ItemImpl, Receiver, ReturnType, Type, TypePath,
    Visibility,
};

pub fn gprogram(program_impl_tokens: TokenStream2) -> TokenStream2 {
    let program_impl = syn::parse2(program_impl_tokens).unwrap_or_else(|err| {
        abort!(
            err.span(),
            "`gprogram` attribute can be applied to impls only: {}",
            err
        )
    });

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
                    ( #route , sails_rtl::meta::AnyServiceMeta::new::< #service_type >())
                )
            };

            wire_up_service_exposure(&mut program_impl, &route_ident, ctor_fn, ctor_idx);

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
    );

    let services_meta = services_data.iter().map(|item| &item.1);

    let services_routes = services_data.iter().map(|item| &item.0);

    let program_type = ImplType::new(&program_impl);
    let program_type_path = program_type.path();
    let program_type_args = program_type.args();
    let program_type_constraints = program_type.constraints();

    let (ctors_data, init_fn) = generate_init(&program_impl, program_type_path, &program_ident);

    let ctors_params_structs = ctors_data.clone().map(|item| item.2);

    let ctors_meta_variants = ctors_data.map(|item| {
        let ctor_route = Ident::new(&item.0, Span::call_site());
        let ctor_params_struct_ident = item.1;
        quote!(#ctor_route(#ctor_params_struct_ident))
    });

    let scale_types_path = sails_paths::scale_types_path();
    let scale_codec_path = sails_paths::scale_codec_path();
    let scale_info_path = sails_paths::scale_info_path();

    quote!(
        #(#services_routes)*

        #program_impl

        impl #program_type_args sails_rtl::meta::ProgramMeta for #program_type_path #program_type_constraints {
            fn constructors() -> scale_info::MetaType {
                scale_info::MetaType::new::<meta::ConstructorsMeta>()
            }

            fn services() -> impl Iterator<Item = (&'static str, sails_rtl::meta::AnyServiceMeta)> {
                [
                    #(#services_meta),*
                ].into_iter()
            }
        }

        use #scale_types_path ::Decode as __ProgramDecode;
        use #scale_types_path ::TypeInfo as __ProgramTypeInfo;

        #(
            #[derive(__ProgramDecode, __ProgramTypeInfo)]
            #[codec(crate = #scale_codec_path )]
            #[scale_info(crate = #scale_info_path )]
            #ctors_params_structs
        )*

        mod meta {
            use super::*;

            #[derive(__ProgramTypeInfo)]
            #[scale_info(crate = #scale_info_path )]
            pub enum ConstructorsMeta {
                #(#ctors_meta_variants),*
            }
        }

        #[cfg(target_arch = "wasm32")]
        pub mod wasm {
            use super::*;
            use sails_rtl::{gstd, hex, prelude::*};

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
        -> < #service_type as sails_rtl::gstd::services::Service>::Exposure
    );
    wrapping_service_ctor_fn.block = parse_quote!({
        let service = self. #original_service_ctor_fn_ident ();
        let exposure = < #service_type as sails_rtl::gstd::services::Service>::expose(
            service,
            sails_rtl::gstd::msg::id().into(),
            #route_ident .as_ref(),
        );
        exposure
    });
    program_impl.items[ctor_idx] = ImplItem::Fn(wrapping_service_ctor_fn);
}

fn generate_init(
    program_impl: &ItemImpl,
    program_type_path: &TypePath,
    program_ident: &Ident,
) -> (
    impl Iterator<Item = (String, Ident, TokenStream2)> + Clone,
    TokenStream2,
) {
    let program_ctors = discover_program_ctors(program_impl, program_type_path);

    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_dispatches = Vec::with_capacity(program_ctors.len());
    let mut invocation_params_structs = Vec::with_capacity(program_ctors.len());

    for (invocation_route, (program_ctor, ..)) in &program_ctors {
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
            )
        });
    }

    let init = if program_ctors.is_empty() {
        let unexpected_ctor_panic =
            shared::generate_unexpected_input_panic(&input_ident, "Unexpected ctor");

        quote!(
            #[no_mangle]
            extern "C" fn init() {
                let #input_ident = gstd::msg::load_bytes().expect("Failed to read input");
                if !#input_ident.is_empty() {
                    #unexpected_ctor_panic
                }
                unsafe {
                    #program_ident = Some(#program_type_path::default());
                }
                gstd::msg::reply_bytes(#input_ident, 0).expect("Failed to send output");
            }
        )
    } else {
        invocation_dispatches.push(shared::generate_unexpected_input_panic(
            &input_ident,
            "Unexpected ctor",
        ));

        quote!(
            #[gstd::async_init]
            async fn init() {
                let mut #input_ident: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
                let (program, invocation_route) = #(#invocation_dispatches)else*;
                unsafe {
                    #program_ident = Some(program);
                }
                gstd::msg::reply_bytes(invocation_route, 0).expect("Failed to send output");
            }
        )
    };

    (invocation_params_structs.into_iter(), init)
}

fn generate_handle<'a>(
    program_ident: &'a Ident,
    service_ctors: impl Iterator<Item = (&'a Ident, &'a Ident)>,
) -> TokenStream2 {
    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_dispatches = Vec::new();

    for (service_route_ident, service_ctor_ident) in service_ctors {
        invocation_dispatches.push({
            quote!(
                if #input_ident.starts_with(& #service_route_ident) {
                    let program_ref = unsafe { #program_ident.as_ref() }.expect("Program not initialized");
                    let mut service = program_ref.#service_ctor_ident();
                    let output = service.handle(&#input_ident[#service_route_ident .len()..]).await;
                    [#service_route_ident .as_ref(), &output].concat()
                }
            )
        });
    }

    invocation_dispatches.push(shared::generate_unexpected_input_panic(
        &input_ident,
        "Unexpected service",
    ));

    quote!(
        #[gstd::async_main]
        async fn main() {
            let mut #input_ident: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
            let output: Vec<u8> = #(#invocation_dispatches)else*;
            gstd::msg::reply_bytes(output, 0).expect("Failed to send output");
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
        let program_type_path = ImplType::new(&program_impl).path().clone();

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
