use proc_macro2::TokenStream;
use quote::quote;
use sails_macros_core::__gservice_internal as gservice;

#[test]
fn works_with_basics() {
    let input = quote! {
        impl SomeService {
            #[export]
            pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                p1
            }

            #[export]
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
        impl<'a, 'b, T, U> SomeService<'a, 'b, T, U>
        where T : Clone,
              U: Iterator<Item = u32> {

            #[export]
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
            #[export]
            pub fn do_this(&mut self) -> u32 {
                42
            }
        }
    };

    let result = gservice(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_events() {
    let args = quote! {
        events = MyEvents,
    };
    let input = quote! {
        impl MyServiceWithEvents {
            #[export]
            pub fn do_this(&mut self) -> u32 {
                42
            }

            #[export]
            pub fn this(&self) -> bool {
                true
            }
        }
    };

    let result = gservice(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_lifetimes_and_events() {
    let args = quote! {
        events = MyEvents,
    };
    let input = quote! {
        impl<'a, T> MyGenericEventsService<'a, T>
        where
            T: Clone,
        {
            #[export]
            pub fn do_this(&mut self) -> u32 {
                42
            }
        }
    };

    let result = gservice(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_extends_and_lifetimes() {
    let args = quote! {
        extends = [BaseWithLifetime<'a>]
    };
    let input = quote! {
        impl<'a> ExtendedWithLifetime<'a> {
            #[export]
            pub fn extended_name(&self) -> String {
                "extended-name".to_string()
            }

            #[export]
            pub fn name(&self) -> String {
                "extended".to_string()
            }
        }
    };

    let result = gservice(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_methods_with_lifetimes() {
    let input = quote! {
        impl ReferenceService {
            #[export]
            pub fn baked(&self) -> &'static str {
                "Static str!"
            }

            #[export]
            pub fn incr(&mut self) -> &'static ReferenceCount {
                unsafe {
                    COUNTER.0 += 1;
                    &*ptr::addr_of!(COUNTER)
                }
            }

            #[export]
            pub fn add_byte(&mut self, byte: u8) -> &'static [u8] {
                unsafe {
                    BYTES.push(byte);
                    &*ptr::addr_of!(BYTES)
                }
            }

            #[export]
            pub async fn first_byte<'a>(&self) -> Option<&'a u8> {
                unsafe { BYTES.first() }
            }

            #[export]
            pub async fn last_byte<'a>(&self) -> Option<&'a u8> {
                unsafe { BYTES.last() }
            }
        }
    };

    let result = gservice(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_crate_path() {
    let args = quote!(crate = sails_rename,);
    let input = quote! {
        impl SomeService {
            #[export]
            pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                p1
            }

            #[export]
            pub fn this(&self, p1: bool) -> bool {
                p1
            }
        }
    };

    let result = gservice(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_reply_with_value() {
    let input = quote! {
        impl SomeService {
            #[export]
            pub async fn do_this(&mut self, p1: u32, p2: String) -> CommandReply<u32> {
                (p1, 100_000_000_000).into()
            }

            #[export]
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
fn works_with_allow_attrs() {
    let input = quote! {
        #[allow(warnings)]
        #[allow(clippy::all)]
        impl SomeService {
            #[allow(unused)]
            #[export]
            pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                p1
            }

            #[export]
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
fn works_with_docs() {
    let input = quote! {
        impl SomeService {
            /// `DoThis` command
            /// Second line
            #[export]
            pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                p1
            }

            /// `This` query
            #[export]
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
fn works_with_special_lifetimes_and_events() {
    let args = quote! {
        events = MyEvents,
    };
    let input = quote! {
        impl<T> MyGenericEventsService<'_, '_, T>
        where
            T: Clone,
        {
            #[export]
            pub fn do_this(&mut self) -> u32 {
                42
            }
        }
    };

    let result = gservice(args, input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}

#[test]
fn works_with_export() {
    let input = quote! {
        impl SomeService {
            #[export(route = "DoSomething", unwrap_result)]
            pub async fn do_this(&mut self, p1: u32, p2: String) -> Result<(u32, String), String> {
                Ok((p1, p2))
            }

            #[export]
            pub fn this(&self, p1: bool) -> bool {
                p1
            }
        }
    };

    let result = gservice(TokenStream::new(), input).to_string();
    let result = prettyplease::unparse(&syn::parse_str(&result).unwrap());

    insta::assert_snapshot!(result);
}
