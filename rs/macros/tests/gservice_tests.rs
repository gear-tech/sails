#![cfg(not(feature = "ethexe"))]

use sails_rs::gstd::services::{Exposure, ExposureWithEvents, Service};
use sails_rs::header::SailsMessageHeader;
use sails_rs::meta::ServiceMeta;
use sails_rs::{Decode, Encode};

mod gservice_with_basics;
// mod gservice_with_events;
// mod gservice_with_export_unwrap_result;
// mod gservice_with_extends;
// mod gservice_with_extends_and_lifetimes;
// mod gservice_with_lifecycles_and_generics;
// mod gservice_with_lifetimes_and_events;
// mod gservice_with_multiple_names;
// mod gservice_with_reply_with_value;
// mod gservice_with_trait_bounds;

/// Same service name is used for all the tests,
/// because under the hood exposure call context
/// stores service names in a static map, which is
/// accessed by different tests in a multi-threaded
/// environment. This leads to test's failure in case
/// race condition occurs.
const SERVICE_NAME: &str = "TestService";
/// Service route which is same as `SERVICE_NAME.encode()`
// const SERVICE_ROUTE: &[u8] = &[44, 84, 101, 115, 116, 83, 101, 114, 118, 105, 99, 101];

#[tokio::test]
async fn gservice_with_basics() {
    use gservice_with_basics::DoThisParams;
    use gservice_with_basics::SomeService;

    let header = SailsMessageHeader::v1(gservice_with_basics::SomeService::INTERFACE_ID, 1, 0);

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
            input,
            |mut output, _| {
                // let service_route = String::decode(&mut output).unwrap();
                // assert_eq!(service_route, SERVICE_NAME);

                // let func_name = String::decode(&mut output).unwrap();
                // assert_eq!(func_name, DO_THIS);

                let result = String::decode(&mut output).unwrap();
                assert_eq!(result, "42: correct");

                assert_eq!(output.len(), 0);
            },
        )
        .await
        .unwrap();
}

// #[test]
// fn gservice_with_extends() {
//     use gservice_with_extends::{
//         base::{BASE_NAME_RESULT, Base},
//         extended::{EXTENDED_NAME_RESULT, Extended, NAME_RESULT},
//     };

//     const NAME_METHOD: &str = "Name";
//     const BASE_NAME_METHOD: &str = "BaseName";
//     const EXTENDED_NAME_METHOD: &str = "ExtendedName";

//     let extended_svc = Extended::new(Base).expose(SERVICE_ROUTE);

//     // Check asyncness of the service.
//     assert!(
//         !extended_svc
//             .check_asyncness(&EXTENDED_NAME_METHOD.encode())
//             .unwrap()
//     );

//     extended_svc
//         .try_handle(&EXTENDED_NAME_METHOD.encode(), |mut output, _| {
//             let actual = output.to_vec();
//             let expected = [
//                 SERVICE_ROUTE.to_vec(),
//                 EXTENDED_NAME_METHOD.encode(),
//                 EXTENDED_NAME_RESULT.encode(),
//             ]
//             .concat();

//             assert_eq!(actual, expected);

//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, EXTENDED_NAME_METHOD);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, EXTENDED_NAME_RESULT);
//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();

//     let extended_svc = Extended::new(Base).expose(SERVICE_ROUTE);
//     // Check asyncness of the base service.
//     assert!(
//         !extended_svc
//             .check_asyncness(&BASE_NAME_METHOD.encode())
//             .unwrap()
//     );

//     extended_svc
//         .try_handle(&BASE_NAME_METHOD.encode(), |mut output, _| {
//             // Even if base service method is called, the service route
//             // will be the same as extended service route.
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, BASE_NAME_METHOD);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, BASE_NAME_RESULT);
//         })
//         .unwrap();

//     let extended_svc = Extended::new(Base).expose(SERVICE_ROUTE);
//     // Check asyncness of the base service.
//     assert!(!extended_svc.check_asyncness(&NAME_METHOD.encode()).unwrap());

//     extended_svc
//         .try_handle(&NAME_METHOD.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, NAME_METHOD);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, NAME_RESULT);
//         })
//         .unwrap();
// }

