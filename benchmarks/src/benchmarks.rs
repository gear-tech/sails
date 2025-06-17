// Todo [sab] write docs to all the benches

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

use gtest::{System, constants::DEFAULT_USER_ALICE};
use sails_rs::{
    calls::{ActionIo, Activation},
    gtest::calls::GTestRemoting,
};

async fn alloc_stress_test(n: u32) -> (u64, u32) {
    // Path taken from the .binpath file
    let wasm_path = "../target/wasm32-gear/release/alloc_stress_app.opt.wasm";

    let system = System::new();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = system.submit_local_code_file(wasm_path);

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
    let payload = AllocStress::encode_call(n);
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
    let stress_res = AllocStress::decode_reply(payload).expect("failed to decode payload");

    let expected_len = alloc_stress_app::fibonacci_sum(n) as usize;
    assert_eq!(stress_res.inner.len(), expected_len);

    (
        block_res
            .gas_burned
            .get(&mid)
            .copied()
            .expect("msg was executed; qed."),
        expected_len.try_into().unwrap(),
    )
}

#[tokio::test]
async fn alloc_stress_bench() {
    let fibonacci_ns = [0, 6, 11, 15, 20, 23, 25, 27];

    for &n in fibonacci_ns.iter() {
        let (gas, len) = alloc_stress_test(n).await;

        crate::store_bench_data(|bench_data| {
            bench_data.alloc.insert(len, gas);
        })
        .unwrap();
    }
}

#[tokio::test]
async fn compute_stress_bench() {
    let wasm_path = "../target/wasm32-gear/release/compute_stress_app.opt.wasm";

    let system = System::new();
    system.init_logger();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = system.submit_local_code_file(wasm_path);

    // Create program and initialize it
    let remoting = GTestRemoting::new(system, DEFAULT_USER_ALICE.into());
    let factory = ComputeStressFactory::new(remoting.clone());
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
    let input_value = 30;
    let payload = ComputeStress::encode_call(input_value);
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
    let stress_res = ComputeStress::decode_reply(payload).expect("failed to decode payload");

    let expected_sum = compute_stress_app::sum_of_fib(input_value);
    assert_eq!(stress_res.res, expected_sum);

    let gas = block_res
        .gas_burned
        .get(&mid)
        .copied()
        .expect("msg was executed; qed.");

    let gas = format!("{gas}")
        .parse::<u64>()
        .expect("value is a valid u64");

    crate::store_bench_data(|bench_data| {
        bench_data.compute = gas;
    })
    .unwrap();
}

#[tokio::test]
async fn counter_bench() {
    let wasm_path = "../target/wasm32-gear/release/counter_bench_app.opt.wasm";

    let system = System::new();
    system.init_logger();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = system.submit_local_code_file(wasm_path);

    // Create program and initialize it
    let remoting = GTestRemoting::new(system, DEFAULT_USER_ALICE.into());
    let factory = CounterBenchFactory::new(remoting.clone());
    let pid = factory
        .new()
        .send_recv(code_id, b"counter_bench_prog")
        .await
        .expect("failed to initialize the program");

    // Using low-level `gtest` as it's possible to have block execution data this way.
    let system_remoting = remoting.system();
    let program = system_remoting
        .get_program(pid)
        .expect("program was created; qed.");
    let from = remoting.actor_id();

    // Form payload for the program. Calling `inc` method.
    let payload = Inc::encode_call();
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
    let res = Inc::decode_reply(payload).expect("failed to decode payload");

    assert_eq!(res, 0);

    let gas_sync_inc = block_res
        .gas_burned
        .get(&mid)
        .copied()
        .expect("msg was executed; qed.");

    crate::store_bench_data(|bench_data| {
        bench_data.counter.sync_call = gas_sync_inc;
    }).unwrap();

    // Increment counter again using async call async method.
    let payload = IncAsync::encode_call();
    let mid2 = program.send_bytes(from, payload);
    let block_res2 = system_remoting.run_next_block();
    assert!(block_res2.succeed.contains(&mid2));

    // Check received payload
    let payload = block_res2
        .log()
        .iter()
        .find_map(|log| {
            log.reply_to()
                .filter(|reply_to| reply_to == &mid2)
                .map(|_| log.payload().to_vec())
        })
        .expect("internal error: no reply was found");
    let res = IncAsync::decode_reply(payload).expect("failed to decode payload");

    assert_eq!(res, 1);

    let gas_async_inc = block_res2
        .gas_burned
        .get(&mid2)
        .copied()
        .expect("msg was executed; qed.");

    crate::store_bench_data(|bench_data| {
        bench_data.counter.async_call = gas_async_inc;
    }).unwrap();
}
