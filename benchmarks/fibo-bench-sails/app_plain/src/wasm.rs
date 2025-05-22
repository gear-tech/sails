use crate::{Action, FiboStressResult, fibonacci_sum};
use gstd::prelude::*;

#[unsafe(no_mangle)]
extern "C" fn handle() {
    match gstd::msg::load().expect("invalid payload") {
        Action::StressOptimized(n) => {
            let sum = fibonacci_sum(n);
            let mut buf = Vec::with_capacity(sum as usize);
            (0..sum).for_each(|_| buf.push(42));
            let payload = FiboStressResult { inner: buf };

            gstd::msg::reply(payload, 0).expect("failed to reply");
        }
        Action::Stress(n) => {
            let sum = fibonacci_sum(n);
            let mut buf = Vec::with_capacity(sum as usize);
            (0..sum).for_each(|_| buf.push(42));
            let payload = FiboStressResult { inner: buf };

            gstd::msg::reply_bytes(payload.encode(), 0).expect("failed to reply");
        }
    }
}
