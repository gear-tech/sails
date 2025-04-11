use quote::quote;
use sails_macros_core::event;

#[test]
fn eth_event_basic() {
    let input = quote! {
        pub enum MyEvent {
            MyEvent1,
        }
    };
    let result = event(quote!(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn eth_event_indexed() {
    let input = quote! {
        pub enum Events {
            MyEvent1 {
                #[indexed]
                sender: u128,
                #[indexed]
                amount: u128,
                note: String,
            },
            MyEvent2(u128, u128, String),
            MyEvent3,
        }
    };
    let result = event(quote!(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn eth_event_sails_rename() {
    let attrs = quote! {
        crate = sails_rename
    };
    let input = quote! {
        pub enum Events {
            MyEvent1 {
                #[indexed]
                sender: u128,
                #[indexed]
                amount: u128,
                note: String,
            },
            MyEvent2(u128, u128, String),
            MyEvent3,
        }
    };
    let result = event(attrs, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}
