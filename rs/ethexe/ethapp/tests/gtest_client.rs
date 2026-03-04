use sails_rs::{
    CodeId,
    alloy_primitives::B256,
    alloy_sol_types::SolValue,
    client::*,
    gear_core_errors::{ErrorReplyReason, SimpleExecutionError},
    gtest::System,
};

#[path = "ethapp_sol_client.rs"]
mod ethapp_sol_client;
pub use ethapp_sol_client::{EthAppProgram, EthApp, EthAppCtors, svc_1::Svc1};

#[cfg(debug_assertions)]
pub(crate) const WASM_PATH: &str = "../target/wasm32-gear/debug/ethapp.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const WASM_PATH: &str = "../target/wasm32-gear/release/ethapp.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn ethapp_sol_works() {
    let (env, code_id) = create_env();
    let actor = Deployment::<EthAppProgram, _>::new(env.clone(), code_id, vec![])
        .create_prg()
        .await
        .unwrap();

    let wasm_size = std::fs::metadata(WASM_PATH).unwrap().len();
    println!("[ethapp_sol_works] Size: {wasm_size}");

    let reply = actor.svc_1().do_this(42, "hello".into()).await;
    assert_eq!(reply, Ok(42));
    println!("[ethapp_sol_works] Handle ok, Size: {wasm_size}");
}

#[tokio::test]
async fn ethapp_remoting_works() {
    let (env, code_id) = create_env();
    let actor = Deployment::<EthAppProgram, _>::new(env.clone(), code_id, vec![])
        .create_prg()
        .await
        .unwrap();

    assert_eq!(actor.svc_1().do_this(42, "hello".into()).await, Ok(42));
}

#[tokio::test]
async fn ethapp_remoting_encode_reply_works() {
    let (env, code_id) = create_env();
    let payload = <ethapp_sol_client::io::CreatePrg as ServiceCall>::encode_params_with_header(
        0,
        &(true,),
    );
    let (program_id, message_id) = env
        .create_program(code_id, vec![], payload.as_slice(), Default::default())
        .unwrap();
    let reply_payload = env
        .message_reply_from_next_blocks(message_id)
        .await
        .unwrap()
        .unwrap();

    // assert
    let callback_selector = sails_rs::solidity::selector("replyOn_createPrg(bytes32)");
    assert_eq!(callback_selector.as_slice(), &reply_payload[..4]);
    let (_message_id,) = <(B256,)>::abi_decode_sequence(&reply_payload[4..]).unwrap();

    // arrange
    let payload = <ethapp_sol_client::svc_1::io::DoThis as ServiceCall>::encode_params_with_header(
        EthAppProgram::ROUTE_ID_SVC_1,
        &(true, 42, "hello".to_string()),
    );
    let reply_payload = env
        .send_for_reply(program_id, payload, Default::default())
        .await
        .unwrap();

    // assert
    let callback_selector = sails_rs::solidity::selector("replyOn_svc1DoThis(bytes32,uint32)");
    assert_eq!(callback_selector.as_slice(), &reply_payload[..4]);

    let (_message_id, result) = <(B256, u32)>::abi_decode_sequence(&reply_payload[4..]).unwrap();
    assert_eq!(42, result);
}

#[tokio::test]
async fn ethapp_ctor_non_payable_fails_with_value() {
    let (env, code_id) = create_env();
    let err = match Deployment::<EthAppProgram, _>::new(env.clone(), code_id, vec![])
        .create_prg()
        .with_value(1000)
        .await
    {
        Ok(_) => panic!("Expected userspace panic"),
        Err(err) => err,
    };
    assert_userspace_panic(err, "panicked with ''create_prg' accepts no value'");
}

#[tokio::test]
async fn ethapp_ctor_payable_works_with_value() {
    let (env, code_id) = create_env();
    assert!(
        Deployment::<EthAppProgram, _>::new(env.clone(), code_id, vec![])
            .create_payable()
            .with_value(1000)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn ethapp_method_non_payable_fails_with_value() {
    let (env, code_id) = create_env();
    let actor = Deployment::<EthAppProgram, _>::new(env.clone(), code_id, vec![])
        .create_prg()
        .await
        .unwrap();

    let err = match actor
        .svc_1()
        .do_this(42, "hello".into())
        .with_value(1000)
        .await
    {
        Ok(_) => panic!("Expected userspace panic"),
        Err(err) => err,
    };
    assert_userspace_panic(err, "panicked with ''do_this' accepts no value'");
}

#[tokio::test]
async fn ethapp_method_payable_works_with_value() {
    let (env, code_id) = create_env();
    let actor = Deployment::<EthAppProgram, _>::new(env.clone(), code_id, vec![])
        .create_prg()
        .await
        .unwrap();

    assert_eq!(
        actor.svc_1().do_this_payable(42).with_value(1000).await,
        Ok(42)
    );
}

fn assert_userspace_panic(err: GtestError, expected_message: &str) {
    match err {
        GtestError::ReplyHasError(
            ErrorReplyReason::Execution(SimpleExecutionError::UserspacePanic),
            payload,
        ) => {
            let msg = String::from_utf8_lossy(&payload);
            assert_eq!(msg, expected_message);
        }
        _ => panic!("Expected userspace panic, got {err:?}"),
    }
}

fn create_env() -> (GtestEnv, CodeId) {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);

    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, ADMIN_ID.into());
    (env, code_id)
}
