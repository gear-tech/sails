#![cfg(not(feature = "ethexe"))]

use sails_rs::futures::*;
use sails_rs::gstd::services::Service;
use sails_rs::{Decode, Encode, MessageId};

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
    use gservice_with_basics::MyDoThisParams;
    use gservice_with_basics::MyService;

    const DO_THIS: &str = "DoThis";

    let input = [
        DO_THIS.encode(),
        MyDoThisParams {
            p1: 42,
            p2: "correct".into(),
        }
        .encode(),
    ]
    .concat();
    let (output, _value) = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&input)
        .await
        .unwrap();
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, "42: correct");

    assert_eq!(output.len(), 0);
}

#[tokio::test]
async fn gservice_with_extends() {
    use gservice_with_extends::{
        base::{Base, BASE_NAME_RESULT},
        extended::{Extended, EXTENDED_NAME_RESULT, NAME_RESULT},
    };

    const NAME_METHOD: &str = "Name";
    const BASE_NAME_METHOD: &str = "BaseName";
    const EXTENDED_NAME_METHOD: &str = "ExtendedName";

    let mut extended_svc = Extended::new(Base).expose(123.into(), &[1, 2, 3]);

    let (output, _value) = extended_svc
        .try_handle(&EXTENDED_NAME_METHOD.encode())
        .await
        .unwrap();

    assert_eq!(
        output,
        [EXTENDED_NAME_METHOD.encode(), EXTENDED_NAME_RESULT.encode()].concat()
    );

    let _base: &<Base as Service>::Exposure = extended_svc.as_base_0();

    let (output, _value) = extended_svc
        .try_handle(&BASE_NAME_METHOD.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, BASE_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, BASE_NAME_RESULT);

    let (output, _value) = extended_svc
        .try_handle(&EXTENDED_NAME_METHOD.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, EXTENDED_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, EXTENDED_NAME_RESULT);

    let (output, _value) = extended_svc
        .try_handle(&NAME_METHOD.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, NAME_RESULT);
}

#[tokio::test]
async fn gservice_with_lifecycles_and_generics() {
    use gservice_with_lifecycles_and_generics::MyGenericService;

    const DO_THIS: &str = "DoThis";

    let my_service = MyGenericService::<'_, String>::default();

    let (output, _value) = my_service
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&DO_THIS.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = u32::decode(&mut output).unwrap();
    assert_eq!(result, 42);

    assert_eq!(output.len(), 0);
}

#[tokio::test]
#[should_panic(expected = "Unknown request: 0xffffffff..ffffffff")]
async fn gservice_panic_on_unexpected_input() {
    use gservice_with_basics::MyService;

    let input = [0xffu8; 16];
    MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&input)
        .await
        .unwrap_or_else(|| sails_rs::gstd::unknown_input_panic("Unknown request", &input));
}

#[tokio::test]
#[should_panic(expected = "Unknown request: 0x81112c00..00000000")]
async fn gservice_panic_on_unexpected_input_double_encoded() {
    use gservice_with_basics::MyService;

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

    MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&input)
        .await
        .unwrap_or_else(|| sails_rs::gstd::unknown_input_panic("Unknown request", &input));
}

