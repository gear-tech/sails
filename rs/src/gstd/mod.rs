#[cfg(feature = "async-runtime")]
pub use async_runtime::{
    MessageFuture, create_program_for_reply, handle_reply_with_hook, message_loop,
    send_bytes_for_reply, send_for_reply, sleep_for,
};
#[cfg(feature = "async-runtime")]
pub type CreateProgramFuture = MessageFuture;
#[cfg(feature = "async-runtime")]
#[cfg(not(feature = "ethexe"))]
#[doc(hidden)]
pub use async_runtime::{handle_signal, set_critical_hook};
#[doc(hidden)]
#[cfg(feature = "ethexe")]
pub use ethexe::{EthEvent, EthEventExpo};
#[doc(hidden)]
pub use events::{EventEmitter, SailsEvent};
#[cfg(not(feature = "async-runtime"))]
#[cfg(not(feature = "ethexe"))]
#[doc(hidden)]
pub use gstd::handle_signal;
#[cfg(not(feature = "async-runtime"))]
#[cfg(not(feature = "ethexe"))]
#[doc(hidden)]
pub use gstd::msg::{CreateProgramFuture, MessageFuture};
pub use gstd::{debug, exec, msg};
#[doc(hidden)]
#[cfg(not(feature = "async-runtime"))]
pub use gstd::{handle_reply_with_hook, message_loop};
pub use locks::{Lock, WaitType};
#[doc(hidden)]
pub use sails_macros::{event, export, program, service};
pub use syscalls::Syscall;

use crate::{
    errors::{Error, Result, RtlError},
    prelude::{any::TypeId, *},
    utils::MaybeUninitBufferWriter,
};
use gcore::stack_buffer;

#[cfg(feature = "async-runtime")]
mod async_runtime;
#[cfg(feature = "ethexe")]
mod ethexe;
mod events;
mod locks;
pub mod services;
mod syscalls;

pub struct CommandReply<T>(T, ValueUnit);

impl<T> CommandReply<T> {
    pub fn new(result: T) -> Self {
        Self(result, 0)
    }

    pub fn with_value(self, value: ValueUnit) -> Self {
        Self(self.0, value)
    }

    pub fn to_tuple(self) -> (T, ValueUnit) {
        (self.0, self.1)
    }
}

impl<T> From<T> for CommandReply<T> {
    fn from(result: T) -> Self {
        Self(result, 0)
    }
}

impl<T> From<(T, ValueUnit)> for CommandReply<T> {
    fn from(value: (T, ValueUnit)) -> Self {
        Self(value.0, value.1)
    }
}

pub fn unknown_input_panic(message: &str, input: &[u8]) -> ! {
    let mut __input = input;
    match String::decode(&mut __input) {
        Ok(s) => panic!("{}: {}", message, s),
        Err(_) => panic!("{}: {}", message, HexSlice(input)),
    }
}

struct HexSlice<T: AsRef<[u8]>>(T);

impl<T: AsRef<[u8]>> core::fmt::Display for HexSlice<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let slice = self.0.as_ref();
        let len = slice.len();
        let precision = f.precision().unwrap_or(4);

        f.write_str("0x")?;
        if len <= precision * 2 {
            for byte in slice {
                write!(f, "{byte:02x}")?;
            }
        } else {
            for byte in &slice[..precision] {
                write!(f, "{byte:02x}")?;
            }
            f.write_str("..")?;
            for byte in &slice[len - precision..] {
                write!(f, "{byte:02x}")?;
            }
        }
        Ok(())
    }
}

impl<T: AsRef<[u8]>> core::fmt::Debug for HexSlice<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

pub trait InvocationIo {
    const ROUTE: &'static [u8];
    type Params: Decode;
    const ASYNC: bool;

    fn check_asyncness(payload: impl AsRef<[u8]>) -> Result<bool> {
        let value = payload.as_ref();
        if !value.starts_with(Self::ROUTE) {
            return Err(Error::Rtl(RtlError::InvocationPrefixMismatches));
        }

        Ok(Self::ASYNC)
    }

    fn decode_params(payload: impl AsRef<[u8]>) -> Result<Self::Params> {
        let mut value = payload.as_ref();
        if !value.starts_with(Self::ROUTE) {
            return Err(Error::Rtl(RtlError::InvocationPrefixMismatches));
        }
        value = &value[Self::ROUTE.len()..];
        Decode::decode(&mut value).map_err(Error::Codec)
    }

