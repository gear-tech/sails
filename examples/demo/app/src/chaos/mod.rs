use sails_rs::gstd::debug;
use sails_rs::{gstd, prelude::*};

static mut REPLY_HOOK_COUNTER: u32 = 0;

pub struct ChaosService;

#[service]
impl ChaosService {
    #[export]
    pub async fn panic_after_wait(&self) {
        let source = gstd::msg::source();
        let _ = gstd::msg::send_for_reply::<()>(source, (), 0, 0)
            .unwrap()
            .await;
        debug!("Message received, now panicking!");
        panic!("Simulated panic after wait");
    }

    #[export]
    pub async fn timeout_wait(&self) {
        let source = gstd::msg::source();
        debug!("before handle_reply");

        let fut = gstd::msg::send_for_reply::<()>(source, (), 0, 10_000_000_000).unwrap();
        let fut = fut
            .handle_reply(|| {
                unsafe { REPLY_HOOK_COUNTER += 1 };
                debug!("handle_reply triggered");
            })
            .unwrap()
            .up_to(Some(1))
            .unwrap();

        let _ = fut.await;
        debug!("after handle_reply");
    }

    #[export]
    pub async fn noop(&self) {
        debug!("Noop");
    }

    #[export]
    pub fn reply_hook_counter(&self) -> u32 {
        unsafe { REPLY_HOOK_COUNTER }
    }
}
