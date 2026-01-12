#![cfg(not(feature = "ethexe"))]

use sails_rs::gstd::services::{Exposure, ExposureWithEvents, Service};
use sails_rs::header::SailsMessageHeader;
use sails_rs::meta::{InterfaceId, ServiceMeta};
use sails_rs::{Decode, Encode};

mod gservice_with_basics;
mod gservice_with_events;
mod gservice_with_export_unwrap_result;
mod gservice_with_extends;
mod gservice_with_extends_and_lifetimes;
mod gservice_with_lifecycles_and_generics;
mod gservice_with_lifetimes_and_events;
mod gservice_with_multiple_names;
mod gservice_with_reply_with_value;
mod gservice_with_trait_bounds;

#[tokio::test]
async fn gservice_with_basics() {
    use gservice_with_basics::DoThisParams;
    use gservice_with_basics::SomeService;

    let header = SailsMessageHeader::v1(SomeService::INTERFACE_ID, 0, 1);

    let input = DoThisParams {
        p1: 42,
        p2: "correct".into(),
    }
    .encode();

    // Check asyncness of the service.
    assert!(
        <SomeService as Service>::Exposure::check_asyncness(
            header.interface_id(),
            header.entry_id()
        )
        .unwrap()
    );

    SomeService
        .expose(header.route_id())
        .try_handle_async(
            header.interface_id(),
            header.entry_id(),
            &input,
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(res_header.interface_id(), SomeService::INTERFACE_ID);
                assert_eq!(res_header.entry_id(), 0);
                assert_eq!(res_header.route_id(), 1);

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, "42: correct");
                assert_eq!(output.len(), 0);
            },
        )
        .await
        .unwrap();
}

#[test]
fn gservice_with_extends() {
    use gservice_with_extends::{
        base::{BASE_NAME_RESULT, Base},
        extended::{EXTENDED_NAME_RESULT, Extended},
    };

    let extended_svc = Extended::new(Base).expose(1);
    // Extended::extended_name
    let header = SailsMessageHeader::v1(Extended::INTERFACE_ID, 0, 1);

    // Check asyncness of the service.
    assert!(
        !<Extended as Service>::Exposure::check_asyncness(header.interface_id(), header.entry_id())
            .unwrap()
    );

    extended_svc
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(res_header.interface_id(), Extended::INTERFACE_ID);
                assert_eq!(res_header.entry_id(), 0);
                assert_eq!(res_header.route_id(), 1);

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, EXTENDED_NAME_RESULT);
                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();

    let extended_svc = Extended::new(Base).expose(1);
    // Base::base_name
    let header = SailsMessageHeader::v1(Base::INTERFACE_ID, 0, 1);
    // Check asyncness of the base service.
    assert!(
        !<Base as Service>::Exposure::check_asyncness(header.interface_id(), header.entry_id())
            .unwrap()
    );

    extended_svc
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(res_header.interface_id(), Base::INTERFACE_ID);
                assert_eq!(res_header.entry_id(), 0);
                assert_eq!(res_header.route_id(), 1);

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, BASE_NAME_RESULT);
            },
        )
        .unwrap();

    let extended_svc = Extended::new(Base).expose(1);
    // Base::name
    let header = SailsMessageHeader::v1(Base::INTERFACE_ID, 1, 1);
    // Check asyncness of the base service.
    assert!(
        !<Base as Service>::Exposure::check_asyncness(header.interface_id(), header.entry_id())
            .unwrap()
    );

    extended_svc
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(res_header.interface_id(), Base::INTERFACE_ID);
                assert_eq!(res_header.entry_id(), 1);
                assert_eq!(res_header.route_id(), 1);

                // TODO: method not overrided
                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, gservice_with_extends::base::NAME_RESULT);
            },
        )
        .unwrap();
}