    fn with_optimized_encode<T: Encode, R>(
        value: &T,
        prefix: &[u8],
        f: impl FnOnce(&[u8]) -> R,
    ) -> R {
        let size = prefix.len() + Self::ROUTE.len() + Encode::encoded_size(value);
        stack_buffer::with_byte_buffer(size, |buffer| {
            let mut buffer_writer = MaybeUninitBufferWriter::new(buffer);

            buffer_writer.write(prefix);
            buffer_writer.write(Self::ROUTE);
            Encode::encode_to(value, &mut buffer_writer);

            buffer_writer.with_buffer(f)
        })
    }

    fn is_empty_tuple<T: 'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<()>()
    }
}

#[macro_export]
macro_rules! ok {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(err) => {
                return Err(err.into());
            }
        }
    };
}

#[cfg(not(feature = "async-runtime"))]
#[cfg(not(feature = "ethexe"))]
#[inline]
pub fn send_bytes_for_reply(
    destination: ActorId,
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
    gas_limit: Option<GasUnit>,
    reply_deposit: Option<GasUnit>,
    reply_hook: Option<Box<dyn FnOnce()>>,
) -> Result<MessageFuture, ::gstd::errors::Error> {
    let reply_deposit = reply_deposit.unwrap_or_default();
    // here can be a redirect target
    let mut message_future = if let Some(gas_limit) = gas_limit {
        ::gstd::msg::send_bytes_with_gas_for_reply(
            destination,
            payload,
            gas_limit,
            value,
            reply_deposit,
        )?
    } else {
        ::gstd::msg::send_bytes_for_reply(destination, payload, value, reply_deposit)?
    };

    message_future = match wait.wait_type() {
        WaitType::Exactly => message_future.exactly(wait.duration())?,
        WaitType::UpTo => message_future.up_to(wait.duration())?,
    };

    if let Some(reply_hook) = reply_hook {
        message_future = message_future.handle_reply(reply_hook)?;
    }
    Ok(message_future)
}

#[cfg(not(feature = "async-runtime"))]
#[cfg(feature = "ethexe")]
#[inline]
pub fn send_bytes_for_reply(
    destination: ActorId,
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
) -> Result<MessageFuture, ::gstd::errors::Error> {
    let value = params.value.unwrap_or(0);
    // here can be a redirect target
    let mut message_future = ::gstd::msg::send_bytes_for_reply(destination, payload, value)?;

    message_future = match wait.wait_type() {
        WaitType::Exactly => message_future.exactly(wait.duration())?,
        WaitType::UpTo => message_future.up_to(wait.duration())?,
    };

    Ok(message_future)
}

#[cfg(not(feature = "async-runtime"))]
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn create_program_for_reply(
    code_id: CodeId,
    salt: &[u8],
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
    gas_limit: Option<GasUnit>,
    reply_deposit: Option<GasUnit>,
    reply_hook: Option<Box<dyn FnOnce()>>,
) -> Result<(CreateProgramFuture, ActorId), ::gstd::errors::Error> {
    #[cfg(not(feature = "ethexe"))]
    let mut future = if let Some(gas_limit) = gas_limit {
        ::gstd::prog::create_program_bytes_with_gas_for_reply(
            code_id,
            salt,
            payload,
            gas_limit,
            value,
            reply_deposit.unwrap_or_default(),
        )?
    } else {
        ::gstd::prog::create_program_bytes_for_reply(
            code_id,
            salt,
            payload,
            value,
            reply_deposit.unwrap_or_default(),
        )?
    };
    #[cfg(feature = "ethexe")]
    let future = ::gstd::prog::create_program_bytes_for_reply(self.code_id, salt, payload, value)?;
    let program_id = future.program_id;

    future = match wait.wait_type() {
        WaitType::Exactly => future.exactly(wait.duration())?,
        WaitType::UpTo => future.up_to(wait.duration())?,
    };

    #[cfg(not(feature = "ethexe"))]
    if let Some(reply_hook) = reply_hook {
        future = future.handle_reply(reply_hook)?;
    }

    Ok((future, program_id))
}
