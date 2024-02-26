use crate::shared::{Handler, ImplType};
use convert_case::{Case, Casing};
use parity_scale_codec::Encode;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::quote;
use syn::{Ident, ImplItem, ItemImpl, ReturnType, Signature, Type, TypePath, Visibility};

pub fn gprogram(program_impl_tokens: TokenStream2) -> TokenStream2 {
    let program_impl = syn::parse2::<ItemImpl>(program_impl_tokens)
        .unwrap_or_else(|err| abort!(err.span(), "Failed to parse program impl: {}", err));
    let program_type = ImplType::new(&program_impl);
    let program_type_path = program_type.path();
    let program_ident = Ident::new("PROGRAM", Span::call_site());

    let (data_structs, init) = generate_init(&program_impl, program_type_path, &program_ident);

    quote!(
        #program_impl

        #data_structs

        #[cfg(target_arch = "wasm32")]
        pub mod wasm {
            use super::*;
            use sails_rtl_gstd::{*, gstd};

            // Publicity is temporary so it can be used from module with main function
            pub(crate) static mut #program_ident: Option<#program_type_path> = None;

            #init
        }
    )
}

fn generate_init(
    program_impl: &ItemImpl,
    program_type_path: &TypePath,
    program_ident: &Ident,
) -> (TokenStream2, TokenStream2) {
    let ctor_funcs = discover_ctor_funcs(program_impl).collect::<Vec<&Signature>>();

    let input_ident = Ident::new("input", Span::call_site());

    let mut invocation_dispatches = Vec::with_capacity(ctor_funcs.len());
    let mut invocation_params_structs = Vec::with_capacity(ctor_funcs.len());

    for ctor_func in &ctor_funcs {
        let handler = Handler::from(ctor_func);

        let invocation_route = handler.func().to_string().to_case(Case::Pascal);
        let invocation_params_struct_ident =
            Ident::new(&format!("__{}Params", invocation_route), Span::call_site());

        invocation_dispatches.push({
            let invocation_route_bytes = invocation_route.encode();
            let invocation_route_len = invocation_route_bytes.len();
            let handler_ident = handler.func();
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

            quote!(
                struct #invocation_params_struct_ident {
                    #(#invocation_params_struct_members),*
                }
            )
        });
    }

    let data_structs = if ctor_funcs.is_empty() {
        quote!()
    } else {
        quote!(
            use sails_rtl_gstd::Decode as InvocationParamsStructsDecode;
            use sails_rtl_gstd::TypeInfo as InvocationParamsStructsTypeInfo;

            #(#[derive(InvocationParamsStructsDecode, InvocationParamsStructsTypeInfo)] #invocation_params_structs )*
        )
    };

    let init = if ctor_funcs.is_empty() {
        quote!(
            #[no_mangle]
            extern "C" fn init() {
                let #input_ident = gstd::msg::load_bytes().expect("Failed to read input");
                if !#input_ident.is_empty() {
                    let input = if #input_ident.len() <= 8 {
                        format!("0x{}", hex::encode(#input_ident))
                    } else {
                        format!(
                            "0x{}..{}",
                            hex::encode(&#input_ident[..4]),
                            hex::encode(&#input_ident[#input_ident.len() - 4..]))
                    };
                    panic!("Unexpected non-empty init request: {}", input);
                }
                unsafe {
                    #program_ident = Some(#program_type_path::default());
                }
                gstd::msg::reply_bytes(#input_ident, 0).expect("Failed to send output");
            }
        )
    } else {
        invocation_dispatches.push(quote!({
            let invocation_route =
                String::decode(&mut #input_ident).expect("Failed to decode invocation route");
            panic!("Unknown init request: {}", invocation_route);
        }));

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

    (data_structs, init)
}

fn discover_ctor_funcs(program_impl: &ItemImpl) -> impl Iterator<Item = &Signature> {
    let self_type_path = syn::parse_str::<TypePath>("Self").unwrap();
    let program_type = ImplType::new(program_impl);
    program_impl.items.iter().filter_map(move |item| {
        if let ImplItem::Fn(fn_item) = item {
            if matches!(fn_item.vis, Visibility::Public(_)) && fn_item.sig.receiver().is_none() {
                if let ReturnType::Type(_, output_type) = &fn_item.sig.output {
                    if let Type::Path(output_type_path) = output_type.as_ref() {
                        if output_type_path == &self_type_path
                            || output_type_path == program_type.path()
                        {
                            return Some(&fn_item.sig);
                        }
                    }
                }
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn gprogram_discovers_public_associated_functions_returning_self_or_the_type_as_ctors() {
        let program_impl = syn::parse2::<ItemImpl>(quote!(
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

        let discovered_ctors = discover_ctor_funcs(&program_impl)
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>();

        assert_eq!(discovered_ctors.len(), 2);
        assert!(discovered_ctors.contains(&String::from("public_associated_func_returning_self")));
        assert!(discovered_ctors.contains(&String::from("public_associated_func_returning_type")));
    }

    #[test]
    fn gprogram_generates_init_for_single_ctor() {
        let input = quote! {
            impl MyProgram {
                pub async fn new(p1: u32, p2: String) -> Self {
                    Self { p1, p2 }
                }
            }
        };

        let result = gprogram(input).to_string();
        let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

        insta::assert_snapshot!(result);
    }

    #[test]
    fn gprogram_generates_init_for_multiple_ctors() {
        let input = quote! {
            impl MyProgram {
                pub async fn new(p1: u32, p2: String) -> Self {
                    Self { p1, p2 }
                }

                pub fn new2(p2: String, p1: u32) -> Self {
                    Self { p1, p2 }
                }
            }
        };

        let result = gprogram(input).to_string();
        let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

        insta::assert_snapshot!(result);
    }

    #[test]
    fn gprogram_generates_init_for_no_ctor() {
        let input = quote! {
            impl MyProgram {
            }
        };

        let result = gprogram(input).to_string();
        let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

        insta::assert_snapshot!(result);
    }
}
