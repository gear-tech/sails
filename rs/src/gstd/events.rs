//! Functionality for notifying off-chain subscribers on events happening in on-chain programs.

use super::utils::MaybeUninitBufferWriter;
use crate::{Encode, Output, Vec, collections::BTreeMap, errors::*};
use core::{any::TypeId, ops::DerefMut};
use scale_info::{StaticTypeInfo, TypeDef};

#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __emit_event<TEvents>(event: TEvents) -> Result<()>
where
    TEvents: parity_scale_codec::Encode + scale_info::StaticTypeInfo,
{
    let route = crate::gstd::services::exposure_context(gstd::msg::id()).route();
    __emit_event_with_route(route, event)
}

#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __emit_event_with_route<TEvents>(route: &[u8], event: TEvents) -> Result<()>
where
    TEvents: parity_scale_codec::Encode + scale_info::StaticTypeInfo,
{
    with_optimized_encode::<_, _>(route, event, |payload| {
        gstd::msg::send_bytes(gstd::ActorId::zero(), payload, 0)?;

        Ok::<_, Error>(())
    })?
}

#[allow(dead_code)]
fn with_optimized_encode<T, TEvents>(
    prefix: &[u8],
    event: TEvents,
    f: impl FnOnce(&[u8]) -> T,
) -> Result<T>
where
    TEvents: parity_scale_codec::Encode + scale_info::StaticTypeInfo,
{
    let mut type_id_to_encoded_event_names_map = type_id_to_event_names_map();
    let encoded_event_names = type_id_to_encoded_event_names_map
        .entry(TypeId::of::<TEvents::Identity>())
        .or_insert_with(extract_encoded_event_names::<TEvents>)
        .as_ref()
        .map_err(Clone::clone)?;

    // todo to be benchmarked
    let event_size = event.encoded_size();
    let res = gcore::stack_buffer::with_byte_buffer(event_size, |buffer| {
        let mut output = MaybeUninitBufferWriter::new(buffer);
        event.encode_to(&mut output);

        output
            .access_buffer(|event_bytes| {
                let event_idx = event_bytes[0] as usize;
                let encoded_event_name = &encoded_event_names[event_idx];
                let encoding_event_bytes = &event_bytes[1..];

                let final_payload_size =
                    prefix.len() + encoded_event_name.len() + encoding_event_bytes.len();
                gcore::stack_buffer::with_byte_buffer(final_payload_size, |buffer| {
                    let mut output = MaybeUninitBufferWriter::new(buffer);
                    output.write(prefix);
                    output.write(encoded_event_name);
                    output.write(encoding_event_bytes);

                    output
                        .access_buffer(f)
                        .expect("the output buffer is initialized previously")
                })
            })
            .expect("the output buffer is initialized when event is encoded")
    });

    Ok(res)
}

fn extract_encoded_event_names<TEvents>() -> Result<Vec<Vec<u8>>, RtlError>
where
    TEvents: StaticTypeInfo,
{
    let type_meta = scale_info::meta_type::<TEvents>();
    let type_info = type_meta.type_info();
    let variant_type_def = match type_info.type_def {
        TypeDef::Variant(variant_type_def) => variant_type_def,
        _ => {
            return Err(RtlError::EventTypeMustBeEnum {
                type_name: type_info.path.ident().unwrap_or("N/A").into(),
            });
        }
    };
    Ok(variant_type_def
        .variants
        .iter()
        .map(|variant| variant.name.encode())
        .collect())
}

type TypeIdToEncodedEventNamesMap = BTreeMap<TypeId, Result<Vec<Vec<u8>>, RtlError>>;

#[cfg(not(target_arch = "wasm32"))]
fn type_id_to_event_names_map() -> impl DerefMut<Target = TypeIdToEncodedEventNamesMap> {
    use spin::Mutex;

    static TYPE_ID_TO_EVENT_NAMES_MAP: Mutex<TypeIdToEncodedEventNamesMap> =
        Mutex::new(TypeIdToEncodedEventNamesMap::new());
    TYPE_ID_TO_EVENT_NAMES_MAP.lock()
}

// This code relies on the fact contracts are executed in a single-threaded environment
#[cfg(target_arch = "wasm32")]
fn type_id_to_event_names_map() -> impl DerefMut<Target = TypeIdToEncodedEventNamesMap> {
    // It is not expected this to be ever big as there are not that many event types in a contract.
    // So it shouldn't incur too many memory operations
    static mut TYPE_ID_TO_EVENT_NAMES_MAP: TypeIdToEncodedEventNamesMap =
        TypeIdToEncodedEventNamesMap::new();
    #[allow(static_mut_refs)]
    unsafe {
        &mut TYPE_ID_TO_EVENT_NAMES_MAP
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::ToString;
    use scale_info::TypeInfo;

    #[derive(Encode, TypeInfo)]
    enum TestEvents {
        Event1(u32),
        Event2 { p1: u16 },
    }

    #[derive(Encode, TypeInfo)]
    struct NotEnum;

    #[test]
    fn encoded_event_names_returns_proper_names_for_enum_type() {
        let encoded_event_names = extract_encoded_event_names::<TestEvents>().unwrap();

        assert_eq!(encoded_event_names.len(), 2);
        assert_eq!(encoded_event_names[0], "Event1".encode());
        assert_eq!(encoded_event_names[1], "Event2".encode());
    }

    #[test]
    fn encoded_event_names_returns_error_for_non_enum_type() {
        let result = extract_encoded_event_names::<NotEnum>();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "type `NotEnum` used as event type must be a enum"
        );
    }

    #[test]
    fn optimized_payload_encoding_returns_proper_payload() {
        let event = TestEvents::Event1(42);
        let res = with_optimized_encode::<_, _>(&[1, 2, 3], event, |payload| {
            assert_eq!(
                payload,
                [1, 2, 3, 24, 69, 118, 101, 110, 116, 49, 42, 00, 00, 00]
            );
        });
        assert!(res.is_ok());

        let event = TestEvents::Event2 { p1: 43 };
        let res = with_optimized_encode::<_, _>(&[], event, |payload| {
            assert_eq!(payload, [24, 69, 118, 101, 110, 116, 50, 43, 00]);
        });
        assert!(res.is_ok());
    }
}
