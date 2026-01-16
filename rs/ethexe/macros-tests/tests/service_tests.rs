use sails_rs::{
    MessageId, Syscall,
    alloy_primitives::B256,
    alloy_sol_types::SolValue,
    gstd::services::{Exposure, ExposureWithEvents as _, Service},
    meta::{SailsMessageHeader, ServiceMeta},
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

    let input = (false, 42u32, "correct".to_owned()).abi_encode_sequence();

    let exposure = MyService.expose(1);

    let header = SailsMessageHeader::v1(<MyService as ServiceMeta>::INTERFACE_ID, 0, 1);
    // Check asyncness for `DoThis`.
    assert!(
        <<MyService as Service>::Exposure as Exposure>::check_asyncness(
            header.interface_id(),
            header.entry_id()
        )
        .unwrap()
    );

    assert!(
        exposure
            .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
            .is_none()
    );

    // act
    let (output, ..) = MyService
        .expose(1)
        .try_handle_solidity_async(header.interface_id(), header.entry_id(), &input)
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn service_with_basics_with_encode_reply() {
    use service_with_basics::MyService;

    let input = (true, 42u32, "correct".to_owned()).abi_encode_sequence();
    let message_id = MessageId::from(123);
    Syscall::with_message_id(message_id);

    let header = SailsMessageHeader::v1(<MyService as ServiceMeta>::INTERFACE_ID, 0, 1);
    // act
    let (output, ..) = MyService
        .expose(1)
        .try_handle_solidity_async(header.interface_id(), header.entry_id(), &input)
        .await
        .unwrap();

    let (mid, result): (B256, String) =
        sails_rs::alloy_sol_types::SolValue::abi_decode_sequence(output.as_slice()).unwrap();
    assert_eq!(message_id, MessageId::new(mid.0));
    assert_eq!("42: correct", result.as_str());
}

#[test]
fn service_with_events() {
    use service_with_events::{MyEvents, MyServiceWithEvents};

    let mut exposure = MyServiceWithEvents(0).expose(1);
    let mut emitter = exposure.emitter();
    exposure.my_method();

    let events = emitter.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[test]
fn service_with_lifetimes_and_events() {
    use service_with_events_and_lifetimes::{MyEvents, MyGenericEventsService};

    let input = (false,).abi_encode_sequence();

    let my_service = MyGenericEventsService::<'_, String>::default();
    let exposure = my_service.expose(1);
    let mut emitter = exposure.emitter();

    let header =
        SailsMessageHeader::v1(<MyGenericEventsService as ServiceMeta>::INTERFACE_ID, 0, 1);
    // Check asyncness for `DoThis`.
    assert!(
        !<<MyGenericEventsService as Service>::Exposure as Exposure>::check_asyncness(
            header.interface_id(),
            header.entry_id()
        )
        .unwrap()
    );

    let (output, ..) = exposure
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(42), result);

    let events = emitter.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[test]
fn service_with_extends() {
    use service_with_extends::{
        base::{BASE_NAME_RESULT, Base},
        extended::{EXTENDED_NAME_RESULT, Extended, NAME_RESULT},
    };

    let input = (false,).abi_encode_sequence();

    let extended_svc = Extended::new(Base).expose(1);

    // Extended::extended_name
    let header = SailsMessageHeader::v1(Extended::INTERFACE_ID, 0, 1);
    // Check asyncness of the service.
    assert!(
        !<Extended as Service>::Exposure::check_asyncness(header.interface_id(), header.entry_id())
            .unwrap()
    );

    let (output, ..) = extended_svc
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(EXTENDED_NAME_RESULT.to_owned()), result);

    let extended_svc = Extended::new(Base).expose(1);
    // Base::base_name
    let header = SailsMessageHeader::v1(Base::INTERFACE_ID, 0, 1);
    let (output, ..) = extended_svc
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();
    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(BASE_NAME_RESULT.to_owned()), result);

    let extended_svc = Extended::new(Base).expose(1);
    // Extended::name
    let header = SailsMessageHeader::v1(Extended::INTERFACE_ID, 1, 1);
    let (output, ..) = extended_svc
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(NAME_RESULT.to_owned()), result);
}

