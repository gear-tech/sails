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
        pub struct __NewResultParams;

        impl crate::meta::Identifiable for __NewResultParams {
            const INTERFACE_ID: crate::meta::InterfaceId = crate::meta::InterfaceId::zero();
        }

        impl crate::meta::MethodMeta for __NewResultParams {
            const ENTRY_ID: u16 = 0;
        }

        impl crate::gstd::InvocationIo for __NewResultParams {
            type Params = (u32, crate::String);
        }
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
}
