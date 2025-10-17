use gstd::exec;
use sails_rs::gstd::debug;
use sails_rs::{gstd, prelude::*};

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
        let this_msg_id = gstd::msg::id();
        debug!("before handle_reply, source={source:?}, this_msg_id={this_msg_id:?}");

        let fut = gstd::msg::send_for_reply::<()>(source, (), 0, 1_000_000).unwrap();
        let fut = fut
            .handle_reply(|| {
                debug!("handle_reply triggered");
                panic!("hande reply")
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
}