#[test]
fn gservice_with_extends_renamed() {
    use gservice_with_extends::{
        base::Base, extended_renamed::ExtendedRenamed, other_base::Base as OtherBase,
    };
    use sails_rs::meta::ServiceMeta;

    let base_services = <ExtendedRenamed as ServiceMeta>::base_services()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(base_services.len(), 2);

    // You can create `ExtendedRenamed` with `Base` without renaming, as it's Rust type.
    let _ = ExtendedRenamed::new((Base, OtherBase)).expose(1);

    let base_service_meta = base_services[0];
    assert_eq!(base_service_meta.name, "RenamedBase");

    let other_base_service_meta = base_services[1];
    assert_eq!(other_base_service_meta.name, "Base");
}

#[test]
fn gservice_extends_pure() {
    use gservice_with_extends::{
        base::{Base, NAME_RESULT},
        extended_pure::ExtendedPure,
    };

    let extended_svc = ExtendedPure::new(Base).expose(1);

    // Base::name
    let header = SailsMessageHeader::v1(Base::INTERFACE_ID, 1, 1);

    extended_svc
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(res_header.interface_id(), Base::INTERFACE_ID);
                assert_eq!(res_header.entry_id(), 1);
                assert_eq!(res_header.route_id(), 1);

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, NAME_RESULT);
                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();
}

#[test]
fn gservice_with_lifecycles_and_generics() {
    use gservice_with_lifecycles_and_generics::SomeService;

    let mut iter = [42u32].into_iter();
    let my_service = SomeService::<'_, '_, String, _>::new(&mut iter);

    // SomeService::do_this
    let header = SailsMessageHeader::v1(<SomeService as ServiceMeta>::INTERFACE_ID, 0, 1);

    my_service
        .expose(1)
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    <SomeService as ServiceMeta>::INTERFACE_ID
                );
                assert_eq!(res_header.entry_id(), 0);
                assert_eq!(res_header.route_id(), 1);

                let result = u32::decode(&mut output).unwrap();
                assert_eq!(result, 42);

                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Unknown request: 0xffffffff..ffffffff")]
async fn gservice_panic_on_unexpected_input() {
    use gservice_with_basics::SomeService;

    let input = [0xffu8; 16];
    SomeService
        .expose(1)
        .try_handle_async(InterfaceId::zero(), 0, &input, |_, _| {
            panic!("Should not reach here");
        })
        .await
        .unwrap_or_else(|| sails_rs::gstd::unknown_input_panic("Unknown request", &input));
}

