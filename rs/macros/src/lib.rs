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
///             `sails-rs` create is re-exprted from another crate.
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
///     use sails_rs::service;
///
///     #[derive(parity_scale_codec::Encode, scale_info::TypeInfo)]
///     enum MyServiceEvents {
///         SomethingDone,
///     }
///
///     pub struct MyService;
///
///     #[service(events = MyServiceEvents)]
///     impl MyService {
///         pub fn do_something(&mut self) -> u32 {
///             self.notify_on(MyServiceEvents::SomethingDone).unwrap();
///             0
///         }
///
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
///             `sails-rs` create is re-exprted from another crate.
/// - `handle_reply` - specifies a path to a function that will be called
///                    after standrd reply handling provided by the `gstd` crate.
/// - `handle_signal` - specifies a path to a function that will be called
///                     after standard signal handling provided by the `gstd` crate.
///
/// # Examples
///
/// ```rust
/// mod my_program {
///     use sails_rs::program;
///
///     pub struct MyProgram;
///
///     #[program(handle_reply = inspect_reply)]
///     impl MyProgram {
///         pub fn default() -> Self {
///             Self
///         }
///
///         pub fn from_seed(_seed: u32) -> Self {
///             Self
///         }
///     }
///
///     fn inspect_reply() {
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn program(args: TokenStream, impl_tokens: TokenStream) -> TokenStream {
    sails_macros_core::gprogram(args.into(), impl_tokens.into()).into()
}

/// Changes default route to services exposed by Sails program.
///
/// By default, every exposed service is available via a route that is PascalCase-ed name
/// of the method used for exposing the service. This macro allows to customize the route
/// so it will be a PascalCase-ed string specified in the attribute.
///
/// This attribute can also be applied to methods exposed by Sails services.
///
/// # Examples
///
/// ```rust
/// mod my_program {
///    use sails_rs::{program, route, service};
///
///    struct MyService;
///
///    #[service]
///    impl MyService {
///        pub fn do_something(&mut self) -> u32 {
///            0
///        }
///    }
///
///    pub struct MyProgram;
///
///    #[program]
///    impl MyProgram {
///         // Exposed as `MyService`
///         pub fn my_service(&self) -> MyService {
///             MyService
///         }
///
///         // Exposed as `Worker`
///         #[route("worker")]
///         pub fn my_worker(&self) -> MyService {
///             MyService
///         }
///    }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn route(args: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    sails_macros_core::groute(args.into(), impl_item_fn_tokens.into()).into()
}
