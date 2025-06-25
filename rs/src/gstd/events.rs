//! Functionality for notifying off-chain subscribers on events happening in on-chain programs.

use super::utils::MaybeUninitBufferWriter;
use crate::{Encode, Output};
use gcore::stack_buffer;

/// Trait for encoding events that can be emitted by Sails programs.
///
/// This trait is used to define events that can be emitted by Sails programs, allowing them to be
/// encoded into a byte slice.
///
/// The `SailsEvent` trait extends the `Encode` trait, which means that any type implementing `SailsEvent`
/// must also implement the `Encode` trait.
///
/// The `skip_bytes` method is used to determine how many bytes should be skipped when encoding the event,
/// which is particularly relevant for enum variants where the first byte is reserved for the index of the variant.
///
/// /// # Examples
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
///         sender: uint128,
///         amount: uint128,
///         note: String,
///     },
/// }
/// ```
pub trait SailsEvent: Encode {
    /// Returns the encoded event name as a byte slice.
    fn encoded_event_name(&self) -> &'static [u8];

    /// The number of bytes to skip when encoding the event.
    ///
    /// For enums, this is always 1 byte, which is reserved for the index of the event enum variant.
    fn skip_bytes() -> usize {
        1
    }
}

#[allow(dead_code)]
fn with_optimized_event_encode<T, E: SailsEvent, F: FnOnce(&[u8]) -> T>(
    prefix: &[u8],
    event: E,
    f: F,
) -> T {
    let encoded_event_name = E::encoded_event_name(&event);
    let encoded_size = Encode::encoded_size(&event);
    let skip_bytes = E::skip_bytes();
    let size = prefix.len() + encoded_event_name.len() + encoded_size - skip_bytes;
    stack_buffer::with_byte_buffer(size, |buffer| {
        let mut buffer_writer = MaybeUninitBufferWriter::new(buffer);
        buffer_writer.write(prefix);
        buffer_writer.write(encoded_event_name);
        buffer_writer.skip_next(skip_bytes); // skip the first byte, which is the index of the event enum variant
        Encode::encode_to(&event, &mut buffer_writer);
        buffer_writer.with_buffer(f)
    })
}

#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __emit_event_with_route<TEvents>(route: &[u8], event: TEvents) -> crate::errors::Result<()>
where
    TEvents: SailsEvent,
{
    with_optimized_event_encode(route, event, |payload| {
        gstd::msg::send_bytes(gstd::ActorId::zero(), payload, 0)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[derive(Encode, TypeInfo)]
    enum TestEvents {
        Event1(u32),
        Event2 { p1: u16 },
    }

    impl SailsEvent for TestEvents {
        fn encoded_event_name(&self) -> &'static [u8] {
            match self {
                TestEvents::Event1(_) => &[24, 69, 118, 101, 110, 116, 49],
                TestEvents::Event2 { .. } => &[24, 69, 118, 101, 110, 116, 50],
            }
        }
    }

    #[test]
    fn trait_optimized_event_encode() {
        let event = TestEvents::Event1(42);
        with_optimized_event_encode(&[1, 2, 3], event, |payload| {
            assert_eq!(
                payload,
                [1, 2, 3, 24, 69, 118, 101, 110, 116, 49, 42, 00, 00, 00]
            );
        });

        let event = TestEvents::Event2 { p1: 43 };
        with_optimized_event_encode(&[], event, |payload| {
            assert_eq!(payload, [24, 69, 118, 101, 110, 116, 50, 43, 00]);
        });
    }
}
