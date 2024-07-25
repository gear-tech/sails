//! Procedural macros for the `Sails` framework.

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn service(args: TokenStream, impl_tokens: TokenStream) -> TokenStream {
    sails_macros_core::gservice(args.into(), impl_tokens.into()).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn gprogram(args: TokenStream, impl_tokens: TokenStream) -> TokenStream {
    sails_macros_core::gprogram(args.into(), impl_tokens.into()).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn route(args: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    sails_macros_core::groute(args.into(), impl_item_fn_tokens.into()).into()
}
