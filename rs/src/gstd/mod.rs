#[doc(hidden)]
// pub use gstd::{async_init, async_main, handle_reply_with_hook, message_loop};
pub use async_runtime::{
    SimpleMessageFuture, handle_reply_with_hook, message_loop, send_bytes_for_reply,
};
#[doc(hidden)]
#[cfg(feature = "ethexe")]
pub use ethexe::{EthEvent, EthEventExpo};
#[doc(hidden)]
pub use events::{EventEmitter, SailsEvent};
#[cfg(not(feature = "ethexe"))]
#[doc(hidden)]
pub use gstd::handle_signal;
pub use gstd::{debug, exec, msg};
#[doc(hidden)]
pub use sails_macros::{event, export, program, service};
pub use syscalls::Syscall;

use crate::{
    errors::{Error, Result, RtlError},
    prelude::{any::TypeId, *},
    utils::MaybeUninitBufferWriter,
};
use gcore::stack_buffer;

pub(crate) mod async_runtime;
#[cfg(feature = "ethexe")]
mod ethexe;
mod events;
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