// #[test]
// fn gservice_with_extends_renamed() {
//     use gservice_with_extends::{
//         base::Base, extended_renamed::ExtendedRenamed, other_base::Base as OtherBase,
//     };
//     use sails_rs::meta::ServiceMeta;

//     let base_services = <ExtendedRenamed as ServiceMeta>::base_services().collect::<Vec<_>>();
//     assert_eq!(base_services.len(), 2);

//     // You can create `ExtendedRenamed` with `Base` without renaming, as it's Rust type.
//     let _ = ExtendedRenamed::new((Base, OtherBase)).expose(SERVICE_ROUTE);

//     let (base_service_name, _) = base_services[0];
//     assert_eq!(base_service_name, "RenamedBase");

//     let (other_base_service_name, _) = base_services[1];
//     assert_eq!(other_base_service_name, "Base");
// }

// #[test]
// fn gservice_extends_pure() {
//     use gservice_with_extends::{
//         base::{Base, NAME_RESULT},
//         extended_pure::ExtendedPure,
//     };

//     const NAME_METHOD: &str = "Name";

//     let extended_svc = ExtendedPure::new(Base).expose(SERVICE_ROUTE);

//     extended_svc
//         .try_handle(&NAME_METHOD.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, NAME_METHOD);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, NAME_RESULT);
//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();
// }

// #[test]
// fn gservice_with_lifecycles_and_generics() {
//     use gservice_with_lifecycles_and_generics::SomeService;

//     const DO_THIS: &str = "DoThis";

//     let mut iter = [42u32].into_iter();
//     let my_service = SomeService::<'_, '_, String, _>::new(&mut iter);

//     my_service
//         .expose(SERVICE_ROUTE)
//         .try_handle(&DO_THIS.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, DO_THIS);

//             let result = u32::decode(&mut output).unwrap();
//             assert_eq!(result, 42);

//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "Unknown request: 0xffffffff..ffffffff")]
// async fn gservice_panic_on_unexpected_input() {
//     use gservice_with_basics::SomeService;

//     let input = [0xffu8; 16];
//     SomeService
//         .expose(SERVICE_ROUTE)
//         .try_handle_async(&input, |_, _| {
//             panic!("Should not reach here");
//         })
//         .await
//         .unwrap_or_else(|| sails_rs::gstd::unknown_input_panic("Unknown request", &input));
// }

// #[test]
// #[should_panic(expected = "Unknown request: 0x81112c00..00000000")]
// fn gservice_panic_on_unexpected_input_double_encoded() {
//     use gservice_with_basics::SomeService;

//     let input = [
//         44, 77, 101, 109, 101, 70, 97, 99, 116, 111, 114, 121, 84, 67, 114, 101, 97, 116, 101, 70,
//         117, 110, 103, 105, 98, 108, 101, 80, 114, 111, 103, 114, 97, 109, 32, 77, 101, 109, 101,
//         78, 97, 109, 101, 16, 77, 69, 77, 69, 2, 44, 68, 101, 115, 99, 114, 105, 112, 116, 105,
//         111, 110, 64, 104, 116, 116, 112, 115, 58, 47, 47, 105, 109, 103, 46, 99, 111, 109, 47, 1,
//         76, 104, 116, 116, 112, 58, 47, 47, 101, 120, 97, 109, 112, 108, 101, 46, 111, 114, 103,
//         47, 1, 76, 104, 116, 116, 112, 58, 47, 47, 116, 101, 108, 101, 103, 114, 97, 109, 46, 109,
//         101, 47, 1, 76, 104, 116, 116, 112, 58, 47, 47, 116, 119, 105, 116, 116, 101, 114, 46, 99,
//         111, 109, 47, 1, 72, 104, 116, 116, 112, 58, 47, 47, 100, 105, 115, 99, 111, 114, 100, 46,
//         103, 103, 47, 1, 84, 104, 116, 116, 112, 58, 47, 47, 116, 111, 107, 101, 110, 111, 109,
//         105, 99, 115, 46, 103, 103, 47, 232, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 150, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     ]
//     .encode()
//     .encode();