#[test]
#[should_panic(expected = "Unknown request: 0x81112c00..00000000")]
fn gservice_panic_on_unexpected_input_double_encoded() {
    use gservice_with_basics::SomeService;

    let input = [
        44, 77, 101, 109, 101, 70, 97, 99, 116, 111, 114, 121, 84, 67, 114, 101, 97, 116, 101, 70,
        117, 110, 103, 105, 98, 108, 101, 80, 114, 111, 103, 114, 97, 109, 32, 77, 101, 109, 101,
        78, 97, 109, 101, 16, 77, 69, 77, 69, 2, 44, 68, 101, 115, 99, 114, 105, 112, 116, 105,
        111, 110, 64, 104, 116, 116, 112, 115, 58, 47, 47, 105, 109, 103, 46, 99, 111, 109, 47, 1,
        76, 104, 116, 116, 112, 58, 47, 47, 101, 120, 97, 109, 112, 108, 101, 46, 111, 114, 103,
        47, 1, 76, 104, 116, 116, 112, 58, 47, 47, 116, 101, 108, 101, 103, 114, 97, 109, 46, 109,
        101, 47, 1, 76, 104, 116, 116, 112, 58, 47, 47, 116, 119, 105, 116, 116, 101, 114, 46, 99,
        111, 109, 47, 1, 72, 104, 116, 116, 112, 58, 47, 47, 100, 105, 115, 99, 111, 114, 100, 46,
        103, 103, 47, 1, 84, 104, 116, 116, 112, 58, 47, 47, 116, 111, 107, 101, 110, 111, 109,
        105, 99, 115, 46, 103, 103, 47, 232, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 150, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
    .encode()
    .encode();

    SomeService
        .expose(1)
        .try_handle(InterfaceId::zero(), 0, &input, |_, _| {
            panic!("Should not reach here");
        })
        .unwrap_or_else(|| sails_rs::gstd::unknown_input_panic("Unknown request", &input));
}

#[test]
fn gservice_with_events() {
    use gservice_with_events::{MyEvents, MyServiceWithEvents};

    let mut exposure = MyServiceWithEvents(0).expose(1);
    let mut emitter = exposure.emitter();
    exposure.my_method();

    let events = emitter.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[test]
fn gservice_with_lifetimes_and_events() {
    use gservice_with_lifetimes_and_events::{MyEvents, Service};

    let my_service = Service::<'_, String>::default();
    let exposure = my_service.expose(1);
    // Base::name
    let header = SailsMessageHeader::v1(<Service as ServiceMeta>::INTERFACE_ID, 0, 1);

    let mut emitter = exposure.emitter();
    exposure
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    <Service as ServiceMeta>::INTERFACE_ID
                );

                let result = u32::decode(&mut output).unwrap();
                assert_eq!(result, 42);

                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();

    let events = emitter.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[test]
fn gservice_with_extends_and_lifetimes() {
    use gservice_with_extends_and_lifetimes::{
        BASE_NAME_RESULT, BaseWithLifetime, BaseWithLifetimeExposure, EXTENDED_NAME_RESULT,
        ExtendedWithLifetime, HIDDEN_NAME_RESULT, NAME_RESULT,
    };

    let int = 42u64;
    let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(1);

    // ExtendedWithLifetime::extended_name
    let header = SailsMessageHeader::v1(ExtendedWithLifetime::INTERFACE_ID, 0, 1);

    extended_svc
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    ExtendedWithLifetime::INTERFACE_ID
                );

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, EXTENDED_NAME_RESULT);
                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();

    let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(1);
    // BaseWithLifetime::base_name
    let header = SailsMessageHeader::v1(BaseWithLifetime::INTERFACE_ID, 0, 1);

    extended_svc
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(res_header.interface_id(), BaseWithLifetime::INTERFACE_ID);

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, BASE_NAME_RESULT);
                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();

    let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(1);
    // ExtendedWithLifetime::name
    let header = SailsMessageHeader::v1(ExtendedWithLifetime::INTERFACE_ID, 1, 1);

    extended_svc
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    ExtendedWithLifetime::INTERFACE_ID
                );
                assert_eq!(res_header.entry_id(), 1);
                assert_eq!(res_header.route_id(), 1);

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, NAME_RESULT);
                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();

    let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(1);
    let base_svc: BaseWithLifetime = extended_svc.into();
    let base_exposure: BaseWithLifetimeExposure<BaseWithLifetime> = base_svc.expose(1);

    let base_name = base_exposure.name();
    assert_eq!(HIDDEN_NAME_RESULT, base_name)
}

