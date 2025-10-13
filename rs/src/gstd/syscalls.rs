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
/// - For the WASM target, direct calls are made to `gcore::msg` and `gcore::exec` to fetch runtime data.
/// - In standard (`std`) environments, a mock implementation uses thread-local state for testing purposes.
/// - In `no_std` configurations without the `std` feature and not WASM target, the functions are marked as unimplemented.
///
/// Use these methods to retrieve contextual information about the current execution environment,
/// ensuring that your program logic remains agnostic of the underlying platform specifics.
pub struct Syscall;

#[cfg(target_arch = "wasm32")]
impl Syscall {
    #[inline(always)]
    pub fn message_id() -> MessageId {
        ::gcore::msg::id()
    }

    #[inline(always)]
    pub fn message_size() -> usize {
        ::gcore::msg::size()
    }

    #[inline(always)]
    pub fn message_source() -> ActorId {
        ::gcore::msg::source()
    }

    #[inline(always)]
    pub fn message_value() -> u128 {
        ::gcore::msg::value()
    }

    #[inline(always)]
    pub fn reply_to() -> Result<MessageId, gcore::errors::Error> {
        ::gcore::msg::reply_to()
    }

    #[inline(always)]
    pub fn reply_code() -> Result<ReplyCode, gcore::errors::Error> {
        ::gcore::msg::reply_code()
    }

    #[cfg(not(feature = "ethexe"))]
    #[inline(always)]
    pub fn signal_from() -> Result<MessageId, gcore::errors::Error> {
        ::gcore::msg::signal_from()
    }

    #[cfg(not(feature = "ethexe"))]
    #[inline(always)]
    pub fn signal_code() -> Result<Option<SignalCode>, gcore::errors::Error> {
        ::gcore::msg::signal_code()
    }

    #[inline(always)]
    pub fn program_id() -> ActorId {
        ::gcore::exec::program_id()
    }

    #[inline(always)]
    pub fn block_height() -> u32 {
        ::gcore::exec::block_height()
    }

    #[inline(always)]
    pub fn block_timestamp() -> u64 {
        ::gcore::exec::block_timestamp()
    }

    #[inline(always)]
    pub fn value_available() -> u128 {
        ::gcore::exec::value_available()
    }

    #[inline(always)]
    pub fn env_vars() -> ::gcore::EnvVars {
        ::gcore::exec::env_vars()
    }

    #[inline(always)]
    pub fn exit(inheritor_id: ActorId) -> ! {
        ::gcore::exec::exit(inheritor_id)
    }

    #[inline(always)]
    pub fn read_bytes() -> Result<Vec<u8>, ::gcore::errors::Error> {
        let mut result = vec![0u8; ::gcore::msg::size()];
        ::gcore::msg::read(result.as_mut())?;
        Ok(result)
    }

    #[cfg(not(feature = "ethexe"))]
    #[inline(always)]
    pub fn system_reserve_gas(amount: GasUnit) -> Result<(), ::gcore::errors::Error> {
        ::gcore::exec::system_reserve_gas(amount)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "std"))]
macro_rules! syscall_unimplemented {
    ($($name:ident(  $( $param:ident : $ty:ty ),* ) -> $type:ty),* $(,)?) => {
        impl Syscall {
            $(
                pub fn $name($( $param: $ty ),* ) -> $type {
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
    env_vars() -> ::gcore::EnvVars,
    exit(_inheritor_id: ActorId) -> !,
    read_bytes() -> Result<Vec<u8>, gcore::errors::Error>,
    system_reserve_gas(_amount: GasUnit) -> Result<(), ::gcore::errors::Error>,
);

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
        read_bytes() -> Result<Vec<u8>, gcore::errors::Error>,
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
                read_bytes: Err(::gcore::errors::Error::SyscallUsage),
            }
        }
    }

    impl Syscall {
        pub fn env_vars() -> ::gcore::EnvVars {
            ::gcore::EnvVars {
                performance_multiplier: gstd::Percent::new(100),
                existential_deposit: 1_000_000_000_000,
                mailbox_threshold: 3000,
                gas_multiplier: gstd::GasMultiplier::from_value_per_gas(100),
            }
        }

        pub fn exit(inheritor_id: ActorId) -> ! {
            panic!("Program exited with inheritor id: {}", inheritor_id);
        }

        #[cfg(not(feature = "ethexe"))]
        pub fn system_reserve_gas(_amount: GasUnit) -> Result<(), ::gcore::errors::Error> {
            Ok(())
        }
    }
};
