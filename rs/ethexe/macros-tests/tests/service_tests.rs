use sails_rs::{
    Encode, MessageId, alloy_primitives::B256, alloy_sol_types::SolValue, gstd::services::Service,
};

mod service_with_basics;
mod service_with_events;
mod service_with_events_and_lifetimes;
mod service_with_export_unwrap_result;
mod service_with_extends;
mod service_with_extends_and_lifetimes;
mod service_with_lifecycles_and_generics;
mod service_with_reply_with_value;
mod service_with_trait_bounds;

#[tokio::test]
async fn service_with_basics() {
    use service_with_basics::MyService;

    const DO_THIS: &str = "DoThis";
    let input = (0u128, false, 42u32, "correct".to_owned()).abi_encode_sequence();

    let mut exposure = MyService.expose(MessageId::from(123), &[1, 2, 3]);

    assert!(
        exposure
            .try_handle_solidity(&DO_THIS.encode(), &input)
            .is_none()
    );

    // Check asyncness for `DoThis`.
    assert!(exposure.check_asyncness(&DO_THIS.encode()).unwrap());

    // act
    let (output, ..) = exposure
        .try_handle_solidity_async(&DO_THIS.encode(), &input)
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn service_with_basics_with_encode_reply() {
    use service_with_basics::MyService;

    const DO_THIS: &str = "DoThis";
    let input = (0u128, true, 42u32, "correct".to_owned()).abi_encode_sequence();
    let message_id = MessageId::from(123);

    // act
    let (output, ..) = MyService
        .expose(message_id, &[1, 2, 3])
        .try_handle_solidity_async(&DO_THIS.encode(), &input)
        .await
        .unwrap();

    let (mid, result): (B256, String) =
        sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false).unwrap();
    assert_eq!(message_id, MessageId::new(mid.0));
    assert_eq!("42: correct", result.as_str());
}

#[test]
fn service_with_events() {
    use service_with_events::{MyEvents, MyServiceWithEvents};

    let mut exposure = MyServiceWithEvents(0).expose(MessageId::from(142), &[1, 4, 2]);
    exposure.my_method();

    let events = exposure.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[test]
fn service_with_lifetimes_and_events() {
    use service_with_events_and_lifetimes::{MyEvents, MyGenericEventsService};

    const DO_THIS: &str = "DoThis";
    let input = (0u128, false).abi_encode_sequence();

    let my_service = MyGenericEventsService::<'_, String>::default();
    let mut exposure = my_service.expose(MessageId::from(123), &[1, 2, 3]);

    assert!(!exposure.check_asyncness(&DO_THIS.encode()).unwrap());

    let (output, ..) = exposure
        .try_handle_solidity(&DO_THIS.encode(), input.as_slice())
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(42), result);

    let events = exposure.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[test]
fn service_with_extends() {
    use service_with_extends::{
        base::{BASE_NAME_RESULT, Base},
        extended::{EXTENDED_NAME_RESULT, Extended, NAME_RESULT},
    };

    const NAME_METHOD: &str = "Name";
    const BASE_NAME_METHOD: &str = "BaseName";
    const EXTENDED_NAME_METHOD: &str = "ExtendedName";
    let input = (0u128, false).abi_encode_sequence();

    let mut extended_svc = Extended::new(Base).expose(123.into(), &[1, 2, 3]);

    assert!(
        !extended_svc
            .check_asyncness(&EXTENDED_NAME_METHOD.encode())
            .unwrap()
    );

    let (output, ..) = extended_svc
        .try_handle_solidity(&EXTENDED_NAME_METHOD.encode(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(EXTENDED_NAME_RESULT.to_owned()), result);

    assert!(
        !extended_svc
            .check_asyncness(&BASE_NAME_METHOD.encode())
            .unwrap()
    );
    let _base: &<Base as Service>::Exposure = extended_svc.as_base_0();

    let (output, ..) = extended_svc
        .try_handle_solidity(&BASE_NAME_METHOD.encode(), &input)
        .unwrap();
    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(BASE_NAME_RESULT.to_owned()), result);

    let (output, ..) = extended_svc
        .try_handle_solidity(&NAME_METHOD.encode(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(NAME_RESULT.to_owned()), result);
}

#[test]
fn service_with_lifecycles_and_generics() {
    use service_with_lifecycles_and_generics::MyGenericService;

    const DO_THIS: &str = "DoThis";
    let input = (0u128, false).abi_encode_sequence();

    let my_service = MyGenericService::<'_, String>::default();

    let (output, ..) = my_service
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THIS.encode(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(42u32), result);
}

#[test]
fn service_with_extends_and_lifetimes() {
    use service_with_extends_and_lifetimes::{
        BASE_NAME_RESULT, BaseWithLifetime, EXTENDED_NAME_RESULT, ExtendedWithLifetime, NAME_RESULT,
    };

    const NAME_METHOD: &str = "Name";
    const BASE_NAME_METHOD: &str = "BaseName";
    const EXTENDED_NAME_METHOD: &str = "ExtendedName";
    let input = (0u128, false).abi_encode_sequence();

    let int = 42u64;
    let mut extended_svc =
        ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(123.into(), &[1, 2, 3]);

    let _base: &<BaseWithLifetime as Service>::Exposure = extended_svc.as_base_0();

    let (output, ..) = extended_svc
        .try_handle_solidity(&EXTENDED_NAME_METHOD.encode(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(EXTENDED_NAME_RESULT.to_owned()), result);

    let (output, ..) = extended_svc
        .try_handle_solidity(&BASE_NAME_METHOD.encode(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(BASE_NAME_RESULT.to_owned()), result);

    let (output, ..) = extended_svc
        .try_handle_solidity(&NAME_METHOD.encode(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(NAME_RESULT.to_owned()), result);
}

#[tokio::test]
async fn service_with_export_unwrap_result() {
    use service_with_export_unwrap_result::MyService;

    const DO_THIS: &str = "DoThis";

    let input = (0u128, false, 42u32, "correct").abi_encode_sequence();
    let (output, ..) = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity_async(&DO_THIS.encode(), input.as_slice())
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
#[should_panic(expected = "failed to parse `not a number`")]
async fn service_with_export_unwrap_result_panic() {
    use service_with_export_unwrap_result::MyService;

    const PARSE: &str = "Parse";
    let input = (0u128, false, "not a number").abi_encode_sequence();

    _ = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity_async(&PARSE.encode(), input.as_slice())
        .await
        .unwrap();
}

#[tokio::test]
async fn service_with_reply_with_value() {
    use service_with_reply_with_value::MyServiceWithReplyWithValue;

    const DO_THIS: &str = "DoThis";

    let input = (0u128, false, 42u32, "correct".to_owned()).abi_encode_sequence();
    let (output, value, ..) = MyServiceWithReplyWithValue
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity_async(&DO_THIS.encode(), input.as_slice())
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn service_with_reply_with_value_with_impl_from() {
    use service_with_reply_with_value::MyServiceWithReplyWithValue;

    const DO_THAT: &str = "DoThat";

    let input = (0u128, false, 42u32, "correct".to_owned()).abi_encode_sequence();
    let (output, value, ..) = MyServiceWithReplyWithValue
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity_async(&DO_THAT.encode(), input.as_slice())
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn service_with_trait_bounds() {
    use service_with_trait_bounds::MyServiceWithTraitBounds;

    const DO_THIS: &str = "DoThis";
    let input = (0u128, false).abi_encode_sequence();

    let (output, ..) = MyServiceWithTraitBounds::<u32>::default()
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THIS.encode(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(42u32), result);
}
