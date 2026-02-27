//! Functionality for notifying off-chain subscribers on events happening in on-chain programs.

use crate::{Encode, Output, utils::MaybeUninitBufferWriter};
use core::marker::PhantomData;
use gcore::stack_buffer;
use sails_idl_meta::InterfaceId;

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

    /// Returns `entry_id` for an event.
    fn entry_id(&self) -> u16;

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
    let encoded_size = Encode::encoded_size(&event);
    let skip_bytes = E::skip_bytes();
    let size = prefix.len() + encoded_size - skip_bytes;
    stack_buffer::with_byte_buffer(size, |buffer| {
        let mut buffer_writer = MaybeUninitBufferWriter::new(buffer);
        buffer_writer.write(prefix);
        buffer_writer.skip_next(skip_bytes); // skip the first byte, which is the index of the event enum variant
        Encode::encode_to(&event, &mut buffer_writer);
        buffer_writer.with_buffer(f)
    })
}

/// An event emitter that allows emitting events of type `T` for a specific route.
///
/// This is lightweight and can be cloned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventEmitter<E> {
    interface_id: InterfaceId,
    route_idx: u8,
    _marker: PhantomData<E>,
}

impl<E> EventEmitter<E> {
    pub fn new(interface_id: InterfaceId, route_idx: u8) -> Self {
        Self {
            interface_id,
            route_idx,
            _marker: PhantomData,
        }
    }
}

impl<E: SailsEvent> EventEmitter<E> {
    /// Emits an event.
    #[cfg(target_arch = "wasm32")]
    pub fn emit_event(&mut self, event: E) -> crate::errors::Result<()> {
        let header = crate::meta::SailsMessageHeader::v1(
            self.interface_id,
            event.entry_id(),
            self.route_idx,
        );
        with_optimized_event_encode(header.to_bytes().as_slice(), event, |payload| {
            gstd::msg::send_bytes(gstd::ActorId::zero(), payload, 0)?;
            Ok(())
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(not(feature = "std"))]
    pub fn emit_event(&mut self, _event: E) -> crate::errors::Result<()> {
        unimplemented!(
            "`emit_event` is implemented only for the wasm32 architecture and the std future"
        )
    }
}

#[cfg(feature = "ethexe")]
impl<T: super::EthEvent> EventEmitter<T> {
    /// Emits an event for the Ethexe program.
    #[cfg(target_arch = "wasm32")]
    pub fn emit_eth_event(&mut self, event: T) -> crate::errors::Result<()> {
        super::ethexe::__emit_eth_event(event)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(not(feature = "std"))]
    pub fn emit_eth_event(&mut self, _event: T) -> crate::errors::Result<()> {
        unimplemented!(
            "`emit_eth_event` is implemented only for the wasm32 architecture and the std future"
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "std")]
impl<T: 'static> EventEmitter<T> {
    /// Emits an event.
    pub fn emit_event(&mut self, event: T) -> crate::errors::Result<()> {
        event_registry::push_event(self.route_idx, event);
        Ok(())
    }

    #[cfg(feature = "ethexe")]
    pub fn emit_eth_event(&mut self, event: T) -> crate::errors::Result<()> {
        event_registry::push_event(self.route_idx, event);
        Ok(())
    }

    /// Takes the events emitted for this route and returns them as a `Vec<T>`.
    pub fn take_events(&mut self) -> crate::Vec<T> {
        event_registry::take_events(self.route_idx).unwrap_or_else(|| crate::Vec::new())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "std")]
mod event_registry {
    use core::any::{Any, TypeId};
    use std::{boxed::Box, collections::BTreeMap, sync::Mutex, vec::Vec};

    type Key = (u8, TypeId);

    std::thread_local! {
        /// thread-local registry mapping `(key, TypeId)` -> boxed `Vec<T>`
        static ROUTE_EVENTS: Mutex<BTreeMap<Key, Box<dyn Any>>> = Mutex::new(BTreeMap::new());
    }

    /// Push a `value: T` onto the `Vec<T>` stored under `key`.
    /// If none exists yet, we create a `Vec<T>` for that `(key,TypeId::of::<T>())`.
    pub(super) fn push_event<T: 'static>(key: u8, value: T) {
        ROUTE_EVENTS.with(|mtx| {
            let mut map = mtx.lock().expect("failed to lock ROUTE_EVENTS mutex");
            let slot = map
                .entry((key, TypeId::of::<T>()))
                .or_insert_with(|| Box::new(Vec::<T>::new()));
            // SAFETY: we just inserted a Box<Vec<T>> for exactly this TypeId,
            // so downcast_mut must succeed.
            let vec: &mut Vec<T> = slot
                .downcast_mut::<Vec<T>>()
                .expect("type mismatch in route-events registry");
            vec.push(value);
        });
    }

    /// Take `Vec<T>` for the given `key`, or `None` if nothing was ever pushed.
    pub(super) fn take_events<T: 'static>(key: u8) -> Option<Vec<T>> {
        ROUTE_EVENTS.with(|mtx| {
            let mut map = mtx.lock().expect("failed to lock ROUTE_EVENTS mutex");
            map.remove(&(key, TypeId::of::<T>())).map(|boxed| {
                // SAFETY: we just inserted a Box<Vec<T>> for exactly this TypeId,
                // so downcast must succeed.
                *boxed
                    .downcast::<Vec<T>>()
                    .expect("type mismatch in route-events registry")
            })
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::vec;

        #[test]
        fn event_registry() {
            push_event(1, 42_u32);
            push_event(1, 7_u32);

            assert_eq!(take_events::<u32>(1), Some(vec![42, 7]));
            assert!(take_events::<u32>(1).is_none()); // removed
            assert!(take_events::<i32>(1).is_none()); // wrong type
        }
    }
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

        fn entry_id(&self) -> u16 {
            match self {
                TestEvents::Event1(_) => 0,
                TestEvents::Event2 { .. } => 1,
            }
        }
    }

    #[test]
    fn trait_optimized_event_encode() {
        let event = TestEvents::Event1(42);
        assert_eq!(event.encode(), &[0, 42, 0, 0, 0]);

        with_optimized_event_encode(&[1, 2, 3], event, |payload| {
            assert_eq!(payload, [1, 2, 3, 42, 00, 00, 00]);
        });

        let event = TestEvents::Event2 { p1: 43 };
        assert_eq!(event.encode(), &[1, 43, 0]);
        with_optimized_event_encode(&[], event, |payload| {
            assert_eq!(payload, [43, 00]);
        });
    }
}