//     SomeService
//         .expose(SERVICE_ROUTE)
//         .try_handle(&input, |_, _| {
//             panic!("Should not reach here");
//         })
//         .unwrap_or_else(|| sails_rs::gstd::unknown_input_panic("Unknown request", &input));
// }

// #[test]
// fn gservice_with_events() {
//     use gservice_with_events::{MyEvents, MyServiceWithEvents};

//     let mut exposure = MyServiceWithEvents(0).expose(SERVICE_ROUTE);
//     let mut emitter = exposure.emitter();
//     exposure.my_method();

//     let events = emitter.take_events();
//     assert_eq!(events.len(), 1);
//     assert_eq!(events[0], MyEvents::Event1);
// }

// #[test]
// fn gservice_with_lifetimes_and_events() {
//     use gservice_with_lifetimes_and_events::{MyEvents, MyGenericEventsService};

//     const DO_THIS: &str = "DoThis";

//     let my_service = MyGenericEventsService::<'_, String>::default();
//     let exposure = my_service.expose(SERVICE_ROUTE);

//     let mut emitter = exposure.emitter();
//     exposure
//         .try_handle(&DO_THIS.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, DO_THIS);

//             let result = u32::decode(&mut output).unwrap();
//             assert_eq!(result, 42);

//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();

//     let events = emitter.take_events();
//     assert_eq!(events.len(), 1);
//     assert_eq!(events[0], MyEvents::Event1);
// }

// #[test]
// fn gservice_with_extends_and_lifetimes() {
//     use gservice_with_extends_and_lifetimes::{
//         BASE_NAME_RESULT, BaseWithLifetime, BaseWithLifetimeExposure, EXTENDED_NAME_RESULT,
//         ExtendedWithLifetime, HIDDEN_NAME_RESULT, NAME_RESULT,
//     };

//     const NAME_METHOD: &str = "Name";
//     const BASE_NAME_METHOD: &str = "BaseName";
//     const EXTENDED_NAME_METHOD: &str = "ExtendedName";

//     let int = 42u64;
//     let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(SERVICE_ROUTE);

//     extended_svc
//         .try_handle(&EXTENDED_NAME_METHOD.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, EXTENDED_NAME_METHOD);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, EXTENDED_NAME_RESULT);
//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();

//     let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(SERVICE_ROUTE);

//     extended_svc
//         .try_handle(&BASE_NAME_METHOD.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, BASE_NAME_METHOD);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, BASE_NAME_RESULT);
//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();

//     let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(SERVICE_ROUTE);

//     extended_svc
//         .try_handle(&NAME_METHOD.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, NAME_METHOD);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, NAME_RESULT);
//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();

//     let extended_svc = ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(SERVICE_ROUTE);
//     let base_svc: BaseWithLifetime = extended_svc.into();
//     let base_exposure: BaseWithLifetimeExposure<BaseWithLifetime> = base_svc.expose(SERVICE_ROUTE);

//     let base_name = base_exposure.name();
//     assert_eq!(HIDDEN_NAME_RESULT, base_name)
// }

// #[tokio::test]
// async fn gservice_with_reply_with_value() {
//     use gservice_with_reply_with_value::MyDoThisParams;
//     use gservice_with_reply_with_value::MyServiceWithReplyWithValue;

//     const DO_THIS: &str = "DoThis";

//     let input = [
//         DO_THIS.encode(),
//         MyDoThisParams {
//             p1: 42,
//             p2: "correct".into(),
//         }
//         .encode(),
//     ]
//     .concat();

//     // No sync call with `DoThis` route.
//     assert!(
//         MyServiceWithReplyWithValue
//             .expose(SERVICE_ROUTE)
//             .try_handle(&input, |_, _| {})
//             .is_none()
//     );

//     MyServiceWithReplyWithValue
//         .expose(SERVICE_ROUTE)
//         .try_handle_async(&input, |mut output, value| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, DO_THIS);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, "42: correct");
//             assert_eq!(output.len(), 0);

//             assert_eq!(value, 100_000_000_000);
//         })
//         .await
//         .unwrap();
// }

// #[tokio::test]
// async fn gservice_with_reply_with_value_with_impl_from() {
//     use gservice_with_reply_with_value::MyDoThisParams;
//     use gservice_with_reply_with_value::MyServiceWithReplyWithValue;

