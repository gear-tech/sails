/// Computes the Sails hash for a function signature.
///
/// Supports both a low-level form with explicit kind and name expression, and a
/// shorthand form for `command` and `query` functions.
///
/// # Examples
///
/// ```rust,ignore
/// let hash = sails_rs::hash_fn!("command" "transfer", (u32, String) -> ());
/// let hash = sails_rs::hash_fn!(command transfer(u32, String) -> ());
/// let hash = sails_rs::hash_fn!(query balance_of(u32) -> u128);
/// ```
#[macro_export]
macro_rules! hash_fn {
    (@raw $kind:expr, $name:expr, ( $( $ty:ty ),* $(,)? ) -> $reply:ty $(| $throws:ty )?) => {{
        let mut fn_hash = $crate::keccak_const::Keccak256::new();
        fn_hash = fn_hash.update($kind.as_bytes()).update($name.as_bytes());
        $( fn_hash = fn_hash.update(&<$ty as $crate::sails_reflect_hash::ReflectHash>::HASH); )*
        fn_hash = fn_hash.update(b"res").update(&<$reply as $crate::sails_reflect_hash::ReflectHash>::HASH);
        $( fn_hash = fn_hash.update(b"throws").update(&<$throws as $crate::sails_reflect_hash::ReflectHash>::HASH); )?
        fn_hash.finalize()
    }};

    (
        $kind:literal $name:expr, ( $( $ty:ty ),* $(,)? ) -> $reply:ty $(| $throws:ty )?
    ) => {
        $crate::hash_fn!(@raw $kind, $name, ( $( $ty ),* ) -> $reply $(| $throws )?)
    };

    (
        command $name:ident ( $( $ty:ty ),* $(,)? ) -> $reply:ty $(| $throws:ty )?
    ) => {
        $crate::hash_fn!(@raw "command", stringify!($name), ( $( $ty ),* ) -> $reply $(| $throws )?)
    };

    (
        query $name:ident ( $( $ty:ty ),* $(,)? ) -> $reply:ty $(| $throws:ty )?
    ) => {
        $crate::hash_fn!(@raw "query", stringify!($name), ( $( $ty ),* ) -> $reply $(| $throws )?)
    };
}

/// Evaluates a program constructor call and stores the resulting program in a
/// mutable static slot.
///
/// Supports plain constructor calls plus `.await`, `.unwrap()`, and
/// `.await.unwrap()`.
///
/// The constructor arguments are expected to already be bound in the local
/// scope. When `.unwrap()` is used, `Result` errors are converted into a
/// structured panic through [`ok_or_throws!`].
///
/// # Examples
///
/// ```rust,ignore
/// program_ctor!(PROGRAM = MyProgram::new(p1, p2).await);
/// program_ctor!(PROGRAM = MyProgram::new_result(p1, p2).await.unwrap());
/// ```
#[macro_export]
macro_rules! program_ctor {
    ($prg:ident = $($call:ident)::+ ( $( $arg:ident ),* $(,)? ) $($tail:tt)*) => {{
        $crate::program_ctor!(@inner $prg = [$($call)::+] ( $( $arg ),* ) $($tail)*);
    }};
    (@store $prg:ident = $program:expr) => {{
        unsafe {
            $prg = Some($program);
        }
    }};
    (@throws [$handler:ident] = $call:expr) => {{
        $crate::paste::paste! {
            $crate::ok_or_throws!($call, meta_in_program::[<__ $handler:camel Params>], 0)
        }
    }};
    (@throws [$head:ident :: $($tail:tt)+] = $call:expr) => {{
        $crate::program_ctor!(@throws [$($tail)+] = $call)
    }};
    (@inner $prg:ident = [$($call:tt)+] ( $( $arg:ident ),* ) .await .unwrap()) => {{
        $crate::gstd::message_loop(async move {
            $crate::program_ctor!(@store $prg = $crate::program_ctor!(@throws [$($call)+] = $($call)+ ( $( $arg, )* ).await));
        });
    }};
    (@inner $prg:ident = [$($call:tt)+] ( $( $arg:ident ),* ) .await) => {{
        $crate::gstd::message_loop(async move {
            $crate::program_ctor!(@store $prg = $($call)+ ( $( $arg, )* ).await);
        });
    }};
    (@inner $prg:ident = [$($call:tt)+] ( $( $arg:ident ),* ) .unwrap()) => {{
        $crate::program_ctor!(@store $prg = $crate::program_ctor!(@throws [$($call)+] = $($call)+ ( $( $arg, )* )));
    }};
    (@inner $prg:ident = [$($call:tt)+] ( $( $arg:ident ),* )) => {{
        $crate::program_ctor!(@store $prg = $($call)+ ( $( $arg, )* ));
    }};
}

