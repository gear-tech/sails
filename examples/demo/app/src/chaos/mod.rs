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
        let fut = gstd::msg::send_for_reply::<()>(source, (), 0, 0).unwrap();

        let fut = fut.up_to(Some(1)).unwrap();

        fut.await.unwrap();
    }

    #[export]
    pub async fn noop(&mut self) {
        debug!("Noop");
    }
}
