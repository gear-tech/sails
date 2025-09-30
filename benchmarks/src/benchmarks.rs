//! Benchmark tests for measuring Sails framework performance characteristics.
//!
//! This module contains integration tests that measure various performance aspects
//! of Sails applications including:
//!
//! - **Sync/async calls** - Tests sync/async calls efficiency for the simplest sails program (`counter_bench` test)
//! - **Memory allocation benchmarks** - Tests allocation patterns using stress program (`alloc_stress_bench` test)
//! - **Compute performance benchmarks** - Tests CPU-intensive operations like Fibonacci calculations (`compute_stress_bench` test)
//! - **Cross-program performance** - Tests performance of cross-program communication (`cross_program_bench` test)
//! - **Redirect performance** - Tests performance of redirecting calls to another program (`redirect_bench` test)
//!
//! All benchmarks use the `gtest` framework to simulate on-chain execution and measure
//! gas consumption. Results are stored to the shared benchmark data file for analysis.
//!
//! To run benchmarks, use the following command (for example, from the root of the workspace):
//! ```bash
//! cargo test --release --manifest-path=benchmarks/Cargo.toml
//! ```

use crate::clients::{
    alloc_stress_client::{AllocStressProgram, AllocStressProgramCtors, alloc_stress::*},
    compute_stress_client::{ComputeStressProgram, ComputeStressProgramCtors, compute_stress::*},
    counter_bench_client::{CounterBenchProgram, CounterBenchProgramCtors, counter_bench::*},
};
use gtest::{System, constants::DEFAULT_USER_ALICE};
use itertools::{Either, Itertools};
use ping_pong_bench_app::client::{PingPong, PingPongCtors, PingPongPayload, ping_pong_service::*};
use redirect_client::{RedirectClient, RedirectClientCtors, redirect::*};
use redirect_proxy_client::{RedirectProxyClient, RedirectProxyClientCtors, proxy::*};
use sails_rs::{client::*, prelude::*};
use std::{collections::BTreeMap, sync::atomic::AtomicU64};

static COUNTER_SALT: AtomicU64 = AtomicU64::new(0);

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
            bench_data.update_alloc_bench(len, median(gas_benches));
        })
        .unwrap();
    }
}

#[tokio::test]
async fn compute_stress_bench() {
    let wasm_path = "../target/wasm32-gear/release/compute_stress.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| {
        ComputeStressProgramCtors::new_for_bench(d)
    })
    .await;
    let mut service = program.compute_stress();

    let input_value = 30;
    let expected_sum = compute_stress::sum_of_fib(input_value);

    let mut gas_benches = (0..100)
        .map(|_| {
            let message_id = service.compute_stress(input_value).send_one_way().unwrap();
            let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
            let stress_resp = crate::clients::compute_stress_client::compute_stress::io::ComputeStress::decode_reply_with_prefix(
                "ComputeStress",
                payload.as_slice(),
            )
            .unwrap();
            assert_eq!(stress_resp.res, expected_sum);
            gas
        })
        .collect::<Vec<_>>();
    gas_benches.sort_unstable();

    crate::store_bench_data(|bench_data| {
        bench_data.update_compute_bench(median(gas_benches));
    })
    .unwrap();
}

#[tokio::test]
async fn counter_bench() {
    let wasm_path = "../target/wasm32-gear/release/counter_bench.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| {
        CounterBenchProgramCtors::new_for_bench(d)
    })
    .await;
    let mut service = program.counter_bench();

    let mut expected_value = 0;
    let (mut gas_benches_sync, mut gas_benches_async): (Vec<_>, Vec<_>) = (0..100)
        .enumerate()
        .map(|(i, _)| {
            let is_sync = i % 2 == 0;
            let gas = if is_sync {
                let message_id = service.inc().send_one_way().unwrap();
                let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
                let stress_resp = crate::clients::counter_bench_client::counter_bench::io::Inc::decode_reply_with_prefix(
                    "CounterBench",
                    payload.as_slice(),
                )
                .unwrap();
                assert_eq!(stress_resp, expected_value);
                expected_value += 1;

                gas
            } else {
                let message_id = service.inc_async().send_one_way().unwrap();
                let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
                let stress_resp = crate::clients::counter_bench_client::counter_bench::io::IncAsync::decode_reply_with_prefix(
                    "CounterBench",
                    payload.as_slice(),
                )
                .unwrap();
                assert_eq!(stress_resp, expected_value);
                expected_value += 1;

                gas
            };

            (i, gas)
        })
        .partition_map(|(i, gas)| {
            if i % 2 == 0 {
                Either::Left(gas) // Sync call
            } else {
                Either::Right(gas) // Async call
            }
        });

    gas_benches_sync.sort_unstable();
    gas_benches_async.sort_unstable();

    crate::store_bench_data(|bench_data| {
        bench_data.update_counter_bench(false, median(gas_benches_sync));
        bench_data.update_counter_bench(true, median(gas_benches_async));
    })
    .unwrap();
}