/// Declares an invocation params struct together with its metadata and
/// [`crate::gstd::InvocationIo`] implementation.
///
/// The generated struct always derives [`crate::Decode`] and
/// [`crate::TypeInfo`], using `$crate::scale_codec` and `$crate::scale_info`
/// as the derive crate paths.
///
/// # Examples
///
/// ```rust,ignore
/// sails_rs::invocation_io!(
///     pub struct __FooParams {
///         pub(super) a: u32,
///         pub(super) b: String,
///     }
///     entry_id = 0,
/// );
/// ```
#[macro_export]
macro_rules! invocation_io {
    (
        $struct_vis:vis struct $params_struct:ident {
            $( $field_vis:vis $field:ident : $ty:ty ),* $(,)?
        },
        entry_id = $entry_id:expr $(,)?
    ) => {
        $crate::invocation_io!(
            $struct_vis struct $params_struct {
                $( $field_vis $field : $ty, )*
            },
            interface_id = $crate::meta::InterfaceId::zero(),
            entry_id = $entry_id,
        );
    };

    (
        $struct_vis:vis struct $params_struct:ident {
            $( $field_vis:vis $field:ident : $ty:ty ),* $(,)?
        },
        interface_id = $interface_id:expr,
        entry_id = $entry_id:expr $(,)?
    ) => {
        #[derive($crate::Decode, $crate::TypeInfo)]
        #[codec(crate = $crate::scale_codec)]
        #[scale_info(crate = $crate::scale_info)]
        $struct_vis struct $params_struct {
            $( $field_vis $field: $ty, )*
        }

        impl $crate::meta::Identifiable for $params_struct {
            const INTERFACE_ID: $crate::meta::InterfaceId = $interface_id;
        }

        impl $crate::meta::MethodMeta for $params_struct {
            const ENTRY_ID: u16 = $entry_id;
        }

        impl $crate::gstd::InvocationIo for $params_struct {
            type Params = Self;
        }
    };
}

/// Dispatches a service exposure, selecting the async or sync handling path at
/// runtime and replying with the encoded result.
///
/// The service exposure value must already be bound in the local scope.
#[macro_export]
macro_rules! service_route_dispatch {
    (
        $svc:ident : $service_ty:ty,
        interface_id = $interface_id:expr,
        entry_id = $entry_id:expr,
        input = $input:expr $(,)?
    ) => {{
        let is_async =
            <$service_ty as $crate::gstd::services::Service>::Exposure::check_asyncness(
                $interface_id,
                $entry_id,
            )
            .unwrap_or_else(|| $crate::gstd::unknown_input_panic("Unknown call", &[]));

        if is_async {
            $crate::gstd::message_loop(async move {
                $svc.try_handle_async($interface_id, $entry_id, $input, |encoded_result, value| {
                    $crate::gstd::msg::reply_bytes(encoded_result, value)
                        .expect("Failed to send output");
                })
                .await
                .unwrap_or_else(|| $crate::gstd::unknown_input_panic("Unknown request", &[]));
            });
        } else {
            $svc.try_handle($interface_id, $entry_id, $input, |encoded_result, value| {
                $crate::gstd::msg::reply_bytes(encoded_result, value)
                    .expect("Failed to send output");
            })
            .unwrap_or_else(|| $crate::gstd::unknown_input_panic("Unknown request", &[]));
        }
    }};
}

