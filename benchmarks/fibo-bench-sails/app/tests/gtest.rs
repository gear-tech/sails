use fibo_bench_sails_app as app;
use fibo_bench_sails_client::{
    FiboBenchSailsFactory, FiboStress, FiboStressResult,
    fibo_stress::io::Stress,
    traits::{FiboBenchSailsFactory as FiboBenchSailsFactoryTrait, FiboStress as FiboStressTrait},
};
use gtest::{Gas, System, constants::DEFAULT_USER_ALICE};
use sails_rs::{calls::*, gtest::calls::GTestRemoting};

// Path taken from the .binpath file
const WASM_PATH: &str = "../../../target/wasm32-gear/debug/fibo_bench_sails_app.opt.wasm";

async fn stress_test(n: u32) -> (Gas, usize) {
    // Prepare environment
    let sys = System::new();
    sys.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    sys.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = sys.submit_local_code_file(WASM_PATH);

    // Create program and initialize it
    let remoting = GTestRemoting::new(sys, DEFAULT_USER_ALICE.into());
    let fibo_program_factory = FiboBenchSailsFactory::new(remoting.clone());
    let pid = fibo_program_factory
        .new()
        .send_recv(code_id, b"fibo_prog")
        .await
        .expect("failed to initialize the program");

    // Using low-level `gtest` as it's possible to have block execution data this way.
    let sys_remoting = remoting.system();
    let program = sys_remoting
        .get_program(pid)
        .expect("program was created; qed.");

    // Form payload for the program
    let payload = Stress::encode_call(n);
    let mid = program.send_bytes(remoting.actor_id(), payload);
    let block_res = sys_remoting.run_next_block();

    // Check that message executed successfully
    assert!(block_res.succeed.contains(&mid));

    // Check received payload
    let payload = block_res
        .log()
        .iter()
        .find_map(|log| {
            log.reply_to()
                .filter(|reply_to| reply_to == &mid)
                .map(|_| log.payload().to_vec())
        })
        .expect("internal error: no reply was found");
    let stress_res = Stress::decode_reply(payload).expect("failed to decode payload");

    let expected_len = app::fibonacci_sum(n) as usize;
    assert_eq!(stress_res.inner.len(), expected_len);

    (
        block_res
            .gas_burned
            .get(&mid)
            .copied()
            .expect("msg was executed; qed."),
        expected_len,
    )
}

#[tokio::test]
async fn test_fibo_bench_sails() {
    let nums = [0, 6, 11, 15, 20, 23, 25, 27];
    for n in nums {
        let (gas, length) = stress_test(n).await;
        println!("{gas}");
    }
}
