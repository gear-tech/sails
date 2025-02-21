use sails_rs::alloy_sol_types::SolValue;
use sails_rs::gstd::services::Service;
use sails_rs::{Encode, MessageId};

mod service_with_basics;
mod service_with_extends;

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
