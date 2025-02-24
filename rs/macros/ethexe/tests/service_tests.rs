use sails_rs::alloy_sol_types::SolValue;
use sails_rs::gstd::services::Service;
use sails_rs::{Encode, MessageId};

mod service_with_basics;
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
    let input = (42u32, "correct".to_owned()).abi_encode_params();

    // act
    let (output, _value) = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THIS.encode(), &input)
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn service_with_extends() {
    use service_with_extends::{
        base::{Base, BASE_NAME_RESULT},
        extended::{Extended, EXTENDED_NAME_RESULT, NAME_RESULT},
    };

    const NAME_METHOD: &str = "Name";
    const BASE_NAME_METHOD: &str = "BaseName";
    const EXTENDED_NAME_METHOD: &str = "ExtendedName";

    let mut extended_svc = Extended::new(Base).expose(123.into(), &[1, 2, 3]);

    let (output, _value) = extended_svc
        .try_handle_solidity(&EXTENDED_NAME_METHOD.encode(), &[])
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(EXTENDED_NAME_RESULT.to_owned()), result);

    let _base: &<Base as Service>::Exposure = extended_svc.as_base_0();

    let (output, _value) = extended_svc
        .try_handle_solidity(&BASE_NAME_METHOD.encode(), &[])
        .await
        .unwrap();
    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(BASE_NAME_RESULT.to_owned()), result);

    let (output, _value) = extended_svc
        .try_handle_solidity(&NAME_METHOD.encode(), &[])
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(NAME_RESULT.to_owned()), result);
}

#[tokio::test]
async fn service_with_lifecycles_and_generics() {
    use service_with_lifecycles_and_generics::MyGenericService;

    const DO_THIS: &str = "DoThis";

    let my_service = MyGenericService::<'_, String>::default();

    let (output, _value) = my_service
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THIS.encode(), &[])
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(42u32), result);
}

#[tokio::test]
async fn service_with_extends_and_lifetimes() {
    use service_with_extends_and_lifetimes::{
        BaseWithLifetime, ExtendedWithLifetime, BASE_NAME_RESULT, EXTENDED_NAME_RESULT, NAME_RESULT,
    };

    const NAME_METHOD: &str = "Name";
    const BASE_NAME_METHOD: &str = "BaseName";
    const EXTENDED_NAME_METHOD: &str = "ExtendedName";

    let int = 42u64;
    let mut extended_svc =
        ExtendedWithLifetime::new(BaseWithLifetime::new(&int)).expose(123.into(), &[1, 2, 3]);

    let _base: &<BaseWithLifetime as Service>::Exposure = extended_svc.as_base_0();

    let (output, _value) = extended_svc
        .try_handle_solidity(&EXTENDED_NAME_METHOD.encode(), &[])
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(EXTENDED_NAME_RESULT.to_owned()), result);

    let (output, _value) = extended_svc
        .try_handle_solidity(&BASE_NAME_METHOD.encode(), &[])
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(BASE_NAME_RESULT.to_owned()), result);

    let (output, _value) = extended_svc
        .try_handle_solidity(&NAME_METHOD.encode(), &[])
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(NAME_RESULT.to_owned()), result);
}

#[tokio::test]
async fn gservice_with_export_unwrap_result() {
    use service_with_export_unwrap_result::MyService;

    const DO_THIS: &str = "DoThis";

    let input = (42u32, "correct".to_owned()).abi_encode_params();
    let (output, _value) = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THIS.encode(), input.as_slice())
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
#[should_panic(expected = "failed to parse `not a number`")]
async fn gservice_with_export_unwrap_result_panic() {
    use service_with_export_unwrap_result::MyService;

    const PARSE: &str = "Parse";
    let input = ("not a number",).abi_encode_params();

    _ = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&PARSE.encode(), input.as_slice())
        .await
        .unwrap();
}

#[tokio::test]
async fn gservice_with_reply_with_value() {
    use service_with_reply_with_value::MyServiceWithReplyWithValue;

    const DO_THIS: &str = "DoThis";

    let input = (42u32, "correct".to_owned()).abi_encode_params();
    let (output, value) = MyServiceWithReplyWithValue
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THIS.encode(), input.as_slice())
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn gservice_with_reply_with_value_with_impl_from() {
    use service_with_reply_with_value::MyServiceWithReplyWithValue;

    const DO_THAT: &str = "DoThat";

    let input = (42u32, "correct".to_owned()).abi_encode_params();
    let (output, value) = MyServiceWithReplyWithValue
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THAT.encode(), input.as_slice())
        .await
        .unwrap();

    assert_eq!(value, 100_000_000_000);

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok("42: correct".to_owned()), result);
}

#[tokio::test]
async fn gservice_with_trait_bounds() {
    use service_with_trait_bounds::MyServiceWithTraitBounds;

    const DO_THIS: &str = "DoThis";

    let (output, _value) = MyServiceWithTraitBounds::<u32>::default()
        .expose(MessageId::from(123), &[1, 2, 3])
        .try_handle_solidity(&DO_THIS.encode(), &[])
        .await
        .unwrap();

    let result = sails_rs::alloy_sol_types::SolValue::abi_decode(output.as_slice(), false);
    assert_eq!(Ok(42u32), result);
}
