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
const ERROR: &str = "Syscall is implemented only for the wasm32 architecture and the std future";

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "std"))]
impl Syscall {
    pub fn message_id() -> MessageId {
        unimplemented!("{ERROR}")
    }

    pub fn message_size() -> usize {
        unimplemented!("{ERROR}")
    }

    pub fn message_source() -> ActorId {
        unimplemented!("{ERROR}")
    }

    pub fn message_value() -> u128 {
        unimplemented!("{ERROR}")
    }

    pub fn program_id() -> ActorId {
        unimplemented!("{ERROR}")
    }

    pub fn block_height() -> u32 {
        unimplemented!("{ERROR}")
    }

    pub fn block_timestamp() -> u64 {
        unimplemented!("{ERROR}")
    }

    pub fn value_available() -> u128 {
        unimplemented!("{ERROR}")
    }

    pub fn env_vars() -> gstd::EnvVars {
        unimplemented!("{ERROR}")
    }

    pub fn exit(_inheritor_id: ActorId) -> ! {
        unimplemented!("{ERROR}")
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "std")]
const _: () = {
    use core::cell::RefCell;
    use std::thread_local;

    #[derive(Default, Clone, Copy)]
    struct SyscallState {
        message_id: MessageId,
        message_size: usize,
        message_source: ActorId,
        message_value: u128,
        program_id: ActorId,
        block_height: u32,
        block_timestamp: u64,
        value_available: u128,
    }

    thread_local! {
        static SYSCALL_STATE: RefCell<SyscallState> = RefCell::new(SyscallState::default());
    }

    impl Syscall {
        pub fn message_id() -> MessageId {
            SYSCALL_STATE.with_borrow(|state| state.message_id)
        }

        pub fn message_size() -> usize {
            SYSCALL_STATE.with_borrow(|state| state.message_size)
        }

        pub fn message_source() -> ActorId {
            SYSCALL_STATE.with_borrow(|state| state.message_source)
        }

        pub fn message_value() -> u128 {
            SYSCALL_STATE.with_borrow(|state| state.message_value)
        }

        pub fn program_id() -> ActorId {
            SYSCALL_STATE.with_borrow(|state| state.program_id)
        }

        pub fn block_height() -> u32 {
            SYSCALL_STATE.with_borrow(|state| state.block_height)
        }

        pub fn block_timestamp() -> u64 {
            SYSCALL_STATE.with_borrow(|state| state.block_timestamp)
        }

        pub fn value_available() -> u128 {
            SYSCALL_STATE.with_borrow(|state| state.value_available)
        }

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

    impl Syscall {
        pub fn with_message_id(message_id: MessageId) {
            SYSCALL_STATE.with_borrow_mut(|state| state.message_id = message_id);
        }

        pub fn with_message_size(message_size: usize) {
            SYSCALL_STATE.with_borrow_mut(|state| state.message_size = message_size);
        }

        pub fn with_message_source(message_source: ActorId) {
            SYSCALL_STATE.with_borrow_mut(|state| state.message_source = message_source);
        }

        pub fn with_message_value(message_value: u128) {
            SYSCALL_STATE.with_borrow_mut(|state| state.message_value = message_value);
        }

        pub fn with_program_id(program_id: ActorId) {
            SYSCALL_STATE.with_borrow_mut(|state| state.program_id = program_id);
        }

        pub fn with_block_height(block_height: u32) {
            SYSCALL_STATE.with_borrow_mut(|state| state.block_height = block_height);
        }

        pub fn with_block_timestamp(block_timestamp: u64) {
            SYSCALL_STATE.with_borrow_mut(|state| state.block_timestamp = block_timestamp);
        }

        pub fn with_value_available(value_available: u128) {
            SYSCALL_STATE.with_borrow_mut(|state| state.value_available = value_available);
        }
    }
};
