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
//! - **Example storage benchmarks** - Tests example-shaped state paths (`aggregator_tracker_bench` test)
//! - **Storage benchmarks** - Compares allocator-backed maps with fixed open-addressed maps (`storage_stress_bench` test)
//! - **VFT storage benchmarks** - Tests transfer-shaped balance/allowance paths (`vft_storage_transfer_bench` test)
//!
//! All benchmarks use the `gtest` framework to simulate on-chain execution and measure
//! gas consumption. Results are stored to the shared benchmark data file for analysis.
//!
//! To run benchmarks, use the following command (for example, from the root of the workspace):
//! ```bash
//! cargo test --release --manifest-path=benchmarks/Cargo.toml
//! ```

#[cfg(feature = "gas-profile")]
use crate::clients::minimal_vft_sails_client::{
    MinimalVftSails as _, MinimalVftSailsCtors, MinimalVftSailsProgram, vft::Vft as _,
};
#[cfg(feature = "gas-profile")]
use crate::clients::noop_sails_client::{
    NoopSails as _, NoopSailsCtors, NoopSailsProgram, noop_sails::NoopSails as _,
};
#[cfg(feature = "gas-profile")]
use crate::clients::vft_stress_client::vft_stress::{VftPhase, VftProfileResult};
use crate::clients::{
    alloc_stress_client::{
        AllocStress as _, AllocStressCtors, AllocStressProgram, alloc_stress::*,
    },
    compute_stress_client::{
        ComputeStress as _, ComputeStressCtors, ComputeStressProgram, compute_stress::*,
    },
    counter_bench_client::{
        CounterBench as _, CounterBenchCtors, CounterBenchProgram, counter_bench::*,
    },
    storage_million_client::{
        StorageMillion as _, StorageMillionCtors, StorageMillionProgram, storage_million::*,
    },
    storage_stress_client::{
        StorageStress as _, StorageStressCtors, StorageStressProgram,
        storage_stress::{
            StorageBackend, StorageBenchResult, StorageMap, StorageOp, StorageStress as _,
        },
    },
    vft_stress_client::{
        VftStress as _, VftStressCtors, VftStressProgram,
        vft_stress::{VftStorageBackend, VftStress as _, VftTransferOp, VftTransferResult},
    },
};
use aggregator_client::{
    AggregatorClient as _, AggregatorClientCtors, AggregatorClientProgram,
    TrackerBackend as AggregatorTrackerBackend,
    aggregator::{
        Aggregator as _, OpStatus as AggregatorOpStatus,
        TrackerBenchResult as AggregatorTrackerBenchResult, TrackerOp as AggregatorTrackerOp,
    },
};
#[cfg(feature = "gas-profile")]
use gtest::{Program as GtestProgram, ProgramBuilder as GtestProgramBuilder};
use gtest::{System, constants::DEFAULT_USER_ALICE};
use itertools::{Either, Itertools};
use ping_pong_bench_app::client::{PingPong, PingPongCtors, PingPongProgram, ping_pong_service::*};
use redirect_client::{RedirectClient, RedirectClientCtors, redirect::*};
use redirect_proxy_client::{
    RedirectProxyClient, RedirectProxyClientCtors, RedirectProxyClientProgram, proxy::*,
};
use sails_rs::{client::*, prelude::*};
#[cfg(feature = "gas-profile")]
use std::path::Path;
use std::{collections::BTreeMap, env, sync::atomic::AtomicU64};

