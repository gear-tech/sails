use proc_macro2::TokenStream;
use quote::quote;
use sails_macros_core::__gservice_internal as gservice;

#[test]
fn works_with_basics() {
    let input = quote! {
        impl SomeService {
            pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                p1
            }

            pub fn this(&self, p1: bool) -> bool {
                p1
            }
        }
    };

    let result = gservice(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_lifetimes_and_generics() {
    let input = quote! {
        impl<'a, 'b, T, TEventTrigger> SomeService<'a, 'b, T, TEventTrigger>
        where T : Clone,
              TEventTrigger: EventNotifier<events::SomeEvents> {
            pub fn do_this(&mut self) -> u32 {
                42
            }
        }
    };

    let result = gservice(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_extends() {
    let args = quote! {
        extends = [ExtendedService1, ExtendedService2],
        //arg42 = "Hello, World!"
    };
    let input = quote! {
        impl SomeService {
            pub fn do_this(&mut self) -> u32 {
                42
            }
        }
    };

    let result = gservice(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}