#[tokio::test]
async fn cross_program_bench() {
    let wasm_path = "../target/wasm32-gear/release/ping_pong_bench_app.opt.wasm";
    let env = create_env();
    let program_ping = deploy_for_bench(&env, wasm_path, |d| PingPongCtors::new_for_bench(d)).await;
    let program_pong = deploy_for_bench(&env, wasm_path, |d| PingPongCtors::new_for_bench(d)).await;

    let mut service = program_ping.ping_pong_service();

    let mut gas_benches = (0..100)
        .map(|_| {
            let message_id = service
                .ping(PingPongPayload::Start(program_pong.id()))
                .send_one_way()
                .unwrap();
            let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
            let stress_resp =
                ping_pong_bench_app::client::ping_pong_service::io::Ping::decode_reply_with_prefix(
                    "PingPongService",
                    payload.as_slice(),
                )
                .unwrap();
            assert_eq!(stress_resp, PingPongPayload::Finished);
            gas
        })
        .collect::<Vec<_>>();
    gas_benches.sort_unstable();

    crate::store_bench_data(|bench_data| {
        bench_data.update_cross_program_bench(median(gas_benches));
    })
    .unwrap();
}

#[tokio::test]
async fn redirect_bench() {
    let redirect_wasm_path = "../target/wasm32-gear/release/redirect_app.opt.wasm";
    let proxy_wasm_path = "../target/wasm32-gear/release/redirect_proxy.opt.wasm";

    let env = create_env();
    let program_redirect_1 =
        deploy_for_bench(&env, redirect_wasm_path, |d| RedirectClientCtors::new(d)).await;
    let program_redirect_2 =
        deploy_for_bench(&env, redirect_wasm_path, |d| RedirectClientCtors::new(d)).await;
    let program_proxy = deploy_for_bench(&env, proxy_wasm_path, |d| {
        RedirectProxyClientCtors::new(d, program_redirect_1.id())
    })
    .await;

    // Warm-up proxy program
    (0..100).for_each(|_| {
        let message_id = program_proxy
            .proxy()
            .get_program_id()
            .send_one_way()
            .unwrap();
        let (payload, _gas) = extract_reply_and_gas(env.system(), message_id);
        let resp = redirect_proxy_client::proxy::io::GetProgramId::decode_reply_with_prefix(
            "Proxy",
            payload.as_slice(),
        )
        .unwrap();
        assert_eq!(resp, program_redirect_1.id());
    });

    // Call exit on a redirect program
    program_redirect_1
        .redirect()
        .exit(program_redirect_2.id())
        .send_one_way()
        .unwrap();

    // Bench proxy program
    let gas_benches = (0..100)
        .map(|_| {
            let message_id = program_proxy
                .proxy()
                .get_program_id()
                .send_one_way()
                .unwrap();
            let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
            let resp = redirect_proxy_client::proxy::io::GetProgramId::decode_reply_with_prefix(
                "Proxy",
                payload.as_slice(),
            )
            .unwrap();
            assert_eq!(resp, program_redirect_2.id());
            gas
        })
        .collect::<Vec<_>>();

    crate::store_bench_data(|bench_data| {
        bench_data.update_redirect_bench(median(gas_benches));
    })
    .unwrap();
}

async fn alloc_stress_test(n: u32) -> (usize, u64) {
    // Path taken from the .binpath file
    let wasm_path = "../target/wasm32-gear/release/alloc_stress.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| {
        AllocStressProgramCtors::new_for_bench(d)
    })
    .await;

    let mut service = program.alloc_stress();
    let message_id = service.alloc_stress(n).send_one_way().unwrap();
    let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
    let stress_resp = crate::clients::alloc_stress_client::alloc_stress::io::AllocStress::decode_reply_with_prefix(
        "AllocStress",
        payload.as_slice(),
    )
    .unwrap();

    let expected_len = alloc_stress::fibonacci_sum(n) as usize;
    assert_eq!(stress_resp.inner.len(), expected_len);

    (expected_len, gas)
}

fn create_env() -> GtestEnv {
    let system = System::new();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);
    GtestEnv::new(system, DEFAULT_USER_ALICE.into())
}

async fn deploy_for_bench<
    P: Program,
    IO: CallCodec,
    F: FnOnce(Deployment<GtestEnv, P>) -> PendingCtor<GtestEnv, P, IO>,
>(
    env: &GtestEnv,
    wasm_path: &str,
    f: F,
) -> Actor<GtestEnv, P> {
    let code_id = env.system().submit_local_code_file(wasm_path);
    let salt = COUNTER_SALT
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        .to_le_bytes()
        .to_vec();
    let deployment = env.deploy::<P>(code_id, salt);
    let ctor = f(deployment);
    let program = ctor.await.expect("failed to initialize the program");
    program
}

fn extract_reply_and_gas(system: &System, message_id: MessageId) -> (Vec<u8>, u64) {
    let block_res = system.run_next_block();
    assert!(block_res.succeed.contains(&message_id));
    let payload = block_res
        .log()
        .iter()
        .find_map(|log| {
            log.reply_to()
                .filter(|reply_to| *reply_to == message_id)
                .map(|_| log.payload().to_vec())
        })
        .expect("reply found");

    let gas = *block_res.gas_burned.get(&message_id).expect("gas recorded");
    (payload, gas)
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