//     const DO_THAT: &str = "DoThat";

//     let input = [
//         DO_THAT.encode(),
//         MyDoThisParams {
//             p1: 42,
//             p2: "correct".into(),
//         }
//         .encode(),
//     ]
//     .concat();

//     MyServiceWithReplyWithValue
//         .expose(SERVICE_ROUTE)
//         .try_handle_async(&input, |mut output, value| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, DO_THAT);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, "42: correct");
//             assert_eq!(output.len(), 0);

//             assert_eq!(value, 100_000_000_000);
//         })
//         .await
//         .unwrap();
// }

// #[tokio::test]
// async fn gservice_with_trait_bounds() {
//     use gservice_with_trait_bounds::MyServiceWithTraitBounds;

//     const DO_THIS: &str = "DoThis";

//     // No async call with `DoThis` route.
//     assert!(
//         MyServiceWithTraitBounds::<u32>::default()
//             .expose(SERVICE_ROUTE)
//             .try_handle_async(&DO_THIS.encode(), |_, _| {})
//             .await
//             .is_none()
//     );

//     MyServiceWithTraitBounds::<u32>::default()
//         .expose(SERVICE_ROUTE)
//         .try_handle(&DO_THIS.encode(), |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, DO_THIS);

//             let result = u32::decode(&mut output).unwrap();
//             assert_eq!(result, 42);

//             assert_eq!(output.len(), 0);
//         })
//         .unwrap();
// }

// macro_rules! gservice_works {
//     ($service:expr) => {
//         // `DO_THIS` is an async call
//         let input = [
//             DO_THIS.encode(),
//             MyDoThisParams {
//                 p1: 42,
//                 p2: "correct".into(),
//             }
//             .encode(),
//         ]
//         .concat();
//         $service
//             .expose(SERVICE_ROUTE)
//             .try_handle_async(&input, |mut output, _| {
//                 let service_route = String::decode(&mut output).unwrap();
//                 assert_eq!(service_route, SERVICE_NAME);

//                 let func_name = String::decode(&mut output).unwrap();
//                 assert_eq!(func_name, DO_THIS);

//                 let result = String::decode(&mut output).unwrap();
//                 assert_eq!(result, "42: correct");
//                 assert_eq!(output.len(), 0);
//             })
//             .await
//             .unwrap();
//     };
// }

// #[tokio::test]
// async fn gservice_with_multiple_names() {
//     use gservice_with_multiple_names::MyDoThisParams;
//     const DO_THIS: &str = "DoThis";

//     gservice_works!(gservice_with_multiple_names::MyService);
//     gservice_works!(gservice_with_multiple_names::MyOtherService);
//     gservice_works!(gservice_with_multiple_names::yet_another_service::MyService);
// }

// #[tokio::test]
// async fn gservice_with_export_unwrap_result() {
//     use gservice_with_export_unwrap_result::MyDoThisParams;
//     use gservice_with_export_unwrap_result::MyService;

//     const DO_THIS: &str = "DoThis";

//     let input = [
//         DO_THIS.encode(),
//         MyDoThisParams {
//             p1: 42,
//             p2: "correct".into(),
//         }
//         .encode(),
//     ]
//     .concat();

//     MyService
//         .expose(SERVICE_ROUTE)
//         .try_handle_async(&input, |mut output, _| {
//             let service_route = String::decode(&mut output).unwrap();
//             assert_eq!(service_route, SERVICE_NAME);

//             let func_name = String::decode(&mut output).unwrap();
//             assert_eq!(func_name, DO_THIS);

//             let result = String::decode(&mut output).unwrap();
//             assert_eq!(result, "42: correct");

//             assert_eq!(output.len(), 0);
//         })
//         .await
//         .unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "failed to parse `not a number`")]
// async fn gservice_with_export_unwrap_result_panic() {
//     use gservice_with_export_unwrap_result::MyService;

//     const PARSE: &str = "Parse";

//     let input = (PARSE, "not a number").encode();

//     MyService
//         .expose(SERVICE_ROUTE)
//         .try_handle_async(&input, |_, _| {
//             unreachable!("Should not reach here");
//         })
//         .await
//         .unwrap();
// }
