use quote::quote;
use sails_macros_core::gservice;

#[test]
fn gservice_works() {
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

    let result = gservice(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn gservice_works_for_lifetimes_and_generics() {
    let input = quote! {
        impl<'a, 'b, T, TEventTrigger> SomeService<'a, 'b, T, TEventTrigger>
        where T : Clone,
              TEventTrigger: EventTrigger<events::SomeEvents> {
            pub fn do_this(&mut self) -> u32 {
                42
            }
        }
    };

    let result = gservice(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}