#[tokio::test]
async fn gservice_with_events() {
    use gservice_with_events::{MyEvents, MyServiceWithEvents};

    let mut exposure = MyServiceWithEvents(0).expose(MessageId::from(142), &[1, 4, 2]);
    let events = exposure.listen();
    exposure.my_method();

    drop(exposure); // close sender
    let events: Vec<MyEvents> = events.collect().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[tokio::test]
async fn gservice_with_lifetimes_and_events() {
    use gservice_with_lifetimes_and_events::{MyEvents, MyGenericEventsService};

    const DO_THIS: &str = "DoThis";

    let my_service = MyGenericEventsService::<'_, String>::default();
    let mut exposure = my_service.expose(MessageId::from(123), &[1, 2, 3]);
    let events = exposure.listen();

    let (output, _value) = exposure.try_handle(&DO_THIS.encode()).await.unwrap();

    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = u32::decode(&mut output).unwrap();
    assert_eq!(result, 42);

    assert_eq!(output.len(), 0);

    drop(exposure); // close sender
    let events: Vec<MyEvents> = events.collect().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}

#[tokio::test]
async fn gservice_with_extends_and_lifetimes() {
    use gservice_with_extends_and_lifetimes::{
        BaseWithLifetime, ExtendedWithLifetime, BASE_NAME_RESULT, EXTENDED_NAME_RESULT, NAME_RESULT,
    };

    const NAME_METHOD: &str = "Name";
    const BASE_NAME_METHOD: &str = "BaseName";
    const EXTENDED_NAME_METHOD: &str = "ExtendedName";

    let int = 42u64;
    let mut extended_svc =
        ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(123.into(), &[1, 2, 3]);

    let (output, _value) = extended_svc
        .try_handle(&EXTENDED_NAME_METHOD.encode())
        .await
        .unwrap();

    assert_eq!(
        output,
        [EXTENDED_NAME_METHOD.encode(), EXTENDED_NAME_RESULT.encode()].concat()
    );

    let _base: &<BaseWithLifetime as Service>::Exposure = extended_svc.as_base_0();

    let (output, _value) = extended_svc
        .try_handle(&BASE_NAME_METHOD.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, BASE_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, BASE_NAME_RESULT);

    let (output, _value) = extended_svc
        .try_handle(&EXTENDED_NAME_METHOD.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, EXTENDED_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, EXTENDED_NAME_RESULT);

    let (output, _value) = extended_svc
        .try_handle(&NAME_METHOD.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, NAME_RESULT);
}

#[tokio::test]
async fn gservice_with_reply_with_value() {
    use gservice_with_reply_with_value::MyDoThisParams;
    use gservice_with_reply_with_value::MyServiceWithReplyWithValue;

    const DO_THIS: &str = "DoThis";

    let input = [
        DO_THIS.encode(),
        MyDoThisParams {
            p1: 42,
            p2: "correct".into(),
        }
        .encode(),
    ]
    .concat();
    let (output, value) = MyServiceWithReplyWithValue
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&input)
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, "42: correct");

    assert_eq!(output.len(), 0);
}

#[tokio::test]
async fn gservice_with_reply_with_value_with_impl_from() {
    use gservice_with_reply_with_value::MyDoThisParams;
    use gservice_with_reply_with_value::MyServiceWithReplyWithValue;

    const DO_THAT: &str = "DoThat";

    let input = [
        DO_THAT.encode(),
        MyDoThisParams {
            p1: 42,
            p2: "correct".into(),
        }
        .encode(),
    ]
    .concat();
    let (output, value) = MyServiceWithReplyWithValue
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&input)
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THAT);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, "42: correct");

    assert_eq!(output.len(), 0);
}

#[tokio::test]
async fn gservice_with_trait_bounds() {
    use gservice_with_trait_bounds::MyServiceWithTraitBounds;

    const DO_THIS: &str = "DoThis";

    let (output, _value) = MyServiceWithTraitBounds::<u32>::default()
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&DO_THIS.encode())
        .await
        .unwrap();
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = u32::decode(&mut output).unwrap();
    assert_eq!(result, 42);

    assert_eq!(output.len(), 0);
}

macro_rules! gservice_works {
    ($service:expr) => {
        let input = [
            DO_THIS.encode(),
            MyDoThisParams {
                p1: 42,
                p2: "correct".into(),
            }
            .encode(),
        ]
        .concat();
        let (output, _value) = $service
            .expose(MessageId::from(123), &[1, 2, 3])
            .try_handle(&input)
            .await
            .unwrap();
        let mut output = output.as_slice();

        let func_name = String::decode(&mut output).unwrap();
        assert_eq!(func_name, DO_THIS);

        let result = String::decode(&mut output).unwrap();
        assert_eq!(result, "42: correct");
        assert_eq!(output.len(), 0);
    };
}

#[tokio::test]
async fn gservice_with_multiple_names() {
    use gservice_with_multiple_names::MyDoThisParams;
    const DO_THIS: &str = "DoThis";

    gservice_works!(gservice_with_multiple_names::MyService);
    gservice_works!(gservice_with_multiple_names::MyOtherService);
    gservice_works!(gservice_with_multiple_names::yet_another_service::MyService);
}

#[tokio::test]
async fn gservice_with_export_unwrap_result() {
    use gservice_with_export_unwrap_result::MyDoThisParams;
    use gservice_with_export_unwrap_result::MyService;

    const DO_THIS: &str = "DoThis";

    let input = [
        DO_THIS.encode(),
        MyDoThisParams {
            p1: 42,
            p2: "correct".into(),
        }
        .encode(),
    ]
    .concat();
    let (output, _value) = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&input)
        .await
        .unwrap();
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, "42: correct");

    assert_eq!(output.len(), 0);
}

#[tokio::test]
#[should_panic(expected = "failed to parse `not a number`")]
async fn gservice_with_export_unwrap_result_panic() {
    use gservice_with_export_unwrap_result::MyService;

    const PARSE: &str = "Parse";

    let input = (PARSE, "not a number").encode();
    _ = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle(&input)
        .await
        .unwrap();
}
