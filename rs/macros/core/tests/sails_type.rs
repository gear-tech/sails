use quote::quote;
use sails_macros_core::sails_type;

fn format(tokens: proc_macro2::TokenStream) -> String {
    let text = tokens.to_string();
    prettyplease::unparse(&syn::parse_str(&text).expect("sails_type output must parse"))
}

#[test]
fn struct_basic() {
    let input = quote! {
        pub struct MyType {
            pub a: u32,
            pub b: String,
        }
    };
    let result = format(sails_type(quote!(), input));
    insta::assert_snapshot!(result);
}

#[test]
fn enum_basic() {
    let input = quote! {
        pub enum MyEnum {
            A,
            B(u32),
            C { x: String },
        }
    };
    let result = format(sails_type(quote!(), input));
    insta::assert_snapshot!(result);
}

#[test]
fn struct_with_generics() {
    let input = quote! {
        pub struct MyType<T: Clone> where T: core::fmt::Debug {
            pub value: T,
        }
    };
    let result = format(sails_type(quote!(), input));
    insta::assert_snapshot!(result);
}

#[test]
fn custom_crate_path() {
    let attrs = quote! { crate = sails_rename };
    let input = quote! { pub struct MyType { pub a: u32 } };
    let result = format(sails_type(attrs, input));
    insta::assert_snapshot!(result);
}

#[test]
fn no_reflect_hash_flag() {
    let attrs = quote! { no_reflect_hash };
    let input = quote! { pub struct MyType { pub a: u32 } };
    let result = format(sails_type(attrs, input));
    insta::assert_snapshot!(result);
}

#[test]
fn custom_crate_path_and_no_reflect_hash() {
    let attrs = quote! { crate = sails_rename, no_reflect_hash };
    let input = quote! { pub struct MyType { pub a: u32 } };
    let result = format(sails_type(attrs, input));
    insta::assert_snapshot!(result);
}
