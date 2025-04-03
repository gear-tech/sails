use quote::quote;
use sails_macros_core::derive_eth_event;

#[test]
fn eth_event_basic() {
    let input = quote! {
        pub enum MyEvent {
            MyEvent1,
        }
    };
    let result = derive_eth_event(input).to_string();
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
            MyEvent2(#[indexed] u128, u128, String),
            MyEvent3,
        }
    };
    let result = derive_eth_event(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn eth_event_sails_rename() {
    let input = quote! {
        #[sails_path(crate = sails_rename)]
        pub enum Events {
            MyEvent1 {
                #[indexed]
                sender: u128,
                #[indexed]
                amount: u128,
                note: String,
            },
            MyEvent2(#[indexed] u128, u128, String),
            MyEvent3,
        }
    };
    let result = derive_eth_event(input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}