static COUNTER_SALT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "gas-profile")]
const DEFAULT_FLOOR_SAMPLES: u32 = 3;
#[cfg(feature = "gas-profile")]
const HOT_VFT_INTERFACE_ID: [u8; 8] = [0x6d, 0x69, 0x6e, 0x76, 0x66, 0x74, 0x00, 0x01];
#[cfg(feature = "gas-profile")]
const HOT_VFT_ROUTE_ID: u8 = 1;
#[cfg(feature = "gas-profile")]
const HOT_VFT_APPROVE_ENTRY: u16 = 0;
#[cfg(feature = "gas-profile")]
const HOT_VFT_TRANSFER_ENTRY: u16 = 1;
#[cfg(feature = "gas-profile")]
const HOT_VFT_TRANSFER_FROM_ENTRY: u16 = 2;
const STORAGE_MILLION_LOAD: u32 = 1_000_000;
const STORAGE_MILLION_SAMPLES: u32 = 1;
const STORAGE_MILLION_BATCH_COUNT: u32 = 512;
const STORAGE_MILLION_PREPARE_CHUNK: u32 = 512;

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
    let program = deploy_for_bench(&env, wasm_path, |d| ComputeStressCtors::new_for_bench(d)).await;
    let mut service = program.compute_stress();

    let input_value = 30;
    let expected_sum = compute_stress::sum_of_fib(input_value);

    let mut gas_benches = (0..100)
        .map(|_| {
            let message_id = service.compute_stress(input_value).send_one_way().unwrap();
            let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
            // Low-level approach: decoding using generated io module
            let stress_resp =
                crate::clients::compute_stress_client::compute_stress::io::ComputeStress::decode_reply(ComputeStressProgram::ROUTE_ID_COMPUTE_STRESS, payload.as_slice())
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
    let program = deploy_for_bench(&env, wasm_path, |d| CounterBenchCtors::new_for_bench(d)).await;
    let mut service = program.counter_bench();

    let mut expected_value = 0;
    let (mut gas_benches_sync, mut gas_benches_async): (Vec<_>, Vec<_>) = (0..100)
        .enumerate()
        .map(|(i, _)| {
            let is_sync = i % 2 == 0;
            let gas = if is_sync {
                let message_id = service.inc().send_one_way().unwrap();
                let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
                // Low-level approach: decoding using generated io module
                let stress_resp = crate::clients::counter_bench_client::counter_bench::io::Inc::decode_reply(
                    CounterBenchProgram::ROUTE_ID_COUNTER_BENCH,
                    payload.as_slice(),
                )
                .unwrap();
                assert_eq!(stress_resp, expected_value);
                expected_value += 1;

                gas
            } else {
                let message_id = service.inc_async().send_one_way().unwrap();
                let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
                // Low-level approach: decoding using generated io module
                let stress_resp = crate::clients::counter_bench_client::counter_bench::io::IncAsync::decode_reply(
                    CounterBenchProgram::ROUTE_ID_COUNTER_BENCH,
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

#[cfg(feature = "gas-profile")]
#[tokio::test]
async fn noop_floor_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();

    for sample in 0..floor_samples() {
        for (key, gas) in noop_floor_test(sample).await {
            benches.entry(key).or_default().push(gas);
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect::<BTreeMap<_, _>>();

    crate::write_gas_profile_summary(&medians, &render_noop_floor_markdown(&medians)).unwrap();
}

#[cfg(feature = "gas-profile")]
#[tokio::test]
async fn minimal_vft_floor_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();

    for sample in 0..floor_samples() {
        for (key, gas) in minimal_vft_floor_test(sample).await {
            benches.entry(key).or_default().push(gas);
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect::<BTreeMap<_, _>>();

    crate::write_gas_profile_summary(&medians, &render_minimal_vft_floor_markdown(&medians))
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
            // Low-level approach: decoding using generated io module
            let stress_resp =
                ping_pong_bench_app::client::ping_pong_service::io::Ping::decode_reply(
                    PingPongProgram::ROUTE_ID_PING_PONG_SERVICE,
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
        // Low-level approach: decoding using generated io module
        let resp = redirect_proxy_client::proxy::io::GetProgramId::decode_reply(
            RedirectProxyClientProgram::ROUTE_ID_PROXY,
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
            // Low-level approach: decoding using generated io module
            let resp = redirect_proxy_client::proxy::io::GetProgramId::decode_reply(
                RedirectProxyClientProgram::ROUTE_ID_PROXY,
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

#[tokio::test]
async fn message_stack_bench() {
    let mut benches: BTreeMap<u32, Vec<u64>> = Default::default();
    let limits = [0u32, 1, 5, 10, 20];

    for _ in 0..100 {
        for &limit in limits.iter() {
            let gas = message_stack_test(limit).await;

            benches.entry(limit).or_default().push(gas);
        }
    }

    for (len, gas_benches) in benches {
        crate::store_bench_data(|bench_data| {
            bench_data.update_message_stack_bench(len, median(gas_benches));
        })
        .unwrap();
    }
}

#[tokio::test]
async fn aggregator_tracker_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();
    let loads = [0, 16, 256, 1024];

    for sample in 0..10 {
        for backend in [
            AggregatorTrackerBackend::BTree,
            AggregatorTrackerBackend::SailsFixed,
        ] {
            for &load in loads.iter() {
                let run = aggregator_tracker_test(backend.clone(), load, sample).await;

                for gas in run.prepare {
                    benches
                        .entry(aggregator_tracker_prepare_key(&backend, load))
                        .or_default()
                        .push(gas);
                }

                for (op, gas) in run.operations {
                    benches
                        .entry(aggregator_tracker_bench_key(&backend, &op, load))
                        .or_default()
                        .push(gas);
                }
            }
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect();

    crate::store_bench_data(|bench_data| {
        bench_data.replace_example_benches(medians);
    })
    .unwrap();
}

#[tokio::test]
async fn storage_million_static_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();

    for backend in [
        MillionStorageBackend::GenericStatic,
        MillionStorageBackend::WatActorStatic,
        MillionStorageBackend::MixedActorStatic,
        MillionStorageBackend::ControlActorStatic,
        MillionStorageBackend::PageLocalActorStatic,
        MillionStorageBackend::GroupedActorPages2,
        MillionStorageBackend::GroupedActorPages4,
        MillionStorageBackend::GroupedActorPages8,
        MillionStorageBackend::GroupedActorPages16,
        MillionStorageBackend::GroupedActorPages32,
        MillionStorageBackend::GroupedActorPages64,
        MillionStorageBackend::GroupedActorPages128,
    ] {
        for sample in 0..STORAGE_MILLION_SAMPLES {
            let run =
                storage_million_static_test(backend.clone(), STORAGE_MILLION_LOAD, sample).await;

            benches
                .entry(storage_million_prepare_key(&backend, STORAGE_MILLION_LOAD))
                .or_default()
                .push(run.prepare_total);

            for (op, gas) in run.operations {
                benches
                    .entry(storage_million_bench_key(
                        &backend,
                        &op,
                        STORAGE_MILLION_LOAD,
                    ))
                    .or_default()
                    .push(gas);
            }

            for (op, gas) in run.batch_operations {
                benches
                    .entry(storage_million_batch_key(
                        &backend,
                        &op,
                        STORAGE_MILLION_BATCH_COUNT,
                        STORAGE_MILLION_LOAD,
                    ))
                    .or_default()
                    .push(gas);
            }
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect();

    crate::store_bench_data(|bench_data| {
        bench_data.replace_storage_million_benches(medians);
    })
    .unwrap();
}

#[tokio::test]
async fn vft_million_transfer_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();
    #[cfg(feature = "gas-profile")]
    let mut profile_benches: BTreeMap<String, Vec<u64>> = Default::default();

    let backends = filtered_million_vft_backends(
        "SAILS_STORAGE_MILLION_VFT_BACKENDS",
        &[
            MillionVftBackend::GenericStatic,
            MillionVftBackend::GenericStaticFused,
            MillionVftBackend::GenericStaticFast,
            MillionVftBackend::WatActorStatic,
            MillionVftBackend::MixedActorStatic,
            MillionVftBackend::MixedActorFast,
            MillionVftBackend::TagActorStatic,
            MillionVftBackend::TagU64ActorStatic,
            MillionVftBackend::ControlActorStatic,
            MillionVftBackend::PageLocalActorStatic,
            MillionVftBackend::InlineOwnerAccountU256,
        ],
    );

    #[cfg(feature = "gas-profile")]
    for sample in 0..STORAGE_MILLION_SAMPLES {
        for (key, gas) in vft_framework_overhead_test(sample).await {
            profile_benches.entry(key).or_default().push(gas);
        }
    }

    for backend in backends {
        for sample in 0..STORAGE_MILLION_SAMPLES {
            let run =
                storage_million_vft_transfer_test(backend.clone(), STORAGE_MILLION_LOAD, sample)
                    .await;

            benches
                .entry(storage_million_vft_prepare_key(
                    &backend,
                    STORAGE_MILLION_LOAD,
                ))
                .or_default()
                .push(run.prepare_total);

            for (op, gas) in run.operations {
                benches
                    .entry(storage_million_vft_key(&backend, &op, STORAGE_MILLION_LOAD))
                    .or_default()
                    .push(gas);
            }
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect::<BTreeMap<_, _>>();

    #[cfg(not(feature = "gas-profile"))]
    crate::store_bench_data(|bench_data| {
        for (key, value) in medians.clone() {
            bench_data.update_storage_million_bench(key, value);
        }
    })
    .unwrap();

    #[cfg(feature = "gas-profile")]
    {
        let mut profile_medians = medians.clone();
        profile_medians.extend(
            profile_benches
                .into_iter()
                .map(|(key, gas_benches)| (key, median(gas_benches))),
        );
        crate::write_gas_profile_summary(
            &profile_medians,
            &render_vft_comparison_markdown(&profile_medians),
        )
        .unwrap();
    }
}

#[tokio::test]
async fn vft_million_real_cost_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();
    #[cfg(feature = "gas-profile")]
    let mut profile_benches: BTreeMap<String, Vec<u64>> = Default::default();

    let backends = filtered_million_vft_backends(
        "SAILS_STORAGE_MILLION_VFT_BACKENDS",
        &[
            MillionVftBackend::GenericStatic,
            MillionVftBackend::GenericStaticFused,
            MillionVftBackend::GenericStaticFast,
            MillionVftBackend::WatActorStatic,
            MillionVftBackend::MixedActorStatic,
            MillionVftBackend::MixedActorFast,
            MillionVftBackend::TagActorStatic,
            MillionVftBackend::TagU64ActorStatic,
            MillionVftBackend::PageLocalActorStatic,
            MillionVftBackend::ControlActorStatic,
            MillionVftBackend::GroupedActorPages64,
            MillionVftBackend::GroupedActorPages128,
            MillionVftBackend::InlineOwnerAccountU256,
        ],
    );

    #[cfg(feature = "gas-profile")]
    for sample in 0..STORAGE_MILLION_SAMPLES {
        for (key, gas) in vft_framework_overhead_test(sample).await {
            profile_benches.entry(key).or_default().push(gas);
        }
    }

    for backend in backends {
        for sample in 0..STORAGE_MILLION_SAMPLES {
            let run =
                storage_million_vft_real_cost_test(backend.clone(), STORAGE_MILLION_LOAD, sample)
                    .await;

            benches
                .entry(storage_million_vft_real_prepare_key(
                    &backend,
                    STORAGE_MILLION_LOAD,
                ))
                .or_default()
                .push(run.prepare_total);

            for (op, gas) in run.operations {
                benches
                    .entry(storage_million_vft_real_key(
                        &backend,
                        op,
                        STORAGE_MILLION_LOAD,
                    ))
                    .or_default()
                    .push(gas);
            }
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect::<BTreeMap<_, _>>();

    #[cfg(not(feature = "gas-profile"))]
    crate::store_bench_data(|bench_data| {
        for (key, value) in medians.clone() {
            bench_data.update_storage_million_bench(key, value);
        }
    })
    .unwrap();

    #[cfg(feature = "gas-profile")]
    {
        let mut profile_medians = medians.clone();
        profile_medians.extend(
            profile_benches
                .into_iter()
                .map(|(key, gas_benches)| (key, median(gas_benches))),
        );
        crate::write_gas_profile_summary(
            &profile_medians,
            &render_vft_comparison_markdown(&profile_medians),
        )
        .unwrap();
    }
}

#[tokio::test]
#[ignore = "BTreeMap/HashMap 1M VFT prepare may exhaust wasm heap; run manually for feasibility evidence"]
async fn vft_million_dynamic_baseline_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();

    for backend in [MillionVftBackend::BTree, MillionVftBackend::HashMap] {
        for sample in 0..STORAGE_MILLION_SAMPLES {
            let run =
                storage_million_vft_transfer_test(backend.clone(), STORAGE_MILLION_LOAD, sample)
                    .await;

            benches
                .entry(storage_million_vft_prepare_key(
                    &backend,
                    STORAGE_MILLION_LOAD,
                ))
                .or_default()
                .push(run.prepare_total);

            for (op, gas) in run.operations {
                benches
                    .entry(storage_million_vft_key(&backend, &op, STORAGE_MILLION_LOAD))
                    .or_default()
                    .push(gas);
            }
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect::<BTreeMap<_, _>>();

    crate::store_bench_data(|bench_data| {
        for (key, value) in medians {
            bench_data.update_storage_million_bench(key, value);
        }
    })
    .unwrap();
}

#[tokio::test]
async fn storage_stress_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();
    let loads = [0, 16, 256, 1024];
    let wasm_path = "../target/wasm32-gear/release/storage_stress.opt.wasm";
    assert_storage_stress_wasm_static_memory_layout(wasm_path);

    for sample in 0..10 {
        for backend in [
            StorageBackend::HashMap,
            StorageBackend::Fixed,
            StorageBackend::RawStatic,
            StorageBackend::SailsFixed,
            StorageBackend::SailsStatic,
        ] {
            for map in [StorageMap::Balance, StorageMap::Allowance] {
                for &load in loads.iter() {
                    let run = storage_stress_test(backend.clone(), map.clone(), load, sample).await;

                    for gas in run.prepare {
                        benches
                            .entry(storage_prepare_key(&backend, &map, load))
                            .or_default()
                            .push(gas);
                    }

                    for (op, gas) in run.operations {
                        benches
                            .entry(storage_bench_key(&backend, &map, &op, load))
                            .or_default()
                            .push(gas);
                    }
                }
            }
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect();

    crate::store_bench_data(|bench_data| {
        bench_data.replace_storage_benches(medians);
    })
    .unwrap();
}

#[tokio::test]
async fn vft_storage_transfer_bench() {
    let mut benches: BTreeMap<String, Vec<u64>> = Default::default();
    #[cfg(feature = "gas-profile")]
    let mut profile_benches: BTreeMap<String, Vec<u64>> = Default::default();
    let loads = [16, 64, 128, 256, 1024];
    let wasm_path = "../target/wasm32-gear/release/vft_stress.opt.wasm";
    assert_vft_stress_wasm_static_memory_layout(wasm_path);

    for sample in 0..10 {
        #[cfg(feature = "gas-profile")]
        for (key, gas) in vft_framework_overhead_test(sample).await {
            profile_benches.entry(key).or_default().push(gas);
        }

        for backend in [
            VftStorageBackend::BTree,
            VftStorageBackend::HashMap,
            VftStorageBackend::SailsFixed,
            VftStorageBackend::SailsStatic,
            VftStorageBackend::SailsStaticFast,
        ] {
            for &load in loads.iter() {
                let run = vft_storage_transfer_test(backend.clone(), load, sample).await;

                for gas in run.prepare {
                    benches
                        .entry(vft_prepare_key(&backend, load))
                        .or_default()
                        .push(gas);
                }

                for (op, gas) in run.operations {
                    benches
                        .entry(vft_named_key(&backend, &op, load))
                        .or_default()
                        .push(gas);
                }
            }
        }
    }

    let medians = benches
        .into_iter()
        .map(|(key, gas_benches)| (key, median(gas_benches)))
        .collect::<BTreeMap<_, _>>();

    #[cfg(not(feature = "gas-profile"))]
    crate::store_bench_data(|bench_data| {
        for (key, value) in medians.clone() {
            bench_data.update_storage_bench(key, value);
        }
    })
    .unwrap();

    #[cfg(feature = "gas-profile")]
    {
        let mut profile_medians = medians.clone();
        profile_medians.extend(
            profile_benches
                .into_iter()
                .map(|(key, gas_benches)| (key, median(gas_benches))),
        );
        crate::write_gas_profile_summary(
            &profile_medians,
            &render_vft_comparison_markdown(&profile_medians),
        )
        .unwrap();
    }
}

async fn alloc_stress_test(n: u32) -> (usize, u64) {
    // Path taken from the .binpath file
    let wasm_path = "../target/wasm32-gear/release/alloc_stress.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| AllocStressCtors::new_for_bench(d)).await;

    let mut service = program.alloc_stress();
    let message_id = service.alloc_stress(n).send_one_way().unwrap();
    let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
    // Low-level approach: decoding using generated io module
    let stress_resp =
        crate::clients::alloc_stress_client::alloc_stress::io::AllocStress::decode_reply(
            AllocStressProgram::ROUTE_ID_ALLOC_STRESS,
            payload.as_slice(),
        )
        .unwrap();

    let expected_len = alloc_stress::fibonacci_sum(n) as usize;
    assert_eq!(stress_resp.inner.len(), expected_len);

    (expected_len, gas)
}

async fn message_stack_test(limit: u32) -> u64 {
    use ping_pong_stack::client::{PingPongStack as _, ping_pong_stack::PingPongStack as _};
    // Path taken from the .binpath file
    let wasm_path = "../target/wasm32-gear/release/ping_pong_stack.opt.wasm";
    let env = create_env();
    let code_id = env.system().submit_local_code_file(wasm_path);
    let program = deploy_code_for_bench(&env, code_id, |d| {
        ping_pong_stack::client::PingPongStackCtors::create_ping(d, code_id)
    })
    .await;

    let message_id = program
        .ping_pong_stack()
        .start(limit)
        .send_one_way()
        .unwrap();
    let block_res = env.system().run_next_block();
    assert!(block_res.succeed.contains(&message_id));
    assert_eq!(block_res.gas_burned.len(), (limit * 2 + 1) as usize);

    let gas = block_res.gas_burned.values().sum();
    gas
}

struct StorageStressRun {
    prepare: Vec<u64>,
    operations: Vec<(StorageOp, u64)>,
}

struct VftTransferRun {
    prepare: Vec<u64>,
    operations: Vec<(String, u64)>,
}

struct AggregatorTrackerRun {
    prepare: Vec<u64>,
    operations: Vec<(AggregatorTrackerOp, u64)>,
}

struct StorageMillionRun {
    prepare_total: u64,
    operations: Vec<(MillionStorageOp, u64)>,
    batch_operations: Vec<(MillionStorageOp, u64)>,
}

struct MillionVftTransferRun {
    prepare_total: u64,
    operations: Vec<(MillionVftTransferOp, u64)>,
}

struct MillionVftRealCostRun {
    prepare_total: u64,
    operations: Vec<(&'static str, u64)>,
}

async fn storage_million_static_test(
    backend: MillionStorageBackend,
    load: u32,
    sample: u32,
) -> StorageMillionRun {
    let wasm_path = "../target/wasm32-gear/release/storage_million.opt.wasm";
    let env = create_env();
    let program =
        deploy_for_bench(&env, wasm_path, |d| StorageMillionCtors::new_for_bench(d)).await;
    let mut service = program.storage_million();

    let mut start = 0;
    let mut prepare_total = 0u64;
    while start < load {
        let len = (load - start).min(STORAGE_MILLION_PREPARE_CHUNK);
        let message_id = service
            .prepare_chunk(backend.clone(), start, len)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        prepare_total += gas;
        start += len;

        let result =
            crate::clients::storage_million_client::storage_million::io::PrepareChunk::decode_reply(
                StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
                payload.as_slice(),
            )
            .unwrap();
        assert_eq!(result.len, start);
    }

    let mut current_len = load;
    let mut operations = Vec::with_capacity(storage_million_ops().len());
    for op in storage_million_ops() {
        let seed = storage_million_seed_for_op(&op, load, sample);
        let message_id = service
            .bench(backend.clone(), op.clone(), seed)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        let result =
            crate::clients::storage_million_client::storage_million::io::Bench::decode_reply(
                StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
                payload.as_slice(),
            )
            .unwrap();

        assert_storage_million_result(&op, seed, &mut current_len, &result);
        operations.push((op, gas));
    }

    let mut batch_operations = Vec::with_capacity(storage_million_ops().len());
    for op in storage_million_ops() {
        let start_seed = storage_million_batch_seed_for_op(&op, sample);
        let message_id = service
            .bench_many(
                backend.clone(),
                op.clone(),
                start_seed,
                STORAGE_MILLION_BATCH_COUNT,
            )
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        let result =
            crate::clients::storage_million_client::storage_million::io::BenchMany::decode_reply(
                StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
                payload.as_slice(),
            )
            .unwrap();

        assert_storage_million_batch_result(
            &op,
            start_seed,
            STORAGE_MILLION_BATCH_COUNT,
            &mut current_len,
            &result,
        );
        batch_operations.push((op, gas));
    }

    StorageMillionRun {
        prepare_total,
        operations,
        batch_operations,
    }
}

async fn storage_million_vft_transfer_test(
    backend: MillionVftBackend,
    load: u32,
    sample: u32,
) -> MillionVftTransferRun {
    let wasm_path = "../target/wasm32-gear/release/storage_million.opt.wasm";
    let env = create_env();
    let program =
        deploy_for_bench(&env, wasm_path, |d| StorageMillionCtors::new_for_bench(d)).await;
    let mut service = program.storage_million();

    let mut start = 0;
    let mut prepare_total = 0u64;
    while start < load {
        let len = (load - start).min(STORAGE_MILLION_PREPARE_CHUNK);
        let message_id = service
            .prepare_vft_chunk(backend.clone(), start, len)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        prepare_total += gas;
        start += len;

        let result = crate::clients::storage_million_client::storage_million::io::PrepareVftChunk::decode_reply(
            StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
            payload.as_slice(),
        )
        .unwrap();
        assert_eq!(result.balance_len, start);
        assert_eq!(result.allowance_len, start);
    }

    let mut operations = Vec::with_capacity(storage_million_vft_ops().len());
    for op in storage_million_vft_ops() {
        let seed = storage_million_vft_seed_for_op(&op, load, sample);
        let message_id = service
            .bench_vft_transfer(backend.clone(), op.clone(), seed)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        let result = crate::clients::storage_million_client::storage_million::io::BenchVftTransfer::decode_reply(
            StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
            payload.as_slice(),
        )
        .unwrap();

        assert_storage_million_vft_result(&op, seed, load, &result);
        operations.push((op, gas));
    }

    MillionVftTransferRun {
        prepare_total,
        operations,
    }
}

async fn storage_million_vft_real_cost_test(
    backend: MillionVftBackend,
    load: u32,
    sample: u32,
) -> MillionVftRealCostRun {
    let wasm_path = "../target/wasm32-gear/release/storage_million.opt.wasm";
    let env = create_env();
    let program =
        deploy_for_bench(&env, wasm_path, |d| StorageMillionCtors::new_for_bench(d)).await;
    let mut service = program.storage_million();

    let mut start = 0;
    let mut prepare_total = 0u64;
    while start < load {
        let len = (load - start).min(STORAGE_MILLION_PREPARE_CHUNK);
        let message_id = service
            .prepare_vft_chunk(backend.clone(), start, len)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        prepare_total += gas;
        start += len;

        let result = crate::clients::storage_million_client::storage_million::io::PrepareVftChunk::decode_reply(
            StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
            payload.as_slice(),
        )
        .unwrap();
        assert_eq!(result.balance_len, start);
        assert_eq!(result.allowance_len, start);
    }

    let mut operations = Vec::with_capacity(7);

    let transfer_seed = 20_000 + sample;
    let message_id = service
        .bench_vft_transfer_bool(
            backend.clone(),
            MillionVftTransferOp::Transfer,
            transfer_seed,
        )
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            storage_million_vft_real_key(&backend, "transfer_bool", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let transferred = crate::clients::storage_million_client::storage_million::io::BenchVftTransferBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(transferred);
    operations.push(("transfer_bool", gas));

    let fresh_seed = 40_000 + sample;
    let message_id = service
        .bench_vft_transfer_fresh_bool(backend.clone(), fresh_seed)
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            storage_million_vft_real_key(&backend, "transfer_fresh_bool", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let transferred = crate::clients::storage_million_client::storage_million::io::BenchVftTransferFreshBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(transferred);
    operations.push(("transfer_fresh_bool", gas));

    let transfer_from_seed = 30_000 + sample;
    let message_id = service
        .bench_vft_transfer_bool(
            backend.clone(),
            MillionVftTransferOp::TransferFrom,
            transfer_from_seed,
        )
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            storage_million_vft_real_key(&backend, "transfer_from_bool", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let transferred = crate::clients::storage_million_client::storage_million::io::BenchVftTransferBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(transferred);
    operations.push(("transfer_from_bool", gas));

    let owner_seed = 50_000 + sample;
    let spender_seed = load + 60_000 + sample;
    let message_id = service
        .bench_vft_approve_bool(backend.clone(), owner_seed, spender_seed)
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            storage_million_vft_real_key(&backend, "approve_fresh_bool", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let approved = crate::clients::storage_million_client::storage_million::io::BenchVftApproveBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(approved);
    operations.push(("approve_fresh_bool", gas));

    let owner_seed = 70_000 + sample;
    let spender_seed = load + 170_001 + sample;
    let message_id = service
        .bench_vft_approve_bool(backend.clone(), owner_seed, spender_seed)
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            storage_million_vft_real_key(&backend, "approve_second_bool", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let approved = crate::clients::storage_million_client::storage_million::io::BenchVftApproveBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(approved);
    operations.push(("approve_second_bool", gas));

    let spender_seed = load + 170_002 + sample;
    let message_id = service
        .bench_vft_approve_bool(backend.clone(), owner_seed, spender_seed)
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            storage_million_vft_real_key(&backend, "approve_overflow_third_bool", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let approved = crate::clients::storage_million_client::storage_million::io::BenchVftApproveBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(approved);
    operations.push(("approve_overflow_third_bool", gas));

    let owner_seed = 80_000 + sample;
    let inline_second_spender_seed = load + 180_001 + sample;
    let overflow_spender_seed = load + 180_002 + sample;
    let message_id = service
        .bench_vft_approve_bool(backend.clone(), owner_seed, inline_second_spender_seed)
        .send_one_way()
        .unwrap();
    let (payload, _) = extract_reply_and_gas(env.system(), message_id);
    let approved = crate::clients::storage_million_client::storage_million::io::BenchVftApproveBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(approved);
    let message_id = service
        .bench_vft_approve_bool(backend.clone(), owner_seed, overflow_spender_seed)
        .send_one_way()
        .unwrap();
    let (payload, _) = extract_reply_and_gas(env.system(), message_id);
    let approved = crate::clients::storage_million_client::storage_million::io::BenchVftApproveBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(approved);
    let message_id = service
        .bench_vft_transfer_from_spender_bool(backend.clone(), owner_seed, overflow_spender_seed)
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            storage_million_vft_real_key(&backend, "transfer_from_overflow_bool", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let transferred = crate::clients::storage_million_client::storage_million::io::BenchVftTransferFromSpenderBool::decode_reply(
        StorageMillionProgram::ROUTE_ID_STORAGE_MILLION,
        payload.as_slice(),
    )
    .unwrap();
    assert!(transferred);
    operations.push(("transfer_from_overflow_bool", gas));

    MillionVftRealCostRun {
        prepare_total,
        operations,
    }
}

async fn aggregator_tracker_test(
    backend: AggregatorTrackerBackend,
    load: u32,
    sample: u32,
) -> AggregatorTrackerRun {
    let env = create_env();
    let code_id = env.system().submit_code(aggregator_app::WASM_BINARY);
    let program = deploy_code_for_bench(&env, code_id, |d| {
        AggregatorClientCtors::new_with_tracker(d, ActorId::zero(), backend.clone())
    })
    .await;
    let mut service = program.aggregator();

    let ops = aggregator_tracker_ops_for_load(load);
    let mut prepare = Vec::with_capacity(ops.len());
    let mut operations = Vec::with_capacity(ops.len());

    for op in ops {
        let message_id = service.prepare_tracker(load).send_one_way().unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        prepare.push(gas);
        let prep = aggregator_client::aggregator::io::PrepareTracker::decode_reply(
            AggregatorClientProgram::ROUTE_ID_AGGREGATOR,
            payload.as_slice(),
        )
        .unwrap();
        assert_eq!(prep.len, load);

        let seed = aggregator_tracker_seed_for_op(&op, load, sample);
        let message_id = service
            .bench_tracker(op.clone(), seed)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        let result = aggregator_client::aggregator::io::BenchTracker::decode_reply(
            AggregatorClientProgram::ROUTE_ID_AGGREGATOR,
            payload.as_slice(),
        )
        .unwrap();

        assert_aggregator_tracker_result(&op, load, &result);
        operations.push((op, gas));
    }

    AggregatorTrackerRun {
        prepare,
        operations,
    }
}

fn aggregator_tracker_ops_for_load(load: u32) -> Vec<AggregatorTrackerOp> {
    if load == 0 {
        vec![
            AggregatorTrackerOp::InsertFresh,
            AggregatorTrackerOp::ListStatuses,
        ]
    } else {
        vec![
            AggregatorTrackerOp::InsertFresh,
            AggregatorTrackerOp::UpdateExisting,
            AggregatorTrackerOp::ReadExisting,
            AggregatorTrackerOp::ListStatuses,
        ]
    }
}

fn aggregator_tracker_seed_for_op(op: &AggregatorTrackerOp, load: u32, sample: u32) -> u32 {
    match op {
        AggregatorTrackerOp::InsertFresh => 10_000 + load + sample,
        AggregatorTrackerOp::UpdateExisting | AggregatorTrackerOp::ReadExisting => {
            1 + (sample % load)
        }
        AggregatorTrackerOp::ListStatuses => 0,
    }
}

fn assert_aggregator_tracker_result(
    op: &AggregatorTrackerOp,
    load: u32,
    result: &AggregatorTrackerBenchResult,
) {
    match op {
        AggregatorTrackerOp::InsertFresh => {
            assert_eq!(result.len, load + 1);
            assert_eq!(result.status, Some(AggregatorOpStatus::Started));
            assert!(!result.existed);
        }
        AggregatorTrackerOp::UpdateExisting => {
            assert_eq!(result.len, load);
            assert_eq!(result.status, Some(AggregatorOpStatus::Finalized));
            assert!(result.existed);
        }
        AggregatorTrackerOp::ReadExisting => {
            assert_eq!(result.len, load);
            assert_eq!(result.status, Some(AggregatorOpStatus::Started));
            assert!(result.existed);
        }
        AggregatorTrackerOp::ListStatuses => {
            assert_eq!(result.len, load);
            if load == 0 {
                assert_eq!(result.status, None);
                assert!(!result.existed);
            } else {
                assert_eq!(result.status, Some(AggregatorOpStatus::Started));
                assert!(result.existed);
            }
        }
    }
}

async fn storage_stress_test(
    backend: StorageBackend,
    map: StorageMap,
    load: u32,
    sample: u32,
) -> StorageStressRun {
    let wasm_path = "../target/wasm32-gear/release/storage_stress.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| StorageStressCtors::new_for_bench(d)).await;
    let mut service = program.storage_stress();

    let ops = storage_ops_for_load(load);
    let mut prepare = Vec::with_capacity(ops.len());
    let mut operations = Vec::with_capacity(ops.len());

    for op in ops {
        let message_id = service
            .prepare(backend.clone(), map.clone(), load)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        prepare.push(gas);
        let prep =
            crate::clients::storage_stress_client::storage_stress::io::Prepare::decode_reply(
                StorageStressProgram::ROUTE_ID_STORAGE_STRESS,
                payload.as_slice(),
            )
            .unwrap();
        assert_eq!(prep.len, load);

        let seed = storage_seed_for_op(&op, load, sample);
        let message_id = service
            .bench(backend.clone(), map.clone(), op.clone(), seed)
            .send_one_way()
            .unwrap();
        let (payload, gas) = extract_reply_and_gas(env.system(), message_id);
        let result =
            crate::clients::storage_stress_client::storage_stress::io::Bench::decode_reply(
                StorageStressProgram::ROUTE_ID_STORAGE_STRESS,
                payload.as_slice(),
            )
            .unwrap();

        assert_storage_result(&op, load, seed, &result);
        operations.push((op, gas));
    }

    StorageStressRun {
        prepare,
        operations,
    }
}

#[cfg(feature = "gas-profile")]
async fn noop_floor_test(sample: u32) -> Vec<(String, u64)> {
    let mut operations = Vec::with_capacity(6);

    if floor_row_enabled("noop_wat_raw") {
        operations.push(noop_wat_raw_floor_test(sample).await);
    }
    if floor_row_enabled("noop_wat_sails_wire") {
        operations.push(noop_wat_sails_wire_floor_test(sample).await);
    }
    if floor_row_enabled("noop_gstd") {
        operations.push(noop_gstd_floor_test(sample).await);
    }
    if floor_row_enabled("noop_sails") {
        operations.push(noop_sails_floor_test(sample).await);
    }

    let vft_wasm_path = "../target/wasm32-gear/release/vft_stress.opt.wasm";
    if Path::new(vft_wasm_path).exists()
        && (floor_row_enabled("vft_framework_noop") || floor_row_enabled("vft_framework_echo_args"))
    {
        operations.extend(vft_framework_overhead_test(sample).await);
    }

    operations
}

#[cfg(feature = "gas-profile")]
async fn noop_wat_raw_floor_test(sample: u32) -> (String, u64) {
    let env = create_env();
    let program = GtestProgramBuilder::from_binary(::noop_wat::RAW_REPLY_WASM).build(env.system());

    let init_id = program.send_bytes(DEFAULT_USER_ALICE, Vec::new());
    let init_res = env.system().run_next_block();
    assert!(
        init_res.succeed.contains(&init_id),
        "noop-wat raw init failed: {init_res:?}"
    );

    let message_id = program.send_bytes(DEFAULT_USER_ALICE, Vec::new());
    let profile_case = (sample == 0).then(|| format!("noop_wat_raw_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    assert_eq!(payload, vec![1u8]);

    ("noop_wat_raw".to_owned(), gas)
}

#[cfg(feature = "gas-profile")]
async fn noop_wat_sails_wire_floor_test(sample: u32) -> (String, u64) {
    let env = create_env();
    let program =
        GtestProgramBuilder::from_binary(::noop_wat::SAILS_WIRE_REPLY_WASM).build(env.system());

    let init_id = program.send_bytes(DEFAULT_USER_ALICE, Vec::new());
    let init_res = env.system().run_next_block();
    assert!(
        init_res.succeed.contains(&init_id),
        "noop-wat Sails-wire init failed: {init_res:?}"
    );

    let message_id = program.send_bytes(DEFAULT_USER_ALICE, noop_sails_payload());
    let profile_case = (sample == 0).then(|| format!("noop_wat_sails_wire_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let ok = crate::clients::noop_sails_client::noop_sails::io::Noop::decode_reply(
        NoopSailsProgram::ROUTE_ID_NOOP_SAILS,
        payload.as_slice(),
    )
    .unwrap();
    assert!(ok);

    ("noop_wat_sails_wire".to_owned(), gas)
}

#[cfg(feature = "gas-profile")]
async fn noop_gstd_floor_test(sample: u32) -> (String, u64) {
    let _ = ::noop_gstd::PROGRAM_NAME;
    let wasm_path = "../target/wasm32-gear/release/noop_gstd.opt.wasm";
    let env = create_env();
    let program = GtestProgram::from_file(env.system(), wasm_path);

    let init_id = program.send_bytes(DEFAULT_USER_ALICE, Vec::new());
    let init_res = env.system().run_next_block();
    assert!(
        init_res.succeed.contains(&init_id),
        "noop-gstd init failed: {init_res:?}"
    );

    let message_id = program.send_bytes(DEFAULT_USER_ALICE, Vec::new());
    let profile_case = (sample == 0).then(|| format!("noop_gstd_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    assert_eq!(payload, vec![1u8]);

    ("noop_gstd".to_owned(), gas)
}

#[cfg(feature = "gas-profile")]
async fn noop_sails_floor_test(sample: u32) -> (String, u64) {
    let wasm_path = "../target/wasm32-gear/release/noop_sails.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| NoopSailsCtors::new_for_bench(d)).await;
    let mut service = program.noop_sails();

    let message_id = service.noop().send_one_way().unwrap();
    let profile_case = (sample == 0).then(|| format!("noop_sails_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let ok = crate::clients::noop_sails_client::noop_sails::io::Noop::decode_reply(
        NoopSailsProgram::ROUTE_ID_NOOP_SAILS,
        payload.as_slice(),
    )
    .unwrap();
    assert!(ok);

    ("noop_sails".to_owned(), gas)
}

#[cfg(feature = "gas-profile")]
async fn vft_framework_overhead_test(sample: u32) -> Vec<(String, u64)> {
    let wasm_path = "../target/wasm32-gear/release/vft_stress.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| VftStressCtors::new_for_bench(d)).await;
    let mut service = program.vft_stress();
    let mut operations = Vec::with_capacity(2);

    let message_id = service.bench_noop().send_one_way().unwrap();
    let profile_case = (sample == 0).then(|| format!("vft_framework_noop_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let ok = crate::clients::vft_stress_client::vft_stress::io::BenchNoop::decode_reply(
        VftStressProgram::ROUTE_ID_VFT_STRESS,
        payload.as_slice(),
    )
    .unwrap();
    assert!(ok);
    if floor_row_enabled("vft_framework_noop") {
        operations.push(("vft_framework_noop".to_owned(), gas));
    }

    let message_id = service
        .bench_echo_vft_args(
            VftStorageBackend::SailsStatic,
            VftTransferOp::Transfer,
            sample + 1,
        )
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| format!("vft_framework_echo_args_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let ok = crate::clients::vft_stress_client::vft_stress::io::BenchEchoVftArgs::decode_reply(
        VftStressProgram::ROUTE_ID_VFT_STRESS,
        payload.as_slice(),
    )
    .unwrap();
    assert!(ok);
    if floor_row_enabled("vft_framework_echo_args") {
        operations.push(("vft_framework_echo_args".to_owned(), gas));
    }

    operations
}

#[cfg(feature = "gas-profile")]
async fn minimal_vft_floor_test(sample: u32) -> Vec<(String, u64)> {
    let mut operations = Vec::with_capacity(8);

    if floor_row_enabled("minimal_vft_sails_approve") {
        operations.push(minimal_vft_sails_approve_test(sample).await);
    }
    if floor_row_enabled("minimal_vft_sails_transfer") {
        operations.push(minimal_vft_sails_transfer_test(sample).await);
    }
    if floor_row_enabled("minimal_vft_sails_transfer_from") {
        operations.push(minimal_vft_sails_transfer_from_test(sample).await);
    }
    if floor_row_enabled("minimal_vft_hot_approve") {
        operations.push(minimal_vft_hot_test("approve", HOT_VFT_APPROVE_ENTRY, sample).await);
    }
    if floor_row_enabled("minimal_vft_hot_transfer") {
        operations.push(minimal_vft_hot_test("transfer", HOT_VFT_TRANSFER_ENTRY, sample).await);
    }
    if floor_row_enabled("minimal_vft_hot_transfer_from") {
        operations
            .push(minimal_vft_hot_test("transfer_from", HOT_VFT_TRANSFER_FROM_ENTRY, sample).await);
    }

    let vft_wasm_path = "../target/wasm32-gear/release/vft_stress.opt.wasm";
    if Path::new(vft_wasm_path).exists()
        && (floor_row_enabled("vft_framework_noop") || floor_row_enabled("vft_framework_echo_args"))
    {
        operations.extend(vft_framework_overhead_test(sample).await);
    }

    operations
}

#[cfg(feature = "gas-profile")]
async fn minimal_vft_sails_approve_test(sample: u32) -> (String, u64) {
    let wasm_path = "../target/wasm32-gear/release/minimal_vft_sails.opt.wasm";
    let env = create_env();
    let program =
        deploy_for_bench(&env, wasm_path, |d| MinimalVftSailsCtors::new_for_bench(d)).await;
    let mut service = program.vft();

    let spender = ActorId::from(2u64);
    let amount = U256::from(10_000u64);
    let message_id = service.approve(spender, amount).send_one_way().unwrap();
    let profile_case = (sample == 0).then(|| format!("minimal_vft_sails_approve_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let ok = crate::clients::minimal_vft_sails_client::vft::io::Approve::decode_reply(
        MinimalVftSailsProgram::ROUTE_ID_VFT,
        payload.as_slice(),
    )
    .unwrap();
    assert!(ok);

    ("minimal_vft_sails_approve".to_owned(), gas)
}

#[cfg(feature = "gas-profile")]
async fn minimal_vft_sails_transfer_test(sample: u32) -> (String, u64) {
    let wasm_path = "../target/wasm32-gear/release/minimal_vft_sails.opt.wasm";
    let env = create_env();
    let program =
        deploy_for_bench(&env, wasm_path, |d| MinimalVftSailsCtors::new_for_bench(d)).await;
    let mut service = program.vft();

    let to = ActorId::from(3u64);
    let amount = U256::from(7u64);
    let message_id = service.transfer(to, amount).send_one_way().unwrap();
    let profile_case = (sample == 0).then(|| format!("minimal_vft_sails_transfer_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let ok = crate::clients::minimal_vft_sails_client::vft::io::Transfer::decode_reply(
        MinimalVftSailsProgram::ROUTE_ID_VFT,
        payload.as_slice(),
    )
    .unwrap();
    assert!(ok);

    ("minimal_vft_sails_transfer".to_owned(), gas)
}

#[cfg(feature = "gas-profile")]
async fn minimal_vft_sails_transfer_from_test(sample: u32) -> (String, u64) {
    let wasm_path = "../target/wasm32-gear/release/minimal_vft_sails.opt.wasm";
    let env = create_env();
    let program =
        deploy_for_bench(&env, wasm_path, |d| MinimalVftSailsCtors::new_for_bench(d)).await;
    let mut service = program.vft();

    let owner = ActorId::from(DEFAULT_USER_ALICE);
    let to = ActorId::from(4u64);
    let amount = U256::from(5u64);
    let message_id = service
        .transfer_from(owner, to, amount)
        .send_one_way()
        .unwrap();
    let profile_case =
        (sample == 0).then(|| format!("minimal_vft_sails_transfer_from_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    let ok = crate::clients::minimal_vft_sails_client::vft::io::TransferFrom::decode_reply(
        MinimalVftSailsProgram::ROUTE_ID_VFT,
        payload.as_slice(),
    )
    .unwrap();
    assert!(ok);

    ("minimal_vft_sails_transfer_from".to_owned(), gas)
}

#[cfg(feature = "gas-profile")]
async fn minimal_vft_hot_test(label: &str, entry_id: u16, sample: u32) -> (String, u64) {
    let wasm_path = "../target/wasm32-gear/release/minimal_vft_hot.opt.wasm";
    let env = create_env();
    let program = GtestProgram::from_file(env.system(), wasm_path);

    let init_id = program.send_bytes(DEFAULT_USER_ALICE, Vec::new());
    let init_res = env.system().run_next_block();
    assert!(
        init_res.succeed.contains(&init_id),
        "minimal-vft-hot init failed: {init_res:?}"
    );

    let payload = match entry_id {
        HOT_VFT_APPROVE_ENTRY => hot_vft_payload(
            entry_id,
            (ActorId::from(2u64), U256::from(10_000u64)).encode(),
        ),
        HOT_VFT_TRANSFER_ENTRY => {
            hot_vft_payload(entry_id, (ActorId::from(3u64), U256::from(7u64)).encode())
        }
        HOT_VFT_TRANSFER_FROM_ENTRY => hot_vft_payload(
            entry_id,
            (
                ActorId::from(DEFAULT_USER_ALICE),
                ActorId::from(4u64),
                U256::from(5u64),
            )
                .encode(),
        ),
        _ => unreachable!("unknown hot VFT entry"),
    };

    let message_id = program.send_bytes(DEFAULT_USER_ALICE, payload);
    let profile_case = (sample == 0).then(|| format!("minimal_vft_hot_{label}_sample{sample}"));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
    assert_sails_bool_reply(entry_id, &payload);

    (format!("minimal_vft_hot_{label}"), gas)
}

async fn vft_storage_transfer_test(
    backend: VftStorageBackend,
    load: u32,
    sample: u32,
) -> VftTransferRun {
    let wasm_path = "../target/wasm32-gear/release/vft_stress.opt.wasm";
    let env = create_env();
    let program = deploy_for_bench(&env, wasm_path, |d| VftStressCtors::new_for_bench(d)).await;
    let mut service = program.vft_stress();

    let ops = vft_transfer_ops();
    let mut prepare = Vec::with_capacity(ops.len() + 2);
    let mut operations = Vec::with_capacity(ops.len() + 2);

    for op in ops {
        let message_id = service
            .prepare_vft(backend.clone(), load)
            .send_one_way()
            .unwrap();
        let profile_case =
            (sample == 0).then(|| format!("{}_sample{sample}", vft_prepare_key(&backend, load)));
        let (payload, gas) =
            extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
        prepare.push(gas);
        let prep = crate::clients::vft_stress_client::vft_stress::io::PrepareVft::decode_reply(
            VftStressProgram::ROUTE_ID_VFT_STRESS,
            payload.as_slice(),
        )
        .unwrap();
        assert_eq!(prep.balance_len, load);
        assert_eq!(prep.allowance_len, load);

        let seed = vft_transfer_seed_for_op(&op, load, sample);
        let message_id = service
            .bench_vft_transfer(backend.clone(), op.clone(), seed)
            .send_one_way()
            .unwrap();
        let profile_case = (sample == 0)
            .then(|| format!("{}_sample{sample}", vft_transfer_key(&backend, &op, load)));
        let (payload, gas) =
            extract_reply_and_gas_profiled(env.system(), message_id, profile_case.as_deref());
        let result =
            crate::clients::vft_stress_client::vft_stress::io::BenchVftTransfer::decode_reply(
                VftStressProgram::ROUTE_ID_VFT_STRESS,
                payload.as_slice(),
            )
            .unwrap();

        assert_vft_transfer_result(&op, load, seed, &result);
        operations.push((vft_transfer_op_name(&op).to_owned(), gas));
    }

    let fresh_prepare_id = service
        .prepare_vft(backend.clone(), load)
        .send_one_way()
        .unwrap();
    let profile_case =
        (sample == 0).then(|| format!("{}_sample{sample}", vft_prepare_key(&backend, load)));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), fresh_prepare_id, profile_case.as_deref());
    prepare.push(gas);
    let prep = crate::clients::vft_stress_client::vft_stress::io::PrepareVft::decode_reply(
        VftStressProgram::ROUTE_ID_VFT_STRESS,
        payload.as_slice(),
    )
    .unwrap();
    assert_eq!(prep.balance_len, load);
    assert_eq!(prep.allowance_len, load);

    let fresh_seed = 1 + (sample % load);
    let fresh_id = service
        .bench_vft_transfer_fresh_bool(backend.clone(), fresh_seed)
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            vft_named_key(&backend, "transfer_fresh", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), fresh_id, profile_case.as_deref());
    let transferred =
        crate::clients::vft_stress_client::vft_stress::io::BenchVftTransferFreshBool::decode_reply(
            VftStressProgram::ROUTE_ID_VFT_STRESS,
            payload.as_slice(),
        )
        .unwrap();
    assert!(transferred);
    operations.push(("transfer_fresh".to_owned(), gas));

    let approve_prepare_id = service
        .prepare_vft(backend.clone(), load)
        .send_one_way()
        .unwrap();
    let profile_case =
        (sample == 0).then(|| format!("{}_sample{sample}", vft_prepare_key(&backend, load)));
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), approve_prepare_id, profile_case.as_deref());
    prepare.push(gas);
    let prep = crate::clients::vft_stress_client::vft_stress::io::PrepareVft::decode_reply(
        VftStressProgram::ROUTE_ID_VFT_STRESS,
        payload.as_slice(),
    )
    .unwrap();
    assert_eq!(prep.balance_len, load);
    assert_eq!(prep.allowance_len, load);

    let owner_seed = 1 + (sample % load);
    let spender_seed = 40_000 + sample;
    let approve_id = service
        .bench_vft_approve_bool(backend.clone(), owner_seed, spender_seed)
        .send_one_way()
        .unwrap();
    let profile_case = (sample == 0).then(|| {
        format!(
            "{}_sample{sample}",
            vft_named_key(&backend, "approve_fresh", load)
        )
    });
    let (payload, gas) =
        extract_reply_and_gas_profiled(env.system(), approve_id, profile_case.as_deref());
    let approved =
        crate::clients::vft_stress_client::vft_stress::io::BenchVftApproveBool::decode_reply(
            VftStressProgram::ROUTE_ID_VFT_STRESS,
            payload.as_slice(),
        )
        .unwrap();
    assert!(approved);
    operations.push(("approve_fresh".to_owned(), gas));

    #[cfg(feature = "gas-profile")]
    if sample == 0 {
        for op in vft_transfer_ops() {
            let prepare_id = service
                .prepare_vft(backend.clone(), load)
                .send_one_way()
                .unwrap();
            let (payload, _) = extract_reply_and_gas(env.system(), prepare_id);
            let prep = crate::clients::vft_stress_client::vft_stress::io::PrepareVft::decode_reply(
                VftStressProgram::ROUTE_ID_VFT_STRESS,
                payload.as_slice(),
            )
            .unwrap();
            assert_eq!(prep.balance_len, load);
            assert_eq!(prep.allowance_len, load);

            let seed = vft_transfer_seed_for_op(&op, load, sample);
            let message_id = service
                .bench_vft_transfer_profile(backend.clone(), op.clone(), seed)
                .send_one_way()
                .unwrap();
            let profile_case = format!(
                "{}_sample{sample}",
                vft_named_key(
                    &backend,
                    &format!("{}_profile", vft_transfer_op_name(&op)),
                    load
                )
            );
            let (payload, gas) =
                extract_reply_and_gas_profiled(env.system(), message_id, Some(&profile_case));
            let profile =
                crate::clients::vft_stress_client::vft_stress::io::BenchVftTransferProfile::decode_reply(
                    VftStressProgram::ROUTE_ID_VFT_STRESS,
                    payload.as_slice(),
                )
                .unwrap();
            assert_vft_transfer_result(&op, load, seed, &profile.result);
            write_vft_phase_profile(&profile_case, gas, &profile);
        }

        let prepare_id = service
            .prepare_vft(backend.clone(), load)
            .send_one_way()
            .unwrap();
        let (payload, _) = extract_reply_and_gas(env.system(), prepare_id);
        let prep = crate::clients::vft_stress_client::vft_stress::io::PrepareVft::decode_reply(
            VftStressProgram::ROUTE_ID_VFT_STRESS,
            payload.as_slice(),
        )
        .unwrap();
        assert_eq!(prep.balance_len, load);
        assert_eq!(prep.allowance_len, load);

        let fresh_seed = 1 + (sample % load);
        let profile_case = format!(
            "{}_sample{sample}",
            vft_named_key(&backend, "transfer_fresh_profile", load)
        );
        let message_id = service
            .bench_vft_transfer_fresh_profile(backend.clone(), fresh_seed)
            .send_one_way()
            .unwrap();
        let (payload, gas) =
            extract_reply_and_gas_profiled(env.system(), message_id, Some(&profile_case));
        let profile =
            crate::clients::vft_stress_client::vft_stress::io::BenchVftTransferFreshProfile::decode_reply(
                VftStressProgram::ROUTE_ID_VFT_STRESS,
                payload.as_slice(),
            )
            .unwrap();
        assert_vft_transfer_result(&VftTransferOp::Transfer, load, fresh_seed, &profile.result);
        write_vft_phase_profile(&profile_case, gas, &profile);

        let prepare_id = service
            .prepare_vft(backend.clone(), load)
            .send_one_way()
            .unwrap();
        let (payload, _) = extract_reply_and_gas(env.system(), prepare_id);
        let prep = crate::clients::vft_stress_client::vft_stress::io::PrepareVft::decode_reply(
            VftStressProgram::ROUTE_ID_VFT_STRESS,
            payload.as_slice(),
        )
        .unwrap();
        assert_eq!(prep.balance_len, load);
        assert_eq!(prep.allowance_len, load);

        let owner_seed = 1 + (sample % load);
        let spender_seed = 40_000 + sample;
        let profile_case = format!(
            "{}_sample{sample}",
            vft_named_key(&backend, "approve_fresh_profile", load)
        );
        let message_id = service
            .bench_vft_approve_profile(backend.clone(), owner_seed, spender_seed)
            .send_one_way()
            .unwrap();
        let (payload, gas) =
            extract_reply_and_gas_profiled(env.system(), message_id, Some(&profile_case));
        let profile =
            crate::clients::vft_stress_client::vft_stress::io::BenchVftApproveProfile::decode_reply(
                VftStressProgram::ROUTE_ID_VFT_STRESS,
                payload.as_slice(),
            )
            .unwrap();
        assert!(profile.result.transferred);
        write_vft_phase_profile(&profile_case, gas, &profile);
    }

    VftTransferRun {
        prepare,
        operations,
    }
}

fn storage_ops_for_load(load: u32) -> Vec<StorageOp> {
    if load == 0 {
        vec![StorageOp::InsertFresh, StorageOp::ReadMissing]
    } else {
        vec![
            StorageOp::InsertFresh,
            StorageOp::UpdateExisting,
            StorageOp::ReadExisting,
            StorageOp::ReadMissing,
            StorageOp::Remove,
        ]
    }
}

fn vft_transfer_ops() -> [VftTransferOp; 2] {
    [VftTransferOp::Transfer, VftTransferOp::TransferFrom]
}

fn storage_seed_for_op(op: &StorageOp, load: u32, sample: u32) -> u32 {
    match op {
        StorageOp::InsertFresh | StorageOp::ReadMissing => 10_000 + load + sample,
        StorageOp::UpdateExisting | StorageOp::ReadExisting | StorageOp::Remove => {
            1 + (sample % load)
        }
    }
}

fn vft_transfer_seed_for_op(_op: &VftTransferOp, load: u32, sample: u32) -> u32 {
    1 + (sample % load)
}

fn assert_storage_result(op: &StorageOp, load: u32, seed: u32, result: &StorageBenchResult) {
    match op {
        StorageOp::InsertFresh => {
            assert_eq!(result.value, storage_value_for_seed(seed));
            assert_eq!(result.len, load + 1);
            assert!(!result.existed);
        }
        StorageOp::UpdateExisting => {
            assert_eq!(result.value, storage_updated_value_for_seed(seed));
            assert_eq!(result.len, load);
            assert!(result.existed);
        }
        StorageOp::ReadExisting => {
            assert_eq!(result.value, storage_value_for_seed(seed));
            assert_eq!(result.len, load);
            assert!(result.existed);
        }
        StorageOp::ReadMissing => {
            assert_eq!(result.value, U256::zero());
            assert_eq!(result.len, load);
            assert!(!result.existed);
        }
        StorageOp::Remove => {
            assert_eq!(result.value, storage_value_for_seed(seed));
            assert_eq!(result.len, load - 1);
            assert!(result.existed);
        }
    }
}

fn assert_vft_transfer_result(
    op: &VftTransferOp,
    load: u32,
    seed: u32,
    result: &VftTransferResult,
) {
    let amount = vft_transfer_amount(seed);
    assert!(result.transferred);
    assert_eq!(result.from_balance, vft_balance_for_seed(seed) - amount);
    assert_eq!(result.to_balance, amount);
    assert_eq!(result.balance_len, load + 1);

    match op {
        VftTransferOp::Transfer => {
            assert_eq!(result.allowance, U256::zero());
            assert_eq!(result.allowance_len, load);
        }
        VftTransferOp::TransferFrom => {
            assert_eq!(result.allowance, vft_allowance_for_seed(seed) - amount);
            assert_eq!(result.allowance_len, load);
        }
    }
}

fn storage_value_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1)
}

fn storage_updated_value_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1_000_001)
}

fn vft_balance_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 1_000_000)
}

fn vft_allowance_for_seed(seed: u32) -> U256 {
    U256::from(seed as u64 + 10_000)
}

fn vft_transfer_amount(seed: u32) -> U256 {
    U256::from((seed % 7) as u64 + 1)
}

fn storage_million_ops() -> [MillionStorageOp; 5] {
    [
        MillionStorageOp::ReadExisting,
        MillionStorageOp::UpdateExisting,
        MillionStorageOp::ReadMissing,
        MillionStorageOp::InsertFresh,
        MillionStorageOp::Remove,
    ]
}

fn storage_million_vft_ops() -> [MillionVftTransferOp; 2] {
    [
        MillionVftTransferOp::Transfer,
        MillionVftTransferOp::TransferFrom,
    ]
}

fn storage_million_seed_for_op(op: &MillionStorageOp, load: u32, sample: u32) -> u32 {
    match op {
        MillionStorageOp::InsertFresh | MillionStorageOp::ReadMissing => 10_000_000 + load + sample,
        MillionStorageOp::UpdateExisting
        | MillionStorageOp::ReadExisting
        | MillionStorageOp::Remove => 1 + (sample % load),
    }
}

fn storage_million_vft_seed_for_op(op: &MillionVftTransferOp, _load: u32, sample: u32) -> u32 {
    match op {
        MillionVftTransferOp::Transfer => 20_000 + sample,
        MillionVftTransferOp::TransferFrom => 30_000 + sample,
    }
}

fn storage_million_batch_seed_for_op(op: &MillionStorageOp, sample: u32) -> u32 {
    let sample_offset = sample * 10_000;
    match op {
        MillionStorageOp::ReadExisting => 20_000 + sample_offset,
        MillionStorageOp::UpdateExisting => 30_000 + sample_offset,
        MillionStorageOp::ReadMissing => 30_000_000 + sample_offset,
        MillionStorageOp::InsertFresh => 40_000_000 + sample_offset,
        MillionStorageOp::Remove => 40_000 + sample_offset,
    }
}

fn assert_storage_million_result(
    op: &MillionStorageOp,
    seed: u32,
    current_len: &mut u32,
    result: &MillionStorageBenchResult,
) {
    match op {
        MillionStorageOp::InsertFresh => {
            *current_len += 1;
            assert_eq!(result.value, storage_value_for_seed(seed));
            assert_eq!(result.len, *current_len);
            assert!(!result.existed);
        }
        MillionStorageOp::UpdateExisting => {
            assert_eq!(result.value, storage_updated_value_for_seed(seed));
            assert_eq!(result.len, *current_len);
            assert!(result.existed);
        }
        MillionStorageOp::ReadExisting => {
            assert_eq!(result.value, storage_value_for_seed(seed));
            assert_eq!(result.len, *current_len);
            assert!(result.existed);
        }
        MillionStorageOp::ReadMissing => {
            assert_eq!(result.value, U256::zero());
            assert_eq!(result.len, *current_len);
            assert!(!result.existed);
        }
        MillionStorageOp::Remove => {
            *current_len -= 1;
            assert_eq!(result.value, storage_updated_value_for_seed(seed));
            assert_eq!(result.len, *current_len);
            assert!(result.existed);
        }
    }
}

fn assert_storage_million_vft_result(
    op: &MillionVftTransferOp,
    seed: u32,
    load: u32,
    result: &MillionVftTransferResult,
) {
    let amount = vft_transfer_amount(seed);
    let to_seed = seed + 1;

    assert!(result.transferred);
    assert_eq!(result.from_balance, vft_balance_for_seed(seed) - amount);
    assert_eq!(result.to_balance, vft_balance_for_seed(to_seed) + amount);
    assert_eq!(result.balance_len, load);
    assert_eq!(result.allowance_len, load);

    match op {
        MillionVftTransferOp::Transfer => assert_eq!(result.allowance, U256::zero()),
        MillionVftTransferOp::TransferFrom => {
            assert_eq!(result.allowance, vft_allowance_for_seed(seed) - amount)
        }
    }
}

fn assert_storage_million_batch_result(
    op: &MillionStorageOp,
    start_seed: u32,
    count: u32,
    current_len: &mut u32,
    result: &MillionStorageBenchResult,
) {
    let last_seed = start_seed + count - 1;
    match op {
        MillionStorageOp::InsertFresh => {
            *current_len += count;
            assert_eq!(result.value, storage_value_for_seed(last_seed));
            assert_eq!(result.len, *current_len);
            assert!(!result.existed);
        }
        MillionStorageOp::UpdateExisting => {
            assert_eq!(result.value, storage_updated_value_for_seed(last_seed));
            assert_eq!(result.len, *current_len);
            assert!(result.existed);
        }
        MillionStorageOp::ReadExisting => {
            assert_eq!(result.value, storage_value_for_seed(last_seed));
            assert_eq!(result.len, *current_len);
            assert!(result.existed);
        }
        MillionStorageOp::ReadMissing => {
            assert_eq!(result.value, U256::zero());
            assert_eq!(result.len, *current_len);
            assert!(!result.existed);
        }
        MillionStorageOp::Remove => {
            *current_len -= count;
            assert_eq!(result.value, storage_value_for_seed(last_seed));
            assert_eq!(result.len, *current_len);
            assert!(result.existed);
        }
    }
}

fn storage_million_bench_key(
    backend: &MillionStorageBackend,
    op: &MillionStorageOp,
    load: u32,
) -> String {
    format!(
        "{}_{}_{}",
        storage_million_backend_name(backend),
        storage_million_op_name(op),
        load
    )
}

fn storage_million_batch_key(
    backend: &MillionStorageBackend,
    op: &MillionStorageOp,
    count: u32,
    load: u32,
) -> String {
    format!(
        "{}_{}_batch{count}_{load}",
        storage_million_backend_name(backend),
        storage_million_op_name(op)
    )
}

fn storage_million_vft_key(
    backend: &MillionVftBackend,
    op: &MillionVftTransferOp,
    load: u32,
) -> String {
    format!(
        "vft_{}_{}_{}",
        storage_million_vft_backend_name(backend),
        storage_million_vft_op_name(op),
        load
    )
}

fn storage_million_vft_real_key(
    backend: &MillionVftBackend,
    op: &'static str,
    load: u32,
) -> String {
    format!(
        "vft_real_{}_{}_{}",
        storage_million_vft_backend_name(backend),
        op,
        load
    )
}

fn storage_million_prepare_key(backend: &MillionStorageBackend, load: u32) -> String {
    format!("{}_prepare_{load}", storage_million_backend_name(backend))
}

fn storage_million_vft_prepare_key(backend: &MillionVftBackend, load: u32) -> String {
    format!(
        "vft_{}_prepare_{load}",
        storage_million_vft_backend_name(backend)
    )
}

fn storage_million_vft_real_prepare_key(backend: &MillionVftBackend, load: u32) -> String {
    format!(
        "vft_real_{}_prepare_{load}",
        storage_million_vft_backend_name(backend)
    )
}

fn storage_million_backend_name(backend: &MillionStorageBackend) -> &'static str {
    match backend {
        MillionStorageBackend::GenericStatic => "static_balance",
        MillionStorageBackend::WatActorStatic => "wat_actor_balance",
        MillionStorageBackend::MixedActorStatic => "mixed_actor_balance",
        MillionStorageBackend::TagActorStatic => "tag_actor_balance",
        MillionStorageBackend::TagU64ActorStatic => "tag_u64_actor_balance",
        MillionStorageBackend::ControlActorStatic => "control_actor_balance",
        MillionStorageBackend::PageLocalActorStatic => "page_local_actor_balance",
        MillionStorageBackend::GroupedActorPages2 => "grouped_actor_balance_pages2",
        MillionStorageBackend::GroupedActorPages4 => "grouped_actor_balance_pages4",
        MillionStorageBackend::GroupedActorPages8 => "grouped_actor_balance_pages8",
        MillionStorageBackend::GroupedActorPages16 => "grouped_actor_balance_pages16",
        MillionStorageBackend::GroupedActorPages32 => "grouped_actor_balance_pages32",
        MillionStorageBackend::GroupedActorPages64 => "grouped_actor_balance_pages64",
        MillionStorageBackend::GroupedActorPages128 => "grouped_actor_balance_pages128",
    }
}

fn storage_million_vft_backend_name(backend: &MillionVftBackend) -> &'static str {
    match backend {
        MillionVftBackend::BTree => "btree",
        MillionVftBackend::HashMap => "hashmap",
        MillionVftBackend::GenericStatic => "static_balance",
        MillionVftBackend::GenericStaticFused => "static_balance_fused",
        MillionVftBackend::GenericStaticFast => "static_balance_fast",
        MillionVftBackend::WatActorStatic => "wat_actor_balance",
        MillionVftBackend::MixedActorStatic => "mixed_actor_balance",
        MillionVftBackend::MixedActorFast => "mixed_actor_balance_fast",
        MillionVftBackend::TagActorStatic => "tag_actor_balance",
        MillionVftBackend::TagU64ActorStatic => "tag_u64_actor_balance",
        MillionVftBackend::ControlActorStatic => "control_actor_balance",
        MillionVftBackend::PageLocalActorStatic => "page_local_actor_balance",
        MillionVftBackend::GroupedActorPages64 => "grouped_actor_balance_pages64",
        MillionVftBackend::GroupedActorPages128 => "grouped_actor_balance_pages128",
        MillionVftBackend::InlineOwnerAccountU256 => "inline_owner_account_u256",
    }
}

fn filtered_million_vft_backends(
    env_key: &str,
    defaults: &[MillionVftBackend],
) -> Vec<MillionVftBackend> {
    let Ok(value) = env::var(env_key) else {
        return defaults.to_vec();
    };
    if value.trim().is_empty() {
        return defaults.to_vec();
    }

    value
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| parse_million_vft_backend(env_key, name))
        .collect()
}

fn parse_million_vft_backend(env_key: &str, name: &str) -> MillionVftBackend {
    match name {
        "btree" => MillionVftBackend::BTree,
        "hashmap" => MillionVftBackend::HashMap,
        "static_balance" | "generic_static" => MillionVftBackend::GenericStatic,
        "static_balance_fused" | "generic_static_fused" => MillionVftBackend::GenericStaticFused,
        "static_balance_fast" | "generic_static_fast" => MillionVftBackend::GenericStaticFast,
        "wat_actor_balance" | "wat_actor_static" => MillionVftBackend::WatActorStatic,
        "mixed_actor_balance" | "mixed_actor_static" => MillionVftBackend::MixedActorStatic,
        "mixed_actor_balance_fast" | "mixed_actor_fast" => MillionVftBackend::MixedActorFast,
        "tag_actor_balance" | "tag_actor_static" => MillionVftBackend::TagActorStatic,
        "tag_u64_actor_balance" | "tag_u64_actor_static" => MillionVftBackend::TagU64ActorStatic,
        "control_actor_balance" | "control_actor_static" => MillionVftBackend::ControlActorStatic,
        "page_local_actor_balance" | "page_local_actor_static" => {
            MillionVftBackend::PageLocalActorStatic
        }
        "grouped_actor_balance_pages64" | "grouped_actor_pages64" => {
            MillionVftBackend::GroupedActorPages64
        }
        "grouped_actor_balance_pages128" | "grouped_actor_pages128" => {
            MillionVftBackend::GroupedActorPages128
        }
        "inline_owner_account_u256"
        | "inline_owner_account"
        | "inline_allowance_u256"
        | "inline_allowance_balance"
        | "inline_allowance" => MillionVftBackend::InlineOwnerAccountU256,
        other => std::panic!("unknown {env_key} backend `{other}`"),
    }
}

fn storage_million_vft_op_name(op: &MillionVftTransferOp) -> &'static str {
    match op {
        MillionVftTransferOp::Transfer => "transfer",
        MillionVftTransferOp::TransferFrom => "transfer_from",
    }
}

fn storage_million_op_name(op: &MillionStorageOp) -> &'static str {
    match op {
        MillionStorageOp::InsertFresh => "insert_fresh",
        MillionStorageOp::UpdateExisting => "update_existing",
        MillionStorageOp::ReadExisting => "read_existing",
        MillionStorageOp::ReadMissing => "read_missing",
        MillionStorageOp::Remove => "remove",
    }
}

fn aggregator_tracker_bench_key(
    backend: &AggregatorTrackerBackend,
    op: &AggregatorTrackerOp,
    load: u32,
) -> String {
    format!(
        "aggregator_{}_{}_{}",
        aggregator_tracker_backend_name(backend),
        aggregator_tracker_op_name(op),
        load
    )
}

fn aggregator_tracker_prepare_key(backend: &AggregatorTrackerBackend, load: u32) -> String {
    format!(
        "aggregator_{}_prepare_{}",
        aggregator_tracker_backend_name(backend),
        load
    )
}

fn aggregator_tracker_backend_name(backend: &AggregatorTrackerBackend) -> &'static str {
    match backend {
        AggregatorTrackerBackend::BTree => "btree",
        AggregatorTrackerBackend::SailsFixed => "sails_fixed",
    }
}

fn aggregator_tracker_op_name(op: &AggregatorTrackerOp) -> &'static str {
    match op {
        AggregatorTrackerOp::InsertFresh => "insert_fresh",
        AggregatorTrackerOp::UpdateExisting => "update_existing",
        AggregatorTrackerOp::ReadExisting => "read_existing",
        AggregatorTrackerOp::ListStatuses => "list_statuses",
    }
}

fn storage_bench_key(
    backend: &StorageBackend,
    map: &StorageMap,
    op: &StorageOp,
    load: u32,
) -> String {
    format!(
        "{}_{}_{}_{}",
        storage_backend_name(backend),
        storage_map_name(map),
        storage_op_name(op),
        load
    )
}

fn storage_prepare_key(backend: &StorageBackend, map: &StorageMap, load: u32) -> String {
    format!(
        "{}_{}_prepare_{}",
        storage_backend_name(backend),
        storage_map_name(map),
        load
    )
}

fn vft_transfer_key(backend: &VftStorageBackend, op: &VftTransferOp, load: u32) -> String {
    vft_named_key(backend, vft_transfer_op_name(op), load)
}

fn vft_prepare_key(backend: &VftStorageBackend, load: u32) -> String {
    format!("vft_{}_prepare_{load}", vft_backend_name(backend))
}

fn vft_named_key(backend: &VftStorageBackend, op: &str, load: u32) -> String {
    format!("vft_{}_{}_{}", vft_backend_name(backend), op, load)
}

fn storage_backend_name(backend: &StorageBackend) -> &'static str {
    match backend {
        StorageBackend::HashMap => "hashmap",
        StorageBackend::Fixed => "fixed",
        StorageBackend::RawStatic => "raw_static",
        StorageBackend::SailsFixed => "sails_fixed",
        StorageBackend::SailsStatic => "sails_static",
    }
}

fn vft_backend_name(backend: &VftStorageBackend) -> &'static str {
    match backend {
        VftStorageBackend::BTree => "btree",
        VftStorageBackend::HashMap => "hashmap",
        VftStorageBackend::SailsFixed => "sails_fixed",
        VftStorageBackend::SailsStatic => "sails_static",
        VftStorageBackend::SailsStaticFast => "sails_static_fast",
    }
}

fn vft_transfer_op_name(op: &VftTransferOp) -> &'static str {
    match op {
        VftTransferOp::Transfer => "transfer",
        VftTransferOp::TransferFrom => "transfer_from",
    }
}

fn storage_map_name(map: &StorageMap) -> &'static str {
    match map {
        StorageMap::Balance => "balance",
        StorageMap::Allowance => "allowance",
    }
}

fn storage_op_name(op: &StorageOp) -> &'static str {
    match op {
        StorageOp::InsertFresh => "insert_fresh",
        StorageOp::UpdateExisting => "update_existing",
        StorageOp::ReadExisting => "read_existing",
        StorageOp::ReadMissing => "read_missing",
        StorageOp::Remove => "remove",
    }
}

fn assert_storage_stress_wasm_static_memory_layout(wasm_path: &str) {
    let wasm = std::fs::read(wasm_path).expect("storage-stress optimized WASM exists");
    let mut imported_memory_pages = None;

    'payloads: for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
        let wasmparser::Payload::ImportSection(imports) =
            payload.expect("storage-stress optimized WASM parses")
        else {
            continue;
        };

        for imports in imports {
            match imports.expect("storage-stress import parses") {
                wasmparser::Imports::Single(_, import) => {
                    if let wasmparser::TypeRef::Memory(memory) = import.ty {
                        imported_memory_pages = Some(memory.initial);
                        break 'payloads;
                    }
                }
                wasmparser::Imports::Compact1 { items, .. } => {
                    for item in items {
                        let item = item.expect("storage-stress compact import parses");
                        if let wasmparser::TypeRef::Memory(memory) = item.ty {
                            imported_memory_pages = Some(memory.initial);
                            break 'payloads;
                        }
                    }
                }
                wasmparser::Imports::Compact2 { ty, .. } => {
                    if let wasmparser::TypeRef::Memory(memory) = ty {
                        imported_memory_pages = Some(memory.initial);
                        break 'payloads;
                    }
                }
            }
        }
    }

    let imported_memory_pages = imported_memory_pages.expect("storage-stress imports memory");
    assert!(
        imported_memory_pages >= u64::from(::storage_stress::STATIC_MEMORY_END_PAGE),
        "storage-stress imported memory has {imported_memory_pages} pages, expected at least {}",
        ::storage_stress::STATIC_MEMORY_END_PAGE
    );
}

fn assert_vft_stress_wasm_static_memory_layout(wasm_path: &str) {
    let wasm = std::fs::read(wasm_path).expect("vft-stress optimized WASM exists");
    let mut imported_memory_pages = None;

    'payloads: for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
        let wasmparser::Payload::ImportSection(imports) =
            payload.expect("vft-stress optimized WASM parses")
        else {
            continue;
        };

        for imports in imports {
            match imports.expect("vft-stress import parses") {
                wasmparser::Imports::Single(_, import) => {
                    if let wasmparser::TypeRef::Memory(memory) = import.ty {
                        imported_memory_pages = Some(memory.initial);
                        break 'payloads;
                    }
                }
                wasmparser::Imports::Compact1 { items, .. } => {
                    for item in items {
                        let item = item.expect("vft-stress compact import parses");
                        if let wasmparser::TypeRef::Memory(memory) = item.ty {
                            imported_memory_pages = Some(memory.initial);
                            break 'payloads;
                        }
                    }
                }
                wasmparser::Imports::Compact2 { ty, .. } => {
                    if let wasmparser::TypeRef::Memory(memory) = ty {
                        imported_memory_pages = Some(memory.initial);
                        break 'payloads;
                    }
                }
            }
        }
    }

    let imported_memory_pages = imported_memory_pages.expect("vft-stress imports memory");
    assert!(
        imported_memory_pages >= u64::from(::vft_stress::STATIC_MEMORY_END_PAGE),
        "vft-stress imported memory has {imported_memory_pages} pages, expected at least {}",
        ::vft_stress::STATIC_MEMORY_END_PAGE
    );
}

fn create_env() -> GtestEnv {
    let system = System::new();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);
    GtestEnv::new(system, DEFAULT_USER_ALICE.into())
}

async fn deploy_for_bench<P, IO, F>(env: &GtestEnv, wasm_path: &str, f: F) -> Actor<P, GtestEnv>
where
    P: Program,
    IO: ServiceCall,
    <IO as sails_rs::client::ServiceCall>::Output: PendingCtorOutput<P, sails_rs::client::GtestEnv>,
    F: FnOnce(Deployment<P, GtestEnv>) -> PendingCtor<P, IO, GtestEnv>,
{
    let code_id = env.system().submit_local_code_file(wasm_path);
    deploy_code_for_bench(env, code_id, f).await
}

async fn deploy_code_for_bench<P, IO, F>(
    env: &GtestEnv,
    code_id: CodeId,
    f: F,
) -> Actor<P, GtestEnv>
where
    P: Program,
    IO: ServiceCall,
    <IO as sails_rs::client::ServiceCall>::Output: PendingCtorOutput<P, sails_rs::client::GtestEnv>,
    F: FnOnce(Deployment<P, GtestEnv>) -> PendingCtor<P, IO, GtestEnv>,
{
    let salt = COUNTER_SALT
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        .to_le_bytes()
        .to_vec();
    let deployment = env.deploy::<P>(code_id, salt);
    let ctor = f(deployment);
    let program = ctor
        .with_value(100_000_000_000_000)
        .await
        .expect("failed to initialize the program");
    <<IO as sails_rs::client::ServiceCall>::Output as PendingCtorOutput<
        P,
        sails_rs::client::GtestEnv,
    >>::actor(program)
}

fn extract_reply_and_gas(system: &System, message_id: MessageId) -> (Vec<u8>, u64) {
    extract_reply_and_gas_profiled(system, message_id, None)
}

fn extract_reply_and_gas_profiled(
    system: &System,
    message_id: MessageId,
    #[allow(unused_variables)] benchmark: Option<&str>,
) -> (Vec<u8>, u64) {
    let block_res = system.run_next_block();
    assert!(
        block_res.succeed.contains(&message_id),
        "message {message_id:?} did not succeed: {block_res:?}"
    );
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

    #[cfg(feature = "gas-profile")]
    if let Some(benchmark) = benchmark {
        let profile = block_res
            .gas_profiles
            .get(&message_id)
            .expect("gas profile should be recorded");
        let buckets = profile
            .buckets()
            .iter()
            .map(|(key, amount)| {
                (
                    key.category.as_str().to_owned(),
                    key.label.to_owned(),
                    *amount,
                )
            })
            .collect::<Vec<_>>();
        crate::write_gas_profile_artifact(benchmark, gas, buckets)
            .expect("failed to persist gas profile artifact");
    }

    (payload, gas)
}

#[cfg(feature = "gas-profile")]
fn write_vft_phase_profile(benchmark: &str, gas: u64, profile: &VftProfileResult) {
    let buckets = profile
        .phases
        .iter()
        .map(|phase| {
            (
                "wasm_phase".to_owned(),
                vft_phase_name(&phase.phase).to_owned(),
                phase.gas,
            )
        })
        .collect::<Vec<_>>();
    crate::write_gas_profile_artifact(&format!("{benchmark}_wasm_phases"), gas, buckets)
        .expect("failed to persist wasm phase profile artifact");
}

#[cfg(feature = "gas-profile")]
fn vft_phase_name(phase: &VftPhase) -> &'static str {
    match phase {
        VftPhase::ProbeOverhead => "probe_overhead",
        VftPhase::NoopBody => "noop_body",
        VftPhase::EchoBody => "echo_body",
        VftPhase::KeyDerive => "key_derive",
        VftPhase::AllowanceGet => "allowance_get",
        VftPhase::AllowancePut => "allowance_put",
        VftPhase::BalanceGetFrom => "balance_get_from",
        VftPhase::BalanceGetTo => "balance_get_to",
        VftPhase::BalancePutFrom => "balance_put_from",
        VftPhase::BalancePutTo => "balance_put_to",
        VftPhase::BalanceTransfer => "balance_transfer",
        VftPhase::BalanceTransferFrom => "balance_transfer_from",
        VftPhase::ResultBuild => "result_build",
    }
}

#[cfg(feature = "gas-profile")]
fn render_noop_floor_markdown(medians: &BTreeMap<String, u64>) -> String {
    let mut lines = vec![
        "# Noop Floor Gas Profile Summary".to_owned(),
        "".to_owned(),
        "Use a fresh `SAILS_GAS_PROFILE_DIR` for each comparison; the profile breakdown reads every JSON artifact under `profiles/`."
            .to_owned(),
        "".to_owned(),
        "| Key | Median gas | Delta vs `noop_wat_raw` | Delta vs `noop_gstd` |".to_owned(),
        "| --- | ---: | ---: | ---: |".to_owned(),
    ];

    let wat_floor = medians.get("noop_wat_raw").copied().unwrap_or(0);
    let gstd_floor = medians.get("noop_gstd").copied().unwrap_or(0);
    for (key, value) in medians {
        lines.push(format!(
            "| `{key}` | {value} | {} | {} |",
            format_signed_delta(*value, wat_floor),
            format_signed_delta(*value, gstd_floor),
        ));
    }

    render_route_reply_shape_markdown(&mut lines);
    render_wasm_section_sizes_markdown(&mut lines);
    render_profile_breakdown_markdown(&mut lines);

    lines.join("\n")
}

#[cfg(feature = "gas-profile")]
fn format_signed_delta(value: u64, baseline: u64) -> String {
    if value >= baseline {
        (value - baseline).to_string()
    } else {
        format!("-{}", baseline - value)
    }
}

#[cfg(feature = "gas-profile")]
fn render_minimal_vft_floor_markdown(medians: &BTreeMap<String, u64>) -> String {
    let mut lines = vec![
        "# Minimal VFT Floor Gas Profile Summary".to_owned(),
        "".to_owned(),
        "This compares a production-shaped single-backend Sails VFT with a manual Sails-wire hot path and the current benchmark VFT framework rows."
            .to_owned(),
        "".to_owned(),
        "| Key | Median gas | Delta vs matching hot row |".to_owned(),
        "| --- | ---: | ---: |".to_owned(),
    ];

    for (key, value) in medians {
        let hot_key = key.replace("minimal_vft_sails_", "minimal_vft_hot_");
        let delta = medians
            .get(&hot_key)
            .map(|hot| value.saturating_sub(*hot))
            .unwrap_or(0);
        lines.push(format!("| `{key}` | {value} | {delta} |"));
    }

    render_route_reply_shape_markdown(&mut lines);
    render_wasm_section_sizes_markdown(&mut lines);
    render_profile_breakdown_markdown(&mut lines);

    lines.join("\n")
}

#[cfg(feature = "gas-profile")]
fn render_vft_comparison_markdown(medians: &BTreeMap<String, u64>) -> String {
    let mut lines = vec![
        "# VFT Gas Profile Summary".to_owned(),
        "".to_owned(),
        "Use a fresh `SAILS_GAS_PROFILE_DIR` for each comparison; the profile breakdown reads every JSON artifact under `profiles/`."
            .to_owned(),
        "".to_owned(),
        "| Key | Median gas |".to_owned(),
        "| --- | ---: |".to_owned(),
    ];

    for (key, value) in medians {
        lines.push(format!("| `{key}` | {value} |"));
    }

    render_framework_overhead_markdown(medians, &mut lines);
    render_profile_breakdown_markdown(&mut lines);

    lines.join("\n")
}

#[cfg(feature = "gas-profile")]
fn render_framework_overhead_markdown(medians: &BTreeMap<String, u64>, lines: &mut Vec<String>) {
    let Some(noop) = medians.get("vft_framework_noop").copied() else {
        return;
    };

    lines.extend([
        "".to_owned(),
        "## Framework Baselines".to_owned(),
        "".to_owned(),
        "| Baseline | Median gas | Delta vs noop |".to_owned(),
        "| --- | ---: | ---: |".to_owned(),
        format!("| `vft_framework_noop` | {noop} | 0 |"),
    ]);

    if let Some(echo_args) = medians.get("vft_framework_echo_args").copied() {
        lines.push(format!(
            "| `vft_framework_echo_args` | {echo_args} | {} |",
            echo_args.saturating_sub(noop)
        ));
    }

    lines.extend([
        "".to_owned(),
        "`vft_framework_noop` is the lower-bound Sails call cost for this service. \
         The echo delta estimates argument decode plus minimal method-body work without storage."
            .to_owned(),
    ]);
}

#[cfg(feature = "gas-profile")]
fn render_route_reply_shape_markdown(lines: &mut Vec<String>) {
    lines.extend([
        "".to_owned(),
        "## Route And Reply Shape".to_owned(),
        "".to_owned(),
        "| Row family | Input shape | Reply shape |".to_owned(),
        "| --- | --- | --- |".to_owned(),
        "| `noop_wat_raw` | empty bytes | one raw byte `[1]` |".to_owned(),
        "| `noop_wat_sails_wire` | 16-byte Sails header, no params | 16-byte Sails header plus SCALE `bool` |".to_owned(),
        "| `noop_gstd` | empty bytes | one raw byte `[1]` via `gstd::msg::reply_bytes` |".to_owned(),
        "| `noop_sails` | generated Sails call | generated Sails reply with `bool` |".to_owned(),
        "| `minimal_vft_sails_*` | generated Sails call with VFT params | generated Sails reply with `bool` |".to_owned(),
        "| `minimal_vft_hot_*` | manual Sails header with fixed VFT params | manual Sails header plus SCALE `bool` |".to_owned(),
    ]);
}

#[cfg(feature = "gas-profile")]
fn render_profile_breakdown_markdown(lines: &mut Vec<String>) {
    let artifacts = crate::read_gas_profile_artifacts().unwrap_or_default();
    if artifacts.is_empty() {
        return;
    }

    let mut phase_artifacts = BTreeMap::new();
    let mut host_artifacts = BTreeMap::new();
    for artifact in artifacts {
        if let Some(base_name) = artifact.benchmark.strip_suffix("_wasm_phases") {
            phase_artifacts.insert(base_name.to_owned(), artifact);
        } else {
            host_artifacts.insert(artifact.benchmark.clone(), artifact);
        }
    }

    if host_artifacts.is_empty() {
        return;
    }

    lines.extend([
        "".to_owned(),
        "## Profile Breakdown".to_owned(),
        "".to_owned(),
        "| Benchmark | Total | Non-Wasm buckets | Lazy-pages | Residual Wasm | Wasm phases | Unexplained Wasm |"
            .to_owned(),
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |".to_owned(),
    ]);

    for (benchmark, host) in host_artifacts {
        let non_wasm_total = non_wasm_bucket_total(&host);
        let lazy_pages = lazy_pages_bucket_total(&host);
        let residual_wasm = residual_wasm_bucket_total(&host);
        let phase_total = phase_artifacts
            .get(&benchmark)
            .map(bucket_total)
            .unwrap_or(0);
        let unexplained_wasm = residual_wasm.saturating_sub(phase_total);

        lines.push(format!(
            "| `{benchmark}` | {} | {} | {} | {} | {} | {} |",
            host.total_gas,
            non_wasm_total,
            lazy_pages,
            residual_wasm,
            phase_total,
            unexplained_wasm
        ));
    }

    lines.extend([
        "".to_owned(),
        "Unexplained Wasm is `residual;wasm` after subtracting explicit benchmark phases. \
         For profile rows this is the first place to look for Sails decode, routing, \
         reply encoding, and uninstrumented app code."
            .to_owned(),
    ]);
}

#[cfg(feature = "gas-profile")]
fn render_wasm_section_sizes_markdown(lines: &mut Vec<String>) {
    let mut rows = vec![
        (
            "noop_wat_raw",
            WasmSectionSizes::from_bytes(::noop_wat::RAW_REPLY_WASM),
        ),
        (
            "noop_wat_sails_wire",
            WasmSectionSizes::from_bytes(::noop_wat::SAILS_WIRE_REPLY_WASM),
        ),
    ];

    for (name, path) in [
        (
            "noop_gstd",
            "../target/wasm32-gear/release/noop_gstd.opt.wasm",
        ),
        (
            "noop_sails",
            "../target/wasm32-gear/release/noop_sails.opt.wasm",
        ),
        (
            "minimal_vft_sails",
            "../target/wasm32-gear/release/minimal_vft_sails.opt.wasm",
        ),
        (
            "minimal_vft_hot",
            "../target/wasm32-gear/release/minimal_vft_hot.opt.wasm",
        ),
        (
            "vft_stress",
            "../target/wasm32-gear/release/vft_stress.opt.wasm",
        ),
        (
            "storage_million",
            "../target/wasm32-gear/release/storage_million.opt.wasm",
        ),
    ] {
        if let Some(sizes) = WasmSectionSizes::read(path) {
            rows.push((name, sizes));
        }
    }

    if rows.is_empty() {
        return;
    }

    lines.extend([
        "".to_owned(),
        "## Wasm Section Sizes".to_owned(),
        "".to_owned(),
        "| Program | Bytes | Type | Import | Function | Code | Data | Global | Table | Element | Memory | Export | Custom | Other |"
            .to_owned(),
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
            .to_owned(),
    ]);

    for (name, sizes) in rows {
        lines.push(format!(
            "| `{name}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            sizes.file,
            sizes.r#type,
            sizes.import,
            sizes.function,
            sizes.code,
            sizes.data,
            sizes.global,
            sizes.table,
            sizes.element,
            sizes.memory,
            sizes.export,
            sizes.custom,
            sizes.other,
        ));
    }

    lines.extend([
        "".to_owned(),
        "`instrumented_code` precharge tracks code bytes read from storage. `module_instantiation` is driven by instantiated section sizes, especially code, data, globals, tables, elements, and types."
            .to_owned(),
    ]);
}

#[cfg(feature = "gas-profile")]
#[derive(Default)]
struct WasmSectionSizes {
    file: usize,
    r#type: usize,
    import: usize,
    function: usize,
    code: usize,
    data: usize,
    global: usize,
    table: usize,
    element: usize,
    memory: usize,
    export: usize,
    custom: usize,
    other: usize,
}

#[cfg(feature = "gas-profile")]
impl WasmSectionSizes {
    fn read(path: &str) -> Option<Self> {
        let wasm = std::fs::read(path).ok()?;
        Some(Self::from_bytes(&wasm))
    }

    fn from_bytes(wasm: &[u8]) -> Self {
        let mut sizes = Self {
            file: wasm.len(),
            ..Self::default()
        };

        for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
            let Ok(payload) = payload else {
                return sizes;
            };
            let Some((section, range)) = payload.as_section() else {
                continue;
            };
            let len = range.end.saturating_sub(range.start);
            match section {
                1 => sizes.r#type += len,
                2 => sizes.import += len,
                3 => sizes.function += len,
                4 => sizes.table += len,
                5 => sizes.memory += len,
                6 => sizes.global += len,
                7 => sizes.export += len,
                9 => sizes.element += len,
                10 => sizes.code += len,
                11 => sizes.data += len,
                0 => sizes.custom += len,
                _ => sizes.other += len,
            }
        }

        sizes
    }
}

#[cfg(feature = "gas-profile")]
fn bucket_total(artifact: &crate::GasProfileArtifactSerde) -> u64 {
    artifact.buckets.iter().map(|bucket| bucket.amount).sum()
}

#[cfg(feature = "gas-profile")]
fn non_wasm_bucket_total(artifact: &crate::GasProfileArtifactSerde) -> u64 {
    artifact
        .buckets
        .iter()
        .filter(|bucket| bucket.category != "residual" || bucket.label != "wasm")
        .map(|bucket| bucket.amount)
        .sum()
}

#[cfg(feature = "gas-profile")]
fn residual_wasm_bucket_total(artifact: &crate::GasProfileArtifactSerde) -> u64 {
    artifact
        .buckets
        .iter()
        .filter(|bucket| bucket.category == "residual" && bucket.label == "wasm")
        .map(|bucket| bucket.amount)
        .sum()
}

#[cfg(feature = "gas-profile")]
fn lazy_pages_bucket_total(artifact: &crate::GasProfileArtifactSerde) -> u64 {
    artifact
        .buckets
        .iter()
        .filter(|bucket| bucket.category.contains("lazy") || bucket.label.contains("lazy"))
        .map(|bucket| bucket.amount)
        .sum()
}

#[cfg(feature = "gas-profile")]
fn floor_samples() -> u32 {
    env::var("SAILS_FLOOR_SAMPLES")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|samples| *samples > 0)
        .unwrap_or(DEFAULT_FLOOR_SAMPLES)
}

#[cfg(feature = "gas-profile")]
fn floor_row_enabled(row: &str) -> bool {
    let Ok(rows) = env::var("SAILS_FLOOR_ROWS") else {
        return true;
    };

    rows.split(',')
        .map(str::trim)
        .filter(|row| !row.is_empty())
        .any(|enabled| row == enabled || row.starts_with(enabled))
}

#[cfg(feature = "gas-profile")]
fn noop_sails_payload() -> Vec<u8> {
    let mut payload = Vec::with_capacity(16);
    payload.extend_from_slice(b"GM");
    payload.push(1);
    payload.push(16);
    payload.extend_from_slice(&[84, 87, 204, 48, 221, 148, 206, 41]);
    payload.extend_from_slice(&0u16.to_le_bytes());
    payload.push(NoopSailsProgram::ROUTE_ID_NOOP_SAILS);
    payload.push(0);
    payload
}

#[cfg(feature = "gas-profile")]
fn hot_vft_payload(entry_id: u16, params: Vec<u8>) -> Vec<u8> {
    let mut payload = Vec::with_capacity(16 + params.len());
    payload.extend_from_slice(b"GM");
    payload.push(1);
    payload.push(16);
    payload.extend_from_slice(&HOT_VFT_INTERFACE_ID);
    payload.extend_from_slice(&entry_id.to_le_bytes());
    payload.push(HOT_VFT_ROUTE_ID);
    payload.push(0);
    payload.extend_from_slice(&params);
    payload
}

#[cfg(feature = "gas-profile")]
fn assert_sails_bool_reply(entry_id: u16, payload: &[u8]) {
    assert_eq!(payload.len(), 17, "Sails bool reply length");
    assert_eq!(&payload[0..2], b"GM");
    assert_eq!(payload[2], 1);
    assert_eq!(payload[3], 16);
    assert_eq!(&payload[4..12], &HOT_VFT_INTERFACE_ID);
    assert_eq!(u16::from_le_bytes([payload[12], payload[13]]), entry_id);
    assert_eq!(payload[14], HOT_VFT_ROUTE_ID);
    assert_eq!(payload[16], 1);
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
