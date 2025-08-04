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
    alloc_stress_client::{
        AllocStressProgramFactory, alloc_stress::io::AllocStress,
        traits::AllocStressProgramFactory as _,
    },
    compute_stress_client::{
        ComputeStressProgramFactory, compute_stress::io::ComputeStress,
        traits::ComputeStressProgramFactory as _,
    },
    counter_bench_client::{
        CounterBenchProgramFactory,
        counter_bench::io::{Inc, IncAsync},
        traits::CounterBenchProgramFactory as _,
    },
};
use gtest::{System, constants::DEFAULT_USER_ALICE};
use itertools::{Either, Itertools};
use ping_pong_bench_app::client::{
    PingPongFactory, PingPongPayload, ping_pong_service::io::Ping, traits::PingPongFactory as _,
};
use redirect_client::{
    RedirectClientFactory, redirect::io::Exit, traits::RedirectClientFactory as _,
};
use redirect_proxy_client::{
    RedirectProxyClientFactory, proxy::io::GetProgramId, traits::RedirectProxyClientFactory as _,
};
use sails_rs::{
    calls::{ActionIo, Activation},
    gtest::calls::GTestRemoting,
};
use std::{collections::BTreeMap, sync::atomic::AtomicU64};

static COUNTER_SALT: AtomicU64 = AtomicU64::new(0);

macro_rules! create_program_async {
    ($(($factory:ty, $wasm_path:expr)),+) => {{
        create_program_async!($(($factory, $wasm_path, new_for_bench)),+)
    }};

    ($(($factory:ty, $wasm_path:expr, $ctor_name:ident $(, $ctor_params:expr),*)),+) => {{
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
                        .$ctor_name($($ctor_params),*)
                        .send_recv(code_id, &salt)
                        .await
                        .expect("failed to initialize the program")
                }
            ),*
        )
    }};

    ($remoting:expr, $(($factory:ty, $wasm_path:expr)),+) => {{
        create_program_async!($remoting, $(($factory, $wasm_path, new_for_bench)),+)
    }};

    ($remoting:expr, $(($factory:ty, $wasm_path:expr, $ctor_name:ident $(, $ctor_params:expr),*)),+) => {{
        let remoting = $remoting;

        (
            remoting.clone(),
            $({
                let code_id = remoting.system().submit_local_code_file($wasm_path);
                let salt = COUNTER_SALT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    .to_le_bytes();
                let factory = <$factory>::new(remoting.clone());
                factory
                    .$ctor_name($($ctor_params),*)
                    .send_recv(code_id, &salt)
                    .await
                    .expect("failed to initialize the program")
            }),*
        )
    }};
}

