use crate::shared::{self, Func, ImplType};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::quote;
use std::collections::BTreeMap;
use syn::{Ident, ItemImpl, Receiver, ReturnType, Signature, Type, TypePath, Visibility};

pub fn gprogram(program_impl_tokens: TokenStream2) -> TokenStream2 {
    let program_impl = syn::parse2(program_impl_tokens)
        .unwrap_or_else(|err| abort!(err.span(), "Failed to parse program impl: {}", err));
    let program_type = ImplType::new(&program_impl);
    let program_type_path = program_type.path();
    let program_type_args = program_type.args();
    let program_type_constraints = program_type.constraints();
    let program_ident = Ident::new("PROGRAM", Span::call_site());

    let (ctors_data, init) = generate_init(&program_impl, program_type_path, &program_ident);

    let (service_types, handle) = generate_handle(&program_impl, &program_ident);

    let services = service_types.map(|item| {
        let service_route = item.0;
        let service_type = item.1;
        quote!((#service_route, sails_idl_meta::AnyServiceMeta::new::<#service_type>()))
    });

    let ctors_params_structs = ctors_data.clone().map(|item| item.2);

    let ctors_meta_variants = ctors_data.map(|item| {
        let ctor_route = Ident::new(&item.0, Span::call_site());
        let ctor_params_struct_ident = item.1;
        quote!(#ctor_route(#ctor_params_struct_ident))
    });

    quote!(
        #program_impl

        impl #program_type_args sails_idl_meta::ProgramMeta for #program_type_path #program_type_constraints {
            fn constructors() -> scale_info::MetaType {
                scale_info::MetaType::new::<meta::ConstructorsMeta>()
            }

            fn services() -> impl Iterator<Item = (&'static str, sails_idl_meta::AnyServiceMeta)> {
                [
                    #(#services),*
                ].into_iter()
            }
        }

        use sails_rtl::prelude::Decode as __ProgramDecode;
        use sails_rtl::prelude::TypeInfo as __ProgramTypeInfo;

        #(#[derive(__ProgramDecode, __ProgramTypeInfo)] #ctors_params_structs )*

        mod meta {
            use super::*;

            #[derive(__ProgramTypeInfo)]
            pub enum ConstructorsMeta {
                #(#ctors_meta_variants),*
            }
        }

        #[cfg(target_arch = "wasm32")]
        pub mod wasm {
            use super::*;
            use sails_rtl::{gstd, hex, prelude::*};

            static mut #program_ident: Option<#program_type_path> = None;

            #init

            #handle
        }
    )
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

    for (invocation_route, program_ctor) in &program_ctors {
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
                    let request = #invocation_params_struct_ident::decode(&mut &#input_ident[#invocation_route_len..]).expect("Failed to decode request");
                    let program = #program_type_path :: #handler_ident (#(#handler_args),*) #handler_await;
                    static INVOCATION_ROUTE: [u8; #invocation_route_len] = [ #(#invocation_route_bytes),* ];
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

fn generate_handle(
    program_impl: &ItemImpl,
    program_ident: &Ident,
) -> (impl Iterator<Item = (String, Option<Type>)>, TokenStream2) {
    let service_ctors = discover_service_ctors(program_impl);

    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_dispatches = Vec::with_capacity(service_ctors.len());
    let mut last_resort_invocation_dispatch = None;
    let mut service_types = Vec::with_capacity(service_ctors.len());

    for (invocation_route, service_ctor) in &service_ctors {
        let service_ctor = Func::from(service_ctor);
        let service_ctor_ident = service_ctor.ident();

        service_types.push((invocation_route.clone(), service_ctor.result().cloned()));

        if invocation_route.is_empty() {
            last_resort_invocation_dispatch = Some(quote!({
                    let program_ref = unsafe { #program_ident.as_ref() }.expect("Program not initialized");
                    let mut service = program_ref.#service_ctor_ident();
                    let output = service.handle(&#input_ident).await;
                    output
                }
            ));
        } else {
            invocation_dispatches.push({
                let invocation_route_bytes = invocation_route.encode();
                let invocation_route_len = invocation_route_bytes.len();

                quote!(
                    if #input_ident.starts_with(& [ #(#invocation_route_bytes),* ]) {
                        let program_ref = unsafe { #program_ident.as_ref() }.expect("Program not initialized");
                        let mut service = program_ref.#service_ctor_ident();
                        let output = service.handle(&#input_ident[#invocation_route_len..]).await;
                        static INVOCATION_ROUTE: [u8; #invocation_route_len] = [ #(#invocation_route_bytes),* ];
                        [INVOCATION_ROUTE.as_ref(), &output].concat()
                    }
                )
            });
        }
    }

    invocation_dispatches.push(last_resort_invocation_dispatch.unwrap_or_else(|| {
        shared::generate_unexpected_input_panic(&input_ident, "Unexpected service")
    }));

    (
        service_types.into_iter(),
        quote!(
            #[gstd::async_main]
            async fn main() {
                let mut #input_ident: &[u8] = &gstd::msg::load_bytes().expect("Failed to read input");
                let output = #(#invocation_dispatches)else*;
                gstd::msg::reply_bytes(output, 0).expect("Failed to send output");
            }
        ),
    )
}

fn discover_program_ctors<'a>(
    program_impl: &'a ItemImpl,
    program_type_path: &'a TypePath,
) -> BTreeMap<String, &'a Signature> {
    let self_type_path = syn::parse_str::<TypePath>("Self").unwrap();
    shared::discover_invocation_targets(
        program_impl,
        |fn_item| {
            if matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_none() {
                if let ReturnType::Type(_, output_type) = &fn_item.sig.output {
                    if let Type::Path(output_type_path) = output_type.as_ref() {
                        if output_type_path == &self_type_path
                            || output_type_path == program_type_path
                        {
                            return true;
                        }
                    }
                }
            }
            false
        },
        false,
    )
}

fn discover_service_ctors(program_impl: &ItemImpl) -> BTreeMap<String, &Signature> {
    shared::discover_invocation_targets(
        program_impl,
        |fn_item| {
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
        },
        true,
    )
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
            .map(|s| s.1.ident.to_string())
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

        let discovered_services = discover_service_ctors(&program_impl)
            .iter()
            .map(|s| s.1.ident.to_string())
            .collect::<Vec<_>>();

        assert_eq!(discovered_services.len(), 1);
        assert!(discovered_services.contains(&String::from("public_method_returning_smth")));
    }
}