#[tokio::test]
async fn gservice_with_reply_with_value() {
    use gservice_with_reply_with_value::MyDoThisParams;
    use gservice_with_reply_with_value::MyServiceWithReplyWithValue;

    let input = MyDoThisParams {
        p1: 42,
        p2: "correct".into(),
    }
    .encode();

    // MyServiceWithReplyWithValue::do_this
    let header = SailsMessageHeader::v1(MyServiceWithReplyWithValue::INTERFACE_ID, 1, 1);

    // No sync call with `DoThis` route.
    assert!(
        MyServiceWithReplyWithValue
            .expose(1)
            .try_handle(header.interface_id(), header.entry_id(), &input, |_, _| {})
            .is_none()
    );

    MyServiceWithReplyWithValue
        .expose(1)
        .try_handle_async(
            header.interface_id(),
            header.entry_id(),
            &input,
            |mut output, value| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    MyServiceWithReplyWithValue::INTERFACE_ID
                );

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, "42: correct");
                assert_eq!(output.len(), 0);

                assert_eq!(value, 100_000_000_000);
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn gservice_with_reply_with_value_with_impl_from() {
    use gservice_with_reply_with_value::MyDoThisParams;
    use gservice_with_reply_with_value::MyServiceWithReplyWithValue;

    let input = MyDoThisParams {
        p1: 42,
        p2: "correct".into(),
    }
    .encode();

    // MyServiceWithReplyWithValue::do_that
    let header = SailsMessageHeader::v1(MyServiceWithReplyWithValue::INTERFACE_ID, 0, 1);

    MyServiceWithReplyWithValue
        .expose(1)
        .try_handle_async(
            header.interface_id(),
            header.entry_id(),
            &input,
            |mut output, value| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    MyServiceWithReplyWithValue::INTERFACE_ID
                );

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, "42: correct");
                assert_eq!(output.len(), 0);

                assert_eq!(value, 100_000_000_000);
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn gservice_with_trait_bounds() {
    use gservice_with_trait_bounds::MyServiceWithTraitBounds;

    // MyServiceWithReplyWithValue::do_that
    let header = SailsMessageHeader::v1(
        <MyServiceWithTraitBounds as ServiceMeta>::INTERFACE_ID,
        0,
        1,
    );

    // No async call with `DoThis` route.
    assert!(
        MyServiceWithTraitBounds::<u32>::default()
            .expose(1)
            .try_handle_async(header.interface_id(), header.entry_id(), &[], |_, _| {})
            .await
            .is_none()
    );

    MyServiceWithTraitBounds::<u32>::default()
        .expose(1)
        .try_handle(
            header.interface_id(),
            header.entry_id(),
            &[],
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    <MyServiceWithTraitBounds as ServiceMeta>::INTERFACE_ID,
                );

                let result = u32::decode(&mut output).unwrap();
                assert_eq!(result, 42);

                assert_eq!(output.len(), 0);
            },
        )
        .unwrap();
}

macro_rules! gservice_works {
    ($service:ty) => {
        let header = SailsMessageHeader::v1(<$service as ServiceMeta>::INTERFACE_ID, 0, 1);
        // `DO_THIS` is an async call
        let input = gservice_with_multiple_names::MyDoThisParams {
            p1: 42,
            p2: "correct".into(),
        }
        .encode();

        <$service as Default>::default()
            .expose(1)
            .try_handle_async(
                header.interface_id(),
                header.entry_id(),
                &input,
                |mut output, _| {
                    let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                    assert_eq!(
                        res_header.interface_id(),
                        <$service as ServiceMeta>::INTERFACE_ID,
                    );

                    let result = String::decode(&mut output).unwrap();
                    assert_eq!(result, "42: correct");
                    assert_eq!(output.len(), 0);
                },
            )
            .await
            .unwrap();
    };
}

#[tokio::test]
async fn gservice_with_multiple_names() {
    gservice_works!(gservice_with_multiple_names::MyService);
    gservice_works!(gservice_with_multiple_names::MyOtherService);
    gservice_works!(gservice_with_multiple_names::yet_another_service::MyService);
}

#[tokio::test]
async fn gservice_with_export_unwrap_result() {
    use gservice_with_export_unwrap_result::MyDoThisParams;
    use gservice_with_export_unwrap_result::MyService;

    let header = SailsMessageHeader::v1(<MyService as ServiceMeta>::INTERFACE_ID, 0, 1);

    let input = MyDoThisParams {
        p1: 42,
        p2: "correct".into(),
    }
    .encode();

    MyService
        .expose(1)
        .try_handle_async(
            header.interface_id(),
            header.entry_id(),
            &input,
            |mut output, _| {
                let res_header = SailsMessageHeader::decode(&mut output).unwrap();
                assert_eq!(
                    res_header.interface_id(),
                    <MyService as ServiceMeta>::INTERFACE_ID,
                );

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, "42: correct");

                assert_eq!(output.len(), 0);
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "failed to parse `not a number`")]
async fn gservice_with_export_unwrap_result_panic() {
    use gservice_with_export_unwrap_result::MyService;

    let header = SailsMessageHeader::v1(<MyService as ServiceMeta>::INTERFACE_ID, 1, 1);

    let input = "not a number".encode();

    MyService
        .expose(1)
        .try_handle_async(header.interface_id(), header.entry_id(), &input, |_, _| {
            unreachable!("Should not reach here");
        })
        .await
        .unwrap();
}
