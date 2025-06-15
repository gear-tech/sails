//! Constants lengths are not chosen randomly.
//! The following show the sum of the first `n` fibonacci numbers:
//! n = 6, sum = 12;
//! n = 11, sum = 143;
//! n = 15, sum = 986;
//! n = 20, sum = 10_945;
//! n = 23, sum = 46_367;
//! n = 25, sum = 121_392;
//! n = 27, sum = 317_810;
//! So `BYTES`

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use parity_scale_codec::Encode;
use scale_info::TypeInfo;

/// Wrapper over a [`stress_fibo`] buffer result.
#[derive(TypeInfo, Encode)]
pub struct FiboStressResult {
    pub inner: Vec<u8>,
}

pub const BYTES_6: &[u8] = &[b'a'; 12];
pub const BYTES_11: &[u8] = &[b'a'; 143];
pub const BYTES_15: &[u8] = &[b'a'; 986];
pub const BYTES_20: &[u8] = &[b'a'; 10_945];
pub const BYTES_23: &[u8] = &[b'a'; 46_367];
pub const BYTES_25: &[u8] = &[b'a'; 121_392];
pub const BYTES_27: &[u8] = &[b'a'; 317_810];

/// A number of first `n` fibonacci numbers for which the sum is pre-calculated
pub const FIBONACCI_NS: [u32; 8] = [0, 6, 11, 15, 20, 23, 25, 27];

/// Allocates a buffer of size equal to the sum of the first `n` Fibonacci numbers,
/// filling it with the byte `42`.
pub fn stress_fibo(n: u32) -> FiboStressResult {
    let sum = fibonacci_sum(n);
    let mut inner = Vec::with_capacity(sum as usize);
    (0..sum).for_each(|_| inner.push(42));

    FiboStressResult { inner }
}

/// Returns a static byte slice based on the input `n`.
pub fn stress_bytes(n: u32) -> &'static [u8] {
    match n {
        0 => &[],
        6 => BYTES_6,
        11 => BYTES_11,
        15 => BYTES_15,
        20 => BYTES_20,
        23 => BYTES_23,
        25 => BYTES_25,
        27 => BYTES_27,
        _ => unimplemented!("n = {n} is not supported"),
    }
}

/// Counts the sum of the first `n` Fibonacci numbers.
pub fn fibonacci_sum(n: u32) -> u32 {
    let (mut sum, mut prev, mut curr) = (0u32, 0u32, 1u32);

    match n {
        0 => 0,
        1 => sum,
        _ => {
            for _ in 2..=n {
                sum += curr;
                (prev, curr) = (curr, prev + curr);
            }
            sum
        }
    }
}