/// Unwraps a `Result` or converts its error into a structured panic payload.
///
/// The payload is encoded with the provided invocation params type and route
/// index, then sent through [`crate::gstd::Syscall::panic`] when it fits within
/// [`crate::gstd::MAX_PANIC_PAYLOAD_SIZE`].
#[macro_export]
macro_rules! ok_or_throws {
    ($res: expr, $param: ty, $route_idx: expr) => {
        match $res {
            Ok(r) => r,
            Err(e) => {
                let encoded = <$param as $crate::gstd::InvocationIo>::with_optimized_encode(
                    &e,
                    $route_idx,
                    |encoded| encoded.to_vec(),
                );
                if encoded.len() <= $crate::gstd::MAX_PANIC_PAYLOAD_SIZE {
                    $crate::gstd::Syscall::panic(&encoded)
                } else {
                    ::core::panic!("Error payload is too large to panic")
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[allow(dead_code)]
    struct MyProgram {
        p1: u32,
        p2: String,
    }

    static mut PROGRAM: Option<MyProgram> = None;

    impl MyProgram {
        pub async fn new(p1: u32, p2: String) -> Self {
            Self { p1, p2 }
        }

        pub async fn new_result(p1: u32, p2: String) -> Result<Self, String> {
            Ok(Self { p1, p2 })
        }
    }

    mod meta_in_program {
        invocation_io!(pub struct __NewResultParams {}, entry_id = 0,);
    }

    #[test]
    fn program_ctor_async() {
        let p1 = 42_u32;
        let p2 = String::from("payload");

        let _compile = || {
            program_ctor!(PROGRAM = MyProgram::new(p1, p2).await);
        };
    }

    #[test]
    fn program_ctor_async_unwrap_result() {
        let p1 = 42_u32;
        let p2 = String::from("payload");

        let _compile = || {
            program_ctor!(PROGRAM = MyProgram::new_result(p1, p2).await.unwrap());
        };
    }

    #[test]
    fn invocation_io_macro_compiles() {
        invocation_io!(
            pub struct __FooParams {
                pub(super) a: u32,
                pub(super) b: String,
            },
            interface_id = crate::meta::InterfaceId::zero(),
            entry_id = 7,
        );

        let _params: <__FooParams as crate::gstd::InvocationIo>::Params = __FooParams {
            a: 1,
            b: String::from("ok"),
        };
        let _ = (_params.a, &_params.b);

        assert_eq!(<__FooParams as crate::meta::MethodMeta>::ENTRY_ID, 7);
    }

    #[test]
    fn invocation_io_macro_defaults_interface_id_to_zero() {
        invocation_io!(
            pub struct __BarParams {
                pub(super) a: u32,
            },
            entry_id = 3,
        );

        let params = __BarParams { a: 1 };
        let _ = params.a;

        assert_eq!(
            <__BarParams as crate::meta::Identifiable>::INTERFACE_ID,
            crate::meta::InterfaceId::zero()
        );
        assert_eq!(<__BarParams as crate::meta::MethodMeta>::ENTRY_ID, 3);
    }

    #[test]
    fn service_route_dispatch_macro_compiles() {
        struct DummyService;
        struct DummyExposure;

        impl crate::gstd::services::Service for DummyService {
            type Exposure = DummyExposure;

            fn expose(self, _route_idx: u8) -> Self::Exposure {
                DummyExposure
            }
        }

        impl crate::gstd::services::Exposure for DummyExposure {
            fn interface_id() -> crate::meta::InterfaceId {
                crate::meta::InterfaceId::zero()
            }

            fn route_idx(&self) -> u8 {
                0
            }

            fn check_asyncness(
                _interface_id: crate::meta::InterfaceId,
                _entry_id: u16,
            ) -> Option<bool> {
                Some(false)
            }
        }

        impl DummyExposure {
            async fn try_handle_async(
                self,
                _interface_id: crate::meta::InterfaceId,
                _entry_id: u16,
                _input: &[u8],
                _result_handler: impl FnOnce(&[u8], u128),
            ) -> Option<()> {
                Some(())
            }

            fn try_handle(
                self,
                _interface_id: crate::meta::InterfaceId,
                _entry_id: u16,
                _input: &[u8],
                _result_handler: impl FnOnce(&[u8], u128),
            ) -> Option<()> {
                Some(())
            }
        }

        let svc = DummyExposure;
        let interface_id = crate::meta::InterfaceId::zero();
        let entry_id = 0u16;
        let input: &[u8] = &[];

        let _compile = || {
            service_route_dispatch!(
                svc: DummyService,
                interface_id = interface_id,
                entry_id = entry_id,
                input = input,
            );
        };
    }
}
