//! Benchmark tests for measuring Sails framework performance characteristics.
//!
//! This module contains integration tests that measure various performance aspects
//! of Sails applications including:
//!
//! - **Sync/async calls** - Tests sync/async calls efficiency for the simplest sails program (`counter_bench` test)
//! - **Memory allocation benchmarks** - Tests allocation patterns using stress program (`alloc_stress_bench` test)
//! - **Compute performance benchmarks** - Tests CPU-intensive operations like Fibonacci calculations (`compute_stress_bench` test)
//! - **Cross-program communication** - Tests performance of cross-program communication (`cross_program_bench` test)
//!
//! All benchmarks use the `gtest` framework to simulate on-chain execution and measure
//! gas consumption. Results are stored to the shared benchmark data file for analysis.
//!
//! To run benchmarks, use the following command (for example, from the root of the workspace):
//! ```bash
//! cargo test --release --manifest-path=benchmarks/Cargo.toml
//! ```

use alloc_stress_client::{
    AllocStressFactory, alloc_stress::io::AllocStress, traits::AllocStressFactory as _,
};
use compute_stress_client::{
    ComputeStressFactory, compute_stress::io::ComputeStress, traits::ComputeStressFactory as _,
};
use counter_bench_client::{
    CounterBenchFactory,
    counter_bench::io::{Inc, IncAsync},
    traits::CounterBenchFactory as _,
};
use ping_pong_bench_app::client::{
    PingPongFactory, PingPongPayload, ping_pong_service::io::Ping, traits::PingPongFactory as _,
};

use gtest::{System, constants::DEFAULT_USER_ALICE};
use sails_rs::{
    calls::{ActionIo, Activation},
    gtest::calls::GTestRemoting,
};
use std::{collections::BTreeMap, sync::atomic::AtomicU64};

static COUNTER_SALT: AtomicU64 = AtomicU64::new(0);

macro_rules! create_program_async {
    ($(($factory:ty, $wasm_path:expr)),* $(,)?) => {{
        let system = System::new();
        system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

        $(
            #[allow(unused)]
            let code_id = system.submit_local_code_file($wasm_path);
        )*

        let remoting = GTestRemoting::new(system, DEFAULT_USER_ALICE.into());

        (
            remoting.clone(),
            $(
                {
                    let salt = COUNTER_SALT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                        .to_le_bytes();
                    let factory = <$factory>::new(remoting.clone());
                    factory
                        .new_for_bench()
                        .send_recv(code_id, &salt)
                        .await
                        .expect("failed to initialize the program")
                }
            ),*
        )
    }};
}

macro_rules! call_action {
    ($remoting:expr, $pid:expr, $action:ty $(, $action_params:expr),*) => {{
        let system_remoting = $remoting.system();
        let program = system_remoting
            .get_program($pid)
            .expect("program was created; qed.");
        let from = $remoting.actor_id();

        // Form payload for the program
        let payload = <$action>::encode_call($($action_params),*);
        let mid = program.send_bytes(from, payload);
        let block_res = system_remoting.run_next_block();

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

        let resp = <$action>::decode_reply(payload).expect("failed to decode payload");

        let gas_burned = block_res
            .gas_burned
            .get(&mid)
            .copied()
            .expect("msg was executed; qed.");

        (resp, gas_burned)
    }};
}

#[tokio::test]
async fn alloc_stress_bench() {
    let mut benches: BTreeMap<usize, Vec<u64>> = Default::default();
    let fibonacci_ns = [0, 6, 11, 15, 20, 23, 25, 27];

    for _ in 0..100 {
        for &n in fibonacci_ns.iter() {
            let (len, gas) = alloc_stress_test(n).await;

            benches.entry(len).or_default().push(gas);
        }
    }

    for (len, gas_benches) in benches {
        crate::store_bench_data(|bench_data| {
            bench_data.alloc.insert(len, median(gas_benches));
        })
        .unwrap();
    }
}

#[tokio::test]
async fn compute_stress_bench() {
    let wasm_path = "../target/wasm32-gear/release/compute_stress_app.opt.wasm";

    let (remoting, pid) = create_program_async!((ComputeStressFactory<GTestRemoting>, wasm_path));

    let input_value = 30;
    let expected_sum = compute_stress_app::sum_of_fib(input_value);

    let mut gas_benches = (0..100)
        .map(|_| {
            let (stress_resp, gas) = call_action!(remoting, pid, ComputeStress, input_value);
            assert_eq!(stress_resp.res, expected_sum);
            gas
        })
        .collect::<Vec<_>>();
    gas_benches.sort_unstable();

    crate::store_bench_data(|bench_data| {
        bench_data.compute = median(gas_benches);
    })
    .unwrap();
}

#[tokio::test]
async fn counter_bench() {
    let wasm_path = "../target/wasm32-gear/release/counter_bench_app.opt.wasm";

    let (remoting, pid) = create_program_async!((CounterBenchFactory<GTestRemoting>, wasm_path));

    let mut gas_benches_sync = Vec::new();
    let mut gas_benches_async = Vec::new();
    let mut expected_value = 0;
    for _ in 0..100 {
        let (stress_resp, gas_sync_inc) = call_action!(remoting, pid, Inc);
        assert_eq!(stress_resp, expected_value);
        expected_value += 1;
        gas_benches_sync.push(gas_sync_inc);

        let (stress_resp, gas_async_inc) = call_action!(remoting, pid, IncAsync);
        assert_eq!(stress_resp, expected_value);
        expected_value += 1;
        gas_benches_async.push(gas_async_inc);
    }
    gas_benches_sync.sort_unstable();
    gas_benches_async.sort_unstable();

    crate::store_bench_data(|bench_data| {
        bench_data.counter.sync_call = median(gas_benches_sync);
        bench_data.counter.async_call = median(gas_benches_async);
    })
    .unwrap();
}

#[tokio::test]
async fn cross_program_bench() {
    let wasm_path = "../target/wasm32-gear/release/ping_pong_bench_app.opt.wasm";
    let (remoting, start_ping_pid, pong_pid) = create_program_async!(
        (PingPongFactory<GTestRemoting>, wasm_path),
        (PingPongFactory<GTestRemoting>, wasm_path)
    );

    let mut gas_benches = (0..100)
        .map(|_| {
            let (stress_resp, gas) = call_action!(
                remoting,
                start_ping_pid,
                Ping,
                PingPongPayload::Start(pong_pid)
            );
            assert_eq!(stress_resp, PingPongPayload::Finished);
            gas
        })
        .collect::<Vec<_>>();
    gas_benches.sort_unstable();

    crate::store_bench_data(|bench_data| {
        bench_data.cross_program = median(gas_benches);
    })
    .unwrap();
}

async fn alloc_stress_test(n: u32) -> (usize, u64) {
    // Path taken from the .binpath file
    let wasm_path = "../target/wasm32-gear/release/alloc_stress_app.opt.wasm";

    let (remoting, pid) = create_program_async!((AllocStressFactory<GTestRemoting>, wasm_path));
    let (stress_resp, gas) = call_action!(remoting, pid, AllocStress, n);

    let expected_len = alloc_stress_app::fibonacci_sum(n) as usize;
    assert_eq!(stress_resp.inner.len(), expected_len);

    (expected_len, gas)
}

fn median(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();

    assert!(!values.is_empty());

    let len = values.len();
    if len % 2 == 0 {
        let i = len / 2;
        (values[i - 1] + values[i]) / 2
    } else {
        values[len / 2]
    }
}
