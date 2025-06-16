//! Entry point for fibonacci stress benchmarking.
//!
//! TODO add docs

#![cfg(test)]

use alloc_stress_client::{
    AllocStressFactory, AllocStressResult, alloc_stress::io::AllocStress,
    traits::AllocStressFactory as _,
};
use gtest::{Gas, Program, System, constants::DEFAULT_USER_ALICE};
use sails_rs::{
    calls::{ActionIo, Activation},
    gtest::calls::GTestRemoting,
    prelude::Decode,
};

trait Len {
    fn len(&self) -> usize;
}

impl Len for AllocStressResult {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Len for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }
}

async fn stress_test<A>(n: u32) -> (Gas, usize)
where
    A: ActionIo<Params = u32>,
    A::Reply: Len,
{
    // Path taken from the .binpath file
    // TODO [sab] use release
    const WASM_PATH: &str = "../../../target/wasm32-gear/debug/alloc_stress_app.opt.wasm";

    let system = System::new();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = system.submit_local_code_file(WASM_PATH);

    // Create program and initialize it
    let remoting = GTestRemoting::new(system, DEFAULT_USER_ALICE.into());
    let factory = AllocStressFactory::new(remoting.clone());
    let pid = factory
        .new()
        .send_recv(code_id, b"fibo_prog")
        .await
        .expect("failed to initialize the program");

    // Using low-level `gtest` as it's possible to have block execution data this way.
    let system_remoting = remoting.system();
    let program = system_remoting
        .get_program(pid)
        .expect("program was created; qed.");
    let from = remoting.actor_id();

    // Form payload for the program
    let payload = A::encode_call(&n);
    let mid = program.send_bytes(from, payload);
    let block_res = system_remoting.run_next_block();

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
    let stress_res = A::decode_reply(payload).expect("failed to decode payload");

    let expected_len = alloc_stress_app::fibonacci_sum(n) as usize;
    assert_eq!(stress_res.len(), expected_len);

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
async fn sails_stress_fibo() {
    let fibonacci_ns = [0, 6, 11, 15, 20, 23, 25, 27];

    for &n in fibonacci_ns.iter() {
        let (gas, len) = stress_test::<AllocStress>(n).await;

        benchmarks::store_bench_data(|bench_data| {
            bench_data.update_alloc(len.try_into().unwrap(), gas);
        });
    }
}
