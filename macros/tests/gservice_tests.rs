use sails_rtl::gstd::services::Service;
use sails_rtl::{Decode, Encode, MessageId};

mod gservice_with_basics;
mod gservice_with_events;
mod gservice_with_extends;
mod gservice_with_lifecycles_and_generics;

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
    let output = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .handle(&input)
        .await;
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

    let output = extended_svc.handle(&EXTENDED_NAME_METHOD.encode()).await;

    assert_eq!(
        output,
        [EXTENDED_NAME_METHOD.encode(), EXTENDED_NAME_RESULT.encode()].concat()
    );

    let _base: &<Base as Service>::Exposure = extended_svc.as_base_0();

    let output = extended_svc.handle(&BASE_NAME_METHOD.encode()).await;
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, BASE_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, BASE_NAME_RESULT);

    let output = extended_svc.handle(&EXTENDED_NAME_METHOD.encode()).await;
    let mut output = output.as_slice();
    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, EXTENDED_NAME_METHOD);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, EXTENDED_NAME_RESULT);

    let output = extended_svc.handle(&NAME_METHOD.encode()).await;
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

    let output = my_service
        .expose(MessageId::from(123), &[1, 2, 3])
        .handle(&DO_THIS.encode())
        .await;
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
        .handle(&input)
        .await;
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
        .handle(&input)
        .await;
}

#[test]
fn gservice_with_events() {
    use gservice_with_events::{MyEvents, MyServiceWithEvents};

    let mut exposure = MyServiceWithEvents(0).expose(MessageId::from(142), &[1, 4, 2]);

    let mut events = Vec::new();
    {
        let _event_listener_guard = exposure.set_event_listener(|event| events.push(event.clone()));

        exposure.my_method();
    }

    assert_eq!(events.len(), 1);
    assert_eq!(events[0], MyEvents::Event1);
}
