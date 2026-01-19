#[doc(hidden)]
#[cfg(feature = "ethexe")]
pub use ethexe::{EthEvent, EthEventExpo};
#[doc(hidden)]
pub use events::{EventEmitter, SailsEvent};
#[cfg(not(feature = "ethexe"))]
#[doc(hidden)]
pub use gstd::handle_signal;
#[doc(hidden)]
pub use gstd::{async_init, async_main, handle_reply_with_hook, message_loop};
pub use gstd::{debug, exec, msg};
use sails_idl_meta::{Identifiable, InterfaceId, MethodMeta};
#[doc(hidden)]
pub use sails_macros::{event, export, program, service};
pub use syscalls::Syscall;

use crate::{
    errors::{Error, Result, RtlError},
    meta::SailsMessageHeader,
    prelude::{any::TypeId, *},
    utils::MaybeUninitBufferWriter,
};
use gcore::stack_buffer;

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

pub struct HexSlice<T: AsRef<[u8]>>(pub T);

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

pub trait InvocationIo: Identifiable + MethodMeta {
    type Params: Decode;

    fn decode_params(payload: impl AsRef<[u8]>) -> Result<Self::Params> {
        let mut value = payload.as_ref();
        let header: SailsMessageHeader = Decode::decode(&mut value).map_err(Error::Codec)?;
        if header.interface_id() != Self::INTERFACE_ID {
            return Err(Error::Rtl(RtlError::InvocationPrefixMismatches));
        }
        if header.entry_id() != Self::ENTRY_ID {
            return Err(Error::Rtl(RtlError::InvocationPrefixMismatches));
        }
        let value: Self::Params = Decode::decode(&mut value).map_err(Error::Codec)?;
        Ok(value)
    }

    fn with_optimized_encode<T: Encode, R>(
        value: &T,
        route_idx: u8,
        f: impl FnOnce(&[u8]) -> R,
    ) -> R {
        Self::with_optimized_encode_with_id(Self::INTERFACE_ID, Self::ENTRY_ID, value, route_idx, f)
    }

    fn with_optimized_encode_with_id<T: Encode, R>(
        interface_id: InterfaceId,
        entry_id: u16,
        value: &T,
        route_idx: u8,
        f: impl FnOnce(&[u8]) -> R,
    ) -> R {
        let header = SailsMessageHeader::v1(interface_id, entry_id, route_idx);
        let size = 16 + Encode::encoded_size(value);
        stack_buffer::with_byte_buffer(size, |buffer| {
            let mut buffer_writer = MaybeUninitBufferWriter::new(buffer);
            Encode::encode_to(&header, &mut buffer_writer);
            Encode::encode_to(value, &mut buffer_writer);
            buffer_writer.with_buffer(f)
        })
    }
}

pub fn with_optimized_encode<T: Encode, R>(
    value: &T,
    // prefix: &[u8],
    f: impl FnOnce(&[u8]) -> R,
) -> R {
    let size = Encode::encoded_size(value);
    stack_buffer::with_byte_buffer(size, |buffer| {
        let mut buffer_writer = MaybeUninitBufferWriter::new(buffer);
        Encode::encode_to(value, &mut buffer_writer);
        buffer_writer.with_buffer(f)
    })
}

pub fn is_empty_tuple<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<()>()
}
