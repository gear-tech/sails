use crate::{
    collections::HashMap,
    errors::*,
    gstd::{msg, services},
    Vec,
};
use core::{any::TypeId, marker::PhantomData};
use gstd::ActorId as GStdActorId;
use parity_scale_codec::Encode;
use scale_info::{StaticTypeInfo, TypeDef};

pub trait EventTrigger<TEvents> {
    fn trigger(&self, event: TEvents) -> Result<()>;
}

#[derive(Default)]
pub struct GStdEventTrigger<TEvents> {
    _tevents: PhantomData<TEvents>,
}

impl<TEvents> GStdEventTrigger<TEvents>
where
    TEvents: StaticTypeInfo + Encode,
{
    pub fn new() -> Self {
        Self {
            _tevents: PhantomData,
        }
    }

    // This code relies on the fact contracts are executed in a single-threaded environment
    fn type_id_to_event_names_map() -> &'static mut HashMap<TypeId, Result<Vec<Vec<u8>>, RtlError>>
    {
        type TypeIdToEncodedEventNamesMap = HashMap<TypeId, Result<Vec<Vec<u8>>, RtlError>>;
        // It is not expected this to be ever big as there are not that many event types in a contract.
        // So it shouldn't incur too many memory operations
        static mut TYPE_ID_TO_EVENT_NAMES_MAP: Option<TypeIdToEncodedEventNamesMap> = None;
        unsafe { TYPE_ID_TO_EVENT_NAMES_MAP.get_or_insert_with(HashMap::new) }
    }

    fn encoded_event_names() -> Result<&'static [Vec<u8>], RtlError>
    where
        TEvents: StaticTypeInfo,
    {
        let type_id_to_encoded_event_names_map = Self::type_id_to_event_names_map();

        let encoded_event_names = type_id_to_encoded_event_names_map
            .entry(TypeId::of::<TEvents::Identity>())
            .or_insert_with(|| {
                let type_meta = scale_info::meta_type::<TEvents>();
                let type_info = type_meta.type_info();
                let variant_type_def = match type_info.type_def {
                    TypeDef::Variant(variant_type_def) => variant_type_def,
                    _ => {
                        return Err(RtlError::EventTypeMustBeEnum {
                            type_name: type_info.path.ident().unwrap_or("N/A").into(),
                        })
                    }
                };
                Ok(variant_type_def
                    .variants
                    .iter()
                    .map(|variant| variant.name.encode())
                    .collect())
            });
        encoded_event_names
            .as_ref()
            .map_err(Clone::clone)
            .map(|v| v.as_slice())
    }

    fn compose_payload(prefix: &[u8], event: TEvents) -> Result<Vec<u8>, RtlError> {
        let encoded_event_names = Self::encoded_event_names()?;
        let payload = event.encode();
        let event_idx = payload[0]; // It is safe to get this w/o any check as we know the type is a proper event type, i.e. enum
        let encoded_event_name = &encoded_event_names[event_idx as usize];
        Ok([prefix, &encoded_event_name[..], &payload[1..]].concat())
    }
}

impl<TEvents> EventTrigger<TEvents> for GStdEventTrigger<TEvents>
where
    TEvents: Encode + StaticTypeInfo,
{
    fn trigger(&self, event: TEvents) -> Result<()> {
        let payload =
            Self::compose_payload(services::exposure_context(msg::id().into()).route(), event)?;
        msg::send_bytes(GStdActorId::zero(), payload, 0)?;
        Ok(())
    }
}

pub mod mocks {
    use super::*;

    #[derive(Default)]
    pub struct MockEventTrigger<TEvents> {
        _tevents: PhantomData<TEvents>,
    }

    impl<TEvents> MockEventTrigger<TEvents> {
        pub fn new() -> Self {
            Self {
                _tevents: PhantomData,
            }
        }
    }

    impl<TEvents> EventTrigger<TEvents> for MockEventTrigger<TEvents>
    where
        TEvents: Encode + StaticTypeInfo,
    {
        fn trigger(&self, event: TEvents) -> Result<()> {
            GStdEventTrigger::<TEvents>::compose_payload(&[], event)?;
            Ok(())
        }
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
        let encoded_event_names = GStdEventTrigger::<TestEvents>::encoded_event_names().unwrap();

        assert_eq!(encoded_event_names.len(), 2);
        assert_eq!(encoded_event_names[0], "Event1".encode());
        assert_eq!(encoded_event_names[1], "Event2".encode());
    }

    #[test]
    fn encoded_event_names_returns_error_for_non_enum_type() {
        let result = GStdEventTrigger::<NotEnum>::encoded_event_names();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "type `NotEnum` used as event type must be a enum"
        );
    }

    #[test]
    fn compose_payload_returns_proper_payload() {
        let event = TestEvents::Event1(42);
        let payload = GStdEventTrigger::<TestEvents>::compose_payload(&[1, 2, 3], event).unwrap();

        assert_eq!(
            payload,
            [1, 2, 3, 24, 69, 118, 101, 110, 116, 49, 42, 00, 00, 00]
        );

        let event = TestEvents::Event2 { p1: 43 };
        let payload = GStdEventTrigger::<TestEvents>::compose_payload(&[], event).unwrap();

        assert_eq!(payload, [24, 69, 118, 101, 110, 116, 50, 43, 00]);
    }
}
