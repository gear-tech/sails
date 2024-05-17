use quote::quote;
use sails_macros_core::__gprogram_internal;

#[test]
fn gprogram_generates_init_for_single_ctor() {
    let input = quote! {
        impl MyProgram {
            pub async fn new(p1: u32, p2: String) -> Self {
                Self { p1, p2 }
            }
        }
    };

    let result = __gprogram_internal(input).to_string();
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

    let result = __gprogram_internal(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn gprogram_generates_init_for_no_ctor() {
    let input = quote! {
        impl MyProgram {
        }
    };

    let result = __gprogram_internal(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn gprogram_generates_handle_for_single_service_with_non_empty_route() {
    let input = quote! {
        impl MyProgram {
            pub fn service(&self) -> MyService {
                MyService
            }
        }
    };

    let result = __gprogram_internal(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn gprogram_generates_handle_for_multiple_services_with_non_empty_routes() {
    let input = quote! {
        impl MyProgram {
            #[groute("svc1")]
            pub fn service1(&self) -> MyService {
                MyService
            }

            pub fn service2(&self) -> MyService {
                MyService
            }
        }
    };

    let result = __gprogram_internal(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}
