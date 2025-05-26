//! Entry point for fibonacci stress benchmarking.
//! 
//! TODO add docs

#![cfg(test)]

use fibonacci_stress_core::FIBONACCI_NS;
use fibonacci_stress_plain::{Action, WASM_BINARY};
use fibonacci_stress_sails_client::{
    FiboStressResult, FibonacciStressSailsFactory,
    fibo_stress::io::{StressBytes, StressFibo},
    traits::FibonacciStressSailsFactory as _,
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

impl Len for FiboStressResult {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Len for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }
}

async fn stress_test_sails_app<A>(n: u32) -> (Gas, usize)
where
    A: ActionIo<Params = u32>,
    A::Reply: Len,
{
    // Path taken from the .binpath file
    const WASM_PATH: &str = "../../target/wasm32-gear/debug/fibonacci_stress_sails.opt.wasm";

    let system = System::new();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let code_id = system.submit_local_code_file(WASM_PATH);

    // Create program and initialize it
    let remoting = GTestRemoting::new(system, DEFAULT_USER_ALICE.into());
    let factory = FibonacciStressSailsFactory::new(remoting.clone());
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

    let expected_len = fibonacci_stress_core::fibonacci_sum(n) as usize;
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
    for n in FIBONACCI_NS {
        let (gas, len) = stress_test_sails_app::<StressFibo>(n).await;

        println!("[SAILS_STRESS_FIBO]: Length = {len:>7} => Gas: {gas:>15}",);

        // println!("{gas}");
    }
}

#[tokio::test]
async fn sails_stress_bytes() {
    for n in FIBONACCI_NS {
        let (gas, len) = stress_test_sails_app::<StressBytes>(n).await;

        println!("[SAILS_STRESS_BYTES]: Length = {len:>7} => Gas: {gas:>15}",);

        // println!("{gas}");
    }
}

fn stress_test_plain_app(payload: Action) -> (Gas, usize) {
    let expected_len = fibonacci_stress_core::fibonacci_sum(payload.n()) as usize;
    let is_fibo = payload.is_fibo();

    let system = System::new();
    system.mint_to(DEFAULT_USER_ALICE, 1_000_000_000_000_000);

    let program = Program::from_binary_with_id(&system, 4243, WASM_BINARY);

    // Initialize the program
    let mid = program.send_bytes(DEFAULT_USER_ALICE, b"");
    let block_res = system.run_next_block();
    assert!(block_res.succeed.contains(&mid));

    // Send test payload
    let mid = program.send(DEFAULT_USER_ALICE, payload);
    let block_res = system.run_next_block();
    assert!(block_res.succeed.contains(&mid));

    let payload = block_res
        .log()
        .iter()
        .find_map(|log| {
            log.reply_to()
                .filter(|reply_to| reply_to == &mid)
                .map(|_| log.payload().to_vec())
        })
        .expect("internal error: no reply was found");
    let stress_res = if is_fibo {
        FiboStressResult::decode(&mut payload.as_ref())
            .expect("failed to decode payload")
            .inner
    } else {
        Decode::decode(&mut payload.as_ref()).expect("failed to decode payload")
    };

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
async fn plain_stress_fibo_optimized() {
    for n in FIBONACCI_NS {
        let payload = Action::StressFiboOptimized(n);
        let (gas, len) = stress_test_plain_app(payload);

        println!("[PLAIN_STRESS_FIBO_OPTIMIZED]: Length = {len:>7} => Gas: {gas:>14}",);

        // println!("{gas}");
    }
}

#[tokio::test]
async fn plain_stress_fibo() {
    for n in FIBONACCI_NS {
        let payload = Action::StressFibo(n);
        let (gas, len) = stress_test_plain_app(payload);

        println!("[PLAIN_STRESS_FIBO]: Length = {len:>7} => Gas: {gas:>15}",);

        // println!("{gas}");
    }
}

#[tokio::test]
async fn plain_stress_bytes_optimized() {
    for n in FIBONACCI_NS {
        let payload = Action::StressBytesOptimized(n);
        let (gas, len) = stress_test_plain_app(payload);

        println!("[PLAIN_STRESS_BYTES_OPTIMIZED]: Length = {len:>7} => Gas: {gas:>15}",);

        // println!("{gas}");
    }
}

#[tokio::test]
async fn plain_stress_bytes() {
    for n in FIBONACCI_NS {
        let payload = Action::StressBytes(n);
        let (gas, len) = stress_test_plain_app(payload);

        println!("[PLAIN_STRESS_BYTES]: Length = {len:>7} => Gas: {gas:>15}",);

        // println!("{gas}");
    }
}
