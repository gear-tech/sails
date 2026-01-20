//! Procedural macros for the `Sails` framework.

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

/// Generates code for turning a Rust impl block into a Sails service
/// based on a set of public methods of the block. See
/// [documentation](https://github.com/gear-tech/sails?tab=readme-ov-file#application)
/// for details.
///
/// The macro can be customized with the following arguments:
/// - `crate` - specifies path to the `sails-rs` crate allowing the latter
///             to be imported with a different name, for example, when the
///             `sails-rs` create is re-exported from another crate.
/// - `events` - specifies a Rust enum type denoting events that the service can emit.
///              See [documentation](https://github.com/gear-tech/sails?tab=readme-ov-file#events)
///              for details.
/// - `extends` - specifies a list of other services the service extends using the mixin pattern.
///               See [documentation](https://github.com/gear-tech/sails?tab=readme-ov-file#service-extending-mixins)
///               for details.
///
/// # Examples
///
/// ```rust
/// mod my_service {
///     use sails_rs::{export, service, prelude::*};
///
///     #[event]
///     #[derive(parity_scale_codec::Encode, scale_info::TypeInfo, ReflectHash)]
///     #[reflect_hash(crate = sails_rs)]
///     pub enum MyServiceEvents {
///         SomethingDone,
///     }
///
///     pub struct MyService;
///
///     #[service(events = MyServiceEvents)]
///     impl MyService {
///         #[export]
///         pub fn do_something(&mut self) -> u32 {
///             self.emit_event(MyServiceEvents::SomethingDone).unwrap();
///             0
///         }
///
///         #[export]
///         pub fn get_something(&self) -> u32 {
///             0
///         }
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn service(args: TokenStream, impl_tokens: TokenStream) -> TokenStream {
    sails_macros_core::gservice(args.into(), impl_tokens.into()).into()
}

/// Generates code for turning a Rust impl block into a Sails program
/// based on a set of public methods of the block. See
/// [documentation](https://github.com/gear-tech/sails?tab=readme-ov-file#application)
/// for details.
///
/// The macro can be customized with the following arguments:
/// - `crate` - specifies path to the `sails-rs` crate allowing the latter
///             to be imported with a different name, for example, when the
///             `sails-rs` create is re-exported from another crate.
/// - `handle_signal` - specifies a path to a function that will be called
///                     after standard signal handling provided by the `gstd` crate.
/// - `payable` - specifies that the program can accept transfers of value.
///
/// The macro also accepts a `handle_reply` attribute that can be used to specify a function
/// that will handle replies. This function should be defined within the program and accepts `&self`.
/// The function will be called automatically when a reply is received.
///
/// # Examples
///
/// ```rust
/// mod my_program {
///     use sails_rs::program;
///
///     pub struct MyProgram;
///
///     #[program(payable)]
///     impl MyProgram {
///         pub fn default() -> Self {
///             Self
///         }
///
///         pub fn from_seed(_seed: u32) -> Self {
///             Self
///         }
///
///         #[handle_reply]
///         fn inspect_reply(&self) {
///             // Handle reply here
///         }
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn program(args: TokenStream, impl_tokens: TokenStream) -> TokenStream {
    sails_macros_core::gprogram(args.into(), impl_tokens.into()).into()
}

/// Customizes how a service/program method is exposed based on specified arguments.
///
/// The attribute accepts two optional arguments:
/// - `route` - Defines  a custom route for the method.
///    By default, every exposed service method is accessible via a route derived from its name,
///    converted to PascalCase. This argument allows you to override the default route with a
///    string of your choice.
/// - `unwrap_result` - Indicates that the method's `Result<T, E>` return value should be unwrapped.
///   If specified, the method will panic if the result is an `Err`.
///
/// # Examples
///
/// The following example demonstrates the use of the `export` attribute applied to the `do_something` method.
/// - The `route` argument customizes the route to "Something" (convertered to PascalCase).
/// - The `unwrap_result` argument ensures that the method's result is unwrapped, causing it to panic
///   with the message "Something went wrong" if the result is an `Err`.
///
/// ```rust
/// mod my_service {
///    use sails_rs::{export, service};
///
///    struct MyService;
///
///    #[service]
///    impl MyService {
///        #[export(route = "something", unwrap_result)]
///        pub fn do_something(&mut self) -> Result<u32, String> {
///            Err("Something went wrong".to_string())
///        }
///    }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn export(args: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    sails_macros_core::export(args.into(), impl_item_fn_tokens.into()).into()
}

/// Defines event for using within Gear and Ethereum ecosystem.
///
/// Trait `SailsEvent` provides a uniform interface to encode an event into a tuple
/// of variant name and data payload.
///
/// Trait `EthEvent` provides a uniform interface to convert an event into the topics and data payload
/// that are used to emit logs in the Ethereum Virtual Machine (EVM). The logs generated by the EVM
/// consist of:
///
/// - **Topics:** An array of 32-byte values. The first topic is always the keccak256 hash of the event
///   signature, while the remaining topics correspond to indexed fields. For dynamic types (as determined
///   by `<T as alloy_sol_types::SolType>::IS_DYNAMIC`), the ABI-encoded value is hashed before being stored.
///   For static types, the ABI-encoded value is left-padded with zeros to 32 bytes.
/// - **Data:** A byte array containing the ABI-encoded non-indexed fields of the event, encoded as a tuple.
///
/// This is intended to be used with the `#[sails_rs::event]` procedural macro, which automatically
/// implements the trait for your enum-based event definitions.
///
/// # Examples
///
/// Given an event definition:
///
/// ```rust,ignore
/// #[sails_rs::event]
/// #[derive(sails_rs::Encode, sails_rs::TypeInfo)]
/// #[codec(crate = sails_rs::scale_codec)]
/// #[scale_info(crate = sails_rs::scale_info)]
/// pub enum Events {
///     MyEvent {
///         #[indexed]
///         sender: uint128,
///         amount: uint128,
///         note: String,
///     },
/// }
/// ```
///
/// Calling the methods:
///
/// ```rust,ignore
/// let event = Events::MyEvent {
///     sender: 123,
///     amount: 1000,
///     note: "Hello, Ethereum".to_owned(),
/// };
///
/// let topics = event.topics();
/// let data = event.data();
/// ```
///
/// The first topic will be the hash of the event signature (e.g. `"MyEvent(uint128,uint128,String)"`),
/// and additional topics and the data payload will be computed based on the field attributes.
///
/// # Methods
///
/// - `topics()`: Returns a vector of 32-byte topics (`alloy_primitives::B256`) for the event.
/// - `data()`: Returns the ABI-encoded data payload (a `Vec<u8>`) for the non-indexed fields.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn event(args: TokenStream, input: TokenStream) -> TokenStream {
    sails_macros_core::event(args.into(), input.into()).into()
}

/// Marks a method as an override of a base service method.
///
/// This attribute is handled by the `#[service]` macro.
///
/// Arguments:
/// - First argument: Path to the redefined method meta (e.g. `crate::MyMeta`) OR `Interface, EntryId`.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn override_entry(_args: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    impl_item_fn_tokens
}