macro_rules! call_action {
    ($remoting:expr, $pid:expr, $action:ty $(, $action_params:expr),*) => {{
        call_action!($remoting, $pid, $action $(, $action_params),* ; with_reply)
    }};

    ($remoting:expr, $pid:expr, $action:ty $(, $action_params:expr),* ; no_reply_check) => {{
        call_action!($remoting, $pid, $action $(, $action_params),* ; no_reply)
    }};

    ($remoting:expr, $pid:expr, $action:ty $(, $action_params:expr),* ; $mode:ident) => {{
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

        call_action!(@handle_result $mode, block_res, mid, $action)
    }};

    (@handle_result with_reply, $block_res:expr, $mid:expr, $action:ty) => {{
        let payload = $block_res
            .log()
            .iter()
            .find_map(|log| {
                log.reply_to()
                    .filter(|reply_to| reply_to == &$mid)
                    .map(|_| log.payload().to_vec())
            })
            .expect("reply found");

        let resp = <$action>::decode_reply(payload).expect("decode reply");
        let gas = $block_res.gas_burned.get(&$mid).copied().expect("gas recorded");
        (resp, gas)
    }};

    (@handle_result no_reply, $block_res:expr, $mid:expr, $action:ty) => {{
        $block_res.gas_burned.get(&$mid).copied().expect("gas recorded")
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
    let wasm_path = "../target/wasm32-gear/release/compute_stress.opt.wasm";

    let (remoting, pid) =
        create_program_async!((ComputeStressProgramFactory::<GTestRemoting>, wasm_path));

    let input_value = 30;
    let expected_sum = compute_stress::sum_of_fib(input_value);

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
    let wasm_path = "../target/wasm32-gear/release/counter_bench.opt.wasm";

    let (remoting, pid) =
        create_program_async!((CounterBenchProgramFactory::<GTestRemoting>, wasm_path));

    let mut expected_value = 0;
    let (mut gas_benches_sync, mut gas_benches_async): (Vec<_>, Vec<_>) = (0..100)
        .enumerate()
        .map(|(i, _)| {
            let is_sync = i % 2 == 0;
            let gas = if is_sync {
                let (stress_resp, gas_sync_inc) = call_action!(remoting, pid, Inc);
                assert_eq!(stress_resp, expected_value);
                expected_value += 1;

                gas_sync_inc
            } else {
                let (stress_resp, gas_async_inc) = call_action!(remoting, pid, IncAsync);
                assert_eq!(stress_resp, expected_value);
                expected_value += 1;

                gas_async_inc
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
        bench_data.counter.sync_call = median(gas_benches_sync);
        bench_data.counter.async_call = median(gas_benches_async);
    })
    .unwrap();
}

#[tokio::test]
async fn cross_program_bench() {
    let wasm_path = "../target/wasm32-gear/release/ping_pong_bench_app.opt.wasm";
    let (remoting, start_ping_pid, pong_pid) = create_program_async!(
        (PingPongFactory::<GTestRemoting>, wasm_path),
        (PingPongFactory::<GTestRemoting>, wasm_path)
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

#[tokio::test]
async fn redirect_bench() {
    let redirect_wasm_path = "../target/wasm32-gear/release/redirect_app.opt.wasm";
    let proxy_wasm_path = "../target/wasm32-gear/release/redirect_proxy.opt.wasm";

    let (remoting, redirect_pid1, redirect_pid2) = create_program_async!(
        (
            RedirectClientFactory::<GTestRemoting>,
            redirect_wasm_path,
            new
        ),
        (
            RedirectClientFactory::<GTestRemoting>,
            redirect_wasm_path,
            new
        )
    );
    let (remoting, proxy_pid) = create_program_async!(
        remoting,
        (
            RedirectProxyClientFactory::<GTestRemoting>,
            proxy_wasm_path,
            new,
            redirect_pid1
        )
    );

    // Warm-up proxy program
    (0..100).for_each(|_| {
        let (resp, _) = call_action!(remoting, proxy_pid, GetProgramId);
        assert_eq!(resp, redirect_pid1);
    });

    // Call exit on a redirect program
    call_action!(remoting, redirect_pid1, Exit, redirect_pid2; no_reply_check);

    // Bench proxy program
    let gas_benches = (0..100)
        .map(|_| {
            let (resp, gas_get_program) = call_action!(remoting, proxy_pid, GetProgramId);
            assert_eq!(resp, redirect_pid2);

            gas_get_program
        })
        .collect::<Vec<_>>();

    crate::store_bench_data(|bench_data| {
        bench_data.redirect = median(gas_benches);
    })
    .unwrap();
}

async fn alloc_stress_test(n: u32) -> (usize, u64) {
    // Path taken from the .binpath file
    let wasm_path = "../target/wasm32-gear/release/alloc_stress.opt.wasm";

    let (remoting, pid) =
        create_program_async!((AllocStressProgramFactory::<GTestRemoting>, wasm_path));
    let (stress_resp, gas) = call_action!(remoting, pid, AllocStress, n);

    let expected_len = alloc_stress::fibonacci_sum(n) as usize;
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