#[test]
fn service_with_lifecycles_and_generics() {
    use service_with_lifecycles_and_generics::MyGenericService;

    let input = (false,).abi_encode_sequence();

    let my_service = MyGenericService::<'_, String>::default();
    // SomeService::do_this
    let header = SailsMessageHeader::v1(<MyGenericService as ServiceMeta>::INTERFACE_ID, 0, 1);

    let (output, ..) = my_service
        .expose(1)
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(42u32), result);
}

#[test]
fn service_with_extends_and_lifetimes() {
    use service_with_extends_and_lifetimes::{
        BASE_NAME_RESULT, BaseWithLifetime, EXTENDED_NAME_RESULT, ExtendedWithLifetime, NAME_RESULT,
    };

    let input = (false,).abi_encode_sequence();

    let int = 42u64;
    let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(1);

    // ExtendedWithLifetime::extended_name
    let header = SailsMessageHeader::v1(ExtendedWithLifetime::INTERFACE_ID, 0, 1);
    let (output, ..) = extended_svc
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(EXTENDED_NAME_RESULT.to_owned()), result);

    let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(1);
    // BaseWithLifetime::base_name
    let header = SailsMessageHeader::v1(BaseWithLifetime::INTERFACE_ID, 0, 1);
    let (output, ..) = extended_svc
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(BASE_NAME_RESULT.to_owned()), result);

    let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(1);
    // ExtendedWithLifetime::name
    let header = SailsMessageHeader::v1(ExtendedWithLifetime::INTERFACE_ID, 1, 1);
    let (output, ..) = extended_svc
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(NAME_RESULT.to_owned()), result);
}

#[tokio::test]
async fn service_with_export_unwrap_result() {
    use service_with_export_unwrap_result::MyService;

    let header = SailsMessageHeader::v1(<MyService as ServiceMeta>::INTERFACE_ID, 0, 1);

    let input = (false, 42u32, "correct").abi_encode_sequence();
    let (output, ..) = MyService
        .expose(1)
        .try_handle_solidity_async(header.interface_id(), header.entry_id(), &input)
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
#[should_panic(expected = "failed to parse `not a number`")]
async fn service_with_export_unwrap_result_panic() {
    use service_with_export_unwrap_result::MyService;

    let header = SailsMessageHeader::v1(<MyService as ServiceMeta>::INTERFACE_ID, 1, 1);

    let input = (false, "not a number").abi_encode_sequence();

    _ = MyService
        .expose(1)
        .try_handle_solidity_async(header.interface_id(), header.entry_id(), &input)
        .await
        .unwrap();
}

#[tokio::test]
async fn service_with_reply_with_value() {
    use service_with_reply_with_value::MyServiceWithReplyWithValue;

    // MyServiceWithReplyWithValue::do_this
    let header = SailsMessageHeader::v1(MyServiceWithReplyWithValue::INTERFACE_ID, 1, 1);
    let input = (false, 42u32, "correct".to_owned()).abi_encode_sequence();
    let (output, value, ..) = MyServiceWithReplyWithValue
        .expose(1)
        .try_handle_solidity_async(header.interface_id(), header.entry_id(), &input)
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn service_with_reply_with_value_with_impl_from() {
    use service_with_reply_with_value::MyServiceWithReplyWithValue;

    // MyServiceWithReplyWithValue::do_that
    let header = SailsMessageHeader::v1(MyServiceWithReplyWithValue::INTERFACE_ID, 0, 1);
    let input = (false, 42u32, "correct".to_owned()).abi_encode_sequence();
    let (output, value, ..) = MyServiceWithReplyWithValue
        .expose(1)
        .try_handle_solidity_async(header.interface_id(), header.entry_id(), &input)
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn service_with_trait_bounds() {
    use service_with_trait_bounds::MyServiceWithTraitBounds;

    let input = (false,).abi_encode_sequence();

    // MyServiceWithTraitBounds::do_this
    let header = SailsMessageHeader::v1(
        <MyServiceWithTraitBounds as ServiceMeta>::INTERFACE_ID,
        0,
        1,
    );
    let (output, ..) = MyServiceWithTraitBounds::<u32>::default()
        .expose(1)
        .try_handle_solidity(header.interface_id(), header.entry_id(), &input)
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice());
    assert_eq!(Ok(42u32), result);
}
