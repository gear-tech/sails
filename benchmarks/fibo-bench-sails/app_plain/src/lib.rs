#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}

#[cfg(feature = "std")]
pub use code::WASM_BINARY_OPT as WASM_BINARY;
use gstd::prelude::*;

#[derive(Debug, Encode, Decode)]
pub enum Action {
    Stress(u32),
    StressOptimized(u32),
}

#[derive(Encode, Decode)]
struct FiboStressResult {
    inner: Vec<u8>,
}

pub fn fibonacci_sum(n: u32) -> u32 {
    let (mut sum, mut prev, mut curr) = (0u32, 0u32, 1u32);

    match n {
        0 => 0,
        1 => sum,
        _ => {
            for _ in 2..=n {
                sum = sum + curr;
                (prev, curr) = (curr, prev + curr);
            }
            sum
        }
    }
}

#[cfg(not(feature = "std"))]
mod wasm;

#[cfg(test)]
mod tests {
    use super::*;
    use gtest::{Gas, Program, System, constants::DEFAULT_USER_ALICE};

    const NUMS: [u32; 8] = [0, 6, 11, 15, 20, 23, 25, 27];

    fn stress_test(system: &System, program: &Program<'_>, payload: Action) -> (Gas, usize) {
        let expected_len = match &payload {
            Action::Stress(n) => fibonacci_sum(*n) as usize,
            Action::StressOptimized(n) => fibonacci_sum(*n) as usize,
        };

        let mid = program.send(DEFAULT_USER_ALICE, payload);
        let block_res = system.run_next_block();
        assert!(block_res.succeed.contains(&mid));

        let mut payload = block_res
            .log()
            .iter()
            .find_map(|log| {
                log.reply_to()
                    .filter(|reply_to| reply_to == &mid)
                    .map(|_| log.payload().to_vec())
            })
            .expect("internal error: no reply was found");
        let stress_res =
            FiboStressResult::decode(&mut payload.as_ref()).expect("failed to decode payload");

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

    #[test]
    fn stress_optimized() {
        let system = System::new();
        let program = Program::current(&system);

        // Initialize the program
        let mid = program.send_bytes(DEFAULT_USER_ALICE, b"");
        let block_res = system.run_next_block();
        assert!(block_res.succeed.contains(&mid));

        // Stress test
        for n in NUMS {
            let payload = Action::StressOptimized(n);
            let (gas, len) = stress_test(&system, &program, payload);
            println!("{gas}");
        }
    }

    #[test]
    fn stress() {
        let system = System::new();
        let program = Program::current(&system);

        // Initialize the program
        let mid = program.send_bytes(DEFAULT_USER_ALICE, b"");
        let block_res = system.run_next_block();
        assert!(block_res.succeed.contains(&mid));

        // Stress test
        for n in NUMS {
            let payload = Action::Stress(n);
            let (gas, len) = stress_test(&system, &program, payload);
            println!("{gas}");
        }
    }
}
