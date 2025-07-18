use crate::prelude::*;

/// System call interface for accessing the runtime environment.
///
/// The `Syscall` struct provides a collection of methods that abstract lower-level operations,
/// such as retrieving message metadata (ID, size, source, value), fetching the program identifier,
/// obtaining the current block height, and accessing environment variables.
///
/// These methods are essential for enabling on-chain applications to interact with the Gear runtime
/// in a consistent manner. Depending on the target environment, different implementations are provided:
///
/// - For the WASM target, direct calls are made to `gstd::msg` and `gstd::exec` to fetch runtime data.
/// - In standard (`std`) environments, a mock implementation uses thread-local state for testing purposes.
/// - In `no_std` configurations without the `std` feature and not WASM target, the functions are marked as unimplemented.
///
/// Use these methods to retrieve contextual information about the current execution environment,
/// ensuring that your program logic remains agnostic of the underlying platform specifics.
pub struct Syscall;

#[cfg(target_arch = "wasm32")]
impl Syscall {
    pub fn message_id() -> MessageId {
        gstd::msg::id()
    }

    pub fn message_size() -> usize {
        gstd::msg::size()
    }

    pub fn message_source() -> ActorId {
        gstd::msg::source()
    }

    pub fn message_value() -> u128 {
        gstd::msg::value()
    }

    pub fn reply_to() -> Result<MessageId, gcore::errors::Error> {
        gstd::msg::reply_to()
    }

    pub fn reply_code() -> Result<ReplyCode, gcore::errors::Error> {
        gstd::msg::reply_code()
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn signal_from() -> Result<MessageId, gcore::errors::Error> {
        gstd::msg::signal_from()
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn signal_code() -> Result<Option<SignalCode>, gcore::errors::Error> {
        gstd::msg::signal_code()
    }

    pub fn program_id() -> ActorId {
        gstd::exec::program_id()
    }

    pub fn block_height() -> u32 {
        gstd::exec::block_height()
    }

    pub fn block_timestamp() -> u64 {
        gstd::exec::block_timestamp()
    }

    pub fn value_available() -> u128 {
        gstd::exec::value_available()
    }

    pub fn env_vars() -> gstd::EnvVars {
        gstd::exec::env_vars()
    }

    pub fn exit(inheritor_id: ActorId) -> ! {
        gstd::exec::exit(inheritor_id)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "std"))]
macro_rules! syscall_unimplemented {
    ($($name:ident() -> $type:ty),* $(,)?) => {
        impl Syscall {
            $(
                pub fn $name() -> $type {
                    unimplemented!("{ERROR}")
                }
            )*
        }
    };
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "std"))]
const ERROR: &str = "Syscall is implemented only for the wasm32 architecture and the std future";

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "std"))]
syscall_unimplemented!(
    message_id() -> MessageId,
    message_size() -> usize,
    message_source() -> ActorId,
    message_value() -> u128,
    reply_to() -> Result<MessageId, gcore::errors::Error>,
    reply_code() -> Result<ReplyCode, gcore::errors::Error>,
    signal_from() -> Result<MessageId, gcore::errors::Error>,
    signal_code() -> Result<Option<SignalCode>, gcore::errors::Error>,
    program_id() -> ActorId,
    block_height() -> u32,
    block_timestamp() -> u64,
    value_available() -> u128,
    env_vars() -> gstd::EnvVars,
);

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "std"))]
impl Syscall {
    pub fn exit(_inheritor_id: ActorId) -> ! {
        unimplemented!("{ERROR}")
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "std")]
const _: () = {
    use core::cell::RefCell;
    use paste::paste;
    use std::thread_local;

    macro_rules! syscall_struct_impl {
        ($($name:ident() -> $type:ty),* $(,)?) => {
            #[derive(Clone)]
            struct SyscallState {
                $(
                    $name: $type,
                )*
            }

            thread_local! {
                static SYSCALL_STATE: RefCell<SyscallState> = RefCell::new(SyscallState::default());
            }

            impl Syscall {
                $(
                    pub fn $name() -> $type {
                        SYSCALL_STATE.with_borrow(|state| state.$name.clone())
                    }
                )*
            }

            impl Syscall {
                $(
                    paste! {
                        pub fn [<with_ $name>]($name: $type) {
                            SYSCALL_STATE.with_borrow_mut(|state| state.$name = $name);
                        }
                    }
                )*
            }
        };
    }

    syscall_struct_impl!(
        message_id() -> MessageId,
        message_size() -> usize,
        message_source() -> ActorId,
        message_value() -> u128,
        reply_to() -> Result<MessageId, gcore::errors::Error>,
        reply_code() -> Result<ReplyCode, gcore::errors::Error>,
        signal_from() -> Result<MessageId, gcore::errors::Error>,
        signal_code() -> Result<Option<SignalCode>, gcore::errors::Error>,
        program_id() -> ActorId,
        block_height() -> u32,
        block_timestamp() -> u64,
        value_available() -> u128,
    );

    impl Default for SyscallState {
        fn default() -> Self {
            use gear_core_errors::{ExecutionError, ExtError};

            Self {
                message_id: MessageId::default(),
                message_size: 0,
                message_source: ActorId::default(),
                message_value: 0,
                reply_to: Err(ExtError::Execution(ExecutionError::NoReplyContext).into()),
                reply_code: Err(ExtError::Execution(ExecutionError::NoReplyContext).into()),
                signal_from: Err(ExtError::Execution(ExecutionError::NoSignalContext).into()),
                signal_code: Err(ExtError::Execution(ExecutionError::NoSignalContext).into()),
                program_id: ActorId::default(),
                block_height: 0,
                block_timestamp: 0,
                value_available: 0,
            }
        }
    }

    impl Syscall {
        pub fn env_vars() -> gstd::EnvVars {
            gstd::EnvVars {
                performance_multiplier: gstd::Percent::new(100),
                existential_deposit: 1_000_000_000_000,
                mailbox_threshold: 3000,
                gas_multiplier: gstd::GasMultiplier::from_value_per_gas(100),
            }
        }

        pub fn exit(inheritor_id: ActorId) -> ! {
            panic!("Program exited with inheritor id: {}", inheritor_id);
        }
    }
};
