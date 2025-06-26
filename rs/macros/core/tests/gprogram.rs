#![cfg(not(feature = "ethexe"))]

use proc_macro2::TokenStream;
use quote::quote;
use sails_macros_core::__gprogram_internal as gprogram;

#[test]
fn generates_init_for_single_ctor() {
    let input = quote! {
        impl MyProgram {
            pub async fn new(p1: u32, p2: String) -> Self {
                Self { p1, p2 }
            }
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_init_for_multiple_ctors() {
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

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_init_for_no_ctor() {
    let input = quote! {
        impl MyProgram {
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_handle_for_single_service_with_non_empty_route() {
    let input = quote! {
        impl MyProgram {
            pub fn service(&self) -> MyService {
                MyService
            }
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_handle_for_multiple_services_with_non_empty_routes() {
    let input = quote! {
        impl MyProgram {
            #[export(route = "svc1")]
            pub fn service1(&self) -> MyService {
                MyService
            }

            pub fn service2(&self) -> MyService {
                MyService
            }
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_handle_with_gprogram_attributes() {
    let args = quote!(handle_signal = my_handle_signal);
    let input = quote! {
        impl MyProgram {}
    };

    let result = gprogram(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_handle_with_crate_path() {
    let args = quote!(crate = sails_rename,);
    let input = quote! {
        impl MyProgram {}
    };

    let result = gprogram(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_ctors_meta_with_docs() {
    let input = quote! {
        impl MyProgram {
            /// This is New ctor
            pub async fn new(p1: u32, p2: String) -> Self {
                Self { p1, p2 }
            }

            /// This is New2 ctor
            /// With second line
            pub fn new2(p2: String, p1: u32) -> Self {
                Self { p1, p2 }
            }
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_init_with_unwrap_result() {
    let input = quote! {
        impl MyProgram {
            #[export(unwrap_result)]
            pub async fn new(p1: u32, p2: String) -> Result<Self, String> {
                Self { p1, p2 }
            }

            #[export(unwrap_result)]
            pub fn new2(p2: String, p1: u32) -> Result<Self, String> {
                Self { p1, p2 }
            }
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_handle_for_services_with_unwrap_result() {
    let input = quote! {
        impl MyProgram {
            #[export(route = "svc1", unwrap_result)]
            pub fn service1(&self) -> Result<MyService, String> {
                Ok(MyService)
            }

            pub fn service2(&self) -> MyService {
                MyService
            }
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_handle_with_payable() {
    let args = quote!(payable,);
    let input = quote! {
        impl MyProgram {}
    };

    let result = gprogram(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_async_main_with_handle_reply() {
    let args = quote!();
    let input = quote! {
        impl MyProgram {
            #[handle_reply]
            fn handle_reply(&self) {
                // Handle reply
            }
        }
    };

    let result = gprogram(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn generates_async_ctor_with_service() {
    let input = quote! {
        impl MyProgram {
            pub async fn new(p1: u32, p2: String) -> Self {
                Self { p1, p2 }
            }

            pub fn service(&self) -> MyService {
                MyService
            }
        }
    };

    let result = gprogram(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}
