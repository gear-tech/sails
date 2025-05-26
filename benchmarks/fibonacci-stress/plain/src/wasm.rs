use crate::Action;
use fibonacci_stress_core as fibo_stress;
use parity_scale_codec::Encode;

#[unsafe(no_mangle)]
extern "C" fn handle() {
    match gstd::msg::load().expect("invalid payload") {
        Action::StressFiboOptimized(n) => {
            let payload = fibo_stress::stress_fibo(n);
            // This replies using stack buffer, if possible
            gstd::msg::reply(payload, 0).expect("failed to reply");
        }
        Action::StressFibo(n) => {
            let payload = fibo_stress::stress_fibo(n);
            // This replies with heap allocation
            gstd::msg::reply_bytes(payload.encode(), 0).expect("failed to reply");
        }
        Action::StressBytesOptimized(n) => {
            let payload = fibo_stress::stress_bytes(n);
            // This replies using stack buffer, if possible
            gstd::msg::reply(payload, 0).expect("failed to reply");
        }
        Action::StressBytes(n) => {
            let payload = fibo_stress::stress_bytes(n);
            // This replies with heap allocation
            gstd::msg::reply_bytes(payload.encode(), 0).expect("failed to reply");
        }
    }
}
