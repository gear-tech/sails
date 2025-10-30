use sails_rs::gstd::{Lock, debug};
use sails_rs::{gstd, prelude::*};

static mut REPLY_HOOK_COUNTER: u32 = 0;

pub struct ChaosService;

#[service]
impl ChaosService {
    #[export]
    pub async fn panic_after_wait(&self) {
        let source = Syscall::message_source();
        let _ = gstd::send_for_reply::<()>(source, (), 0).unwrap().await;
        debug!("Message received, now panicking!");
        panic!("Simulated panic after wait");
    }

    #[export]
    pub async fn timeout_wait(&self) {
        let source = Syscall::message_source();
        debug!("before handle_reply");

        let fut = gstd::send_bytes_for_reply(
            source,
            &[],
            0,
            Lock::up_to(1),
            None,
            Some(10_000_000_000),
            Some(Box::new(|| {
                unsafe { REPLY_HOOK_COUNTER += 1 };
                debug!("handle_reply triggered");
            })),
        )
        .unwrap();
        let _ = fut.await;
        debug!("after handle_reply");
    }

    #[export]
    pub fn reply_hook_counter(&self) -> u32 {
        unsafe { REPLY_HOOK_COUNTER }
    }
}
