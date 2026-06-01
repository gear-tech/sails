use sails_rs::gstd::{Lock, debug};
use sails_rs::{gstd, prelude::*};

static mut REPLY_HOOK_COUNTER: u32 = 0;
static mut CRITICAL_HOOK_COUNTER: u32 = 0;

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

    /// Suspends the message for `blocks` blocks via `sleep_for`, then returns
    /// the actual block delta. Used to verify the `MessageSleepFuture` lifecycle:
    /// register sleep lock -> `exec::wait_for` -> resume at deadline -> complete.
    #[export]
    pub async fn sleep_then_return(&self, blocks: u32) -> u32 {
        let start = Syscall::block_height();
        gstd::sleep_for(blocks).await;
        Syscall::block_height().saturating_sub(start)
    }

    /// Sends two messages for reply concurrently and resolves with the first to
    /// reply via `futures::select`. Exercises `next_lock` picking the earliest
    /// of several armed locks and `forget_future` reclaiming the losing branch
    /// (its `MessageFuture` is dropped unresolved). Returns `0` if the first
    /// send (`b"A"`) wins, `1` if the second (`b"B"`) does.
    #[export]
    pub async fn select_first_reply(&self) -> u8 {
        use sails_rs::futures::future::{Either, select};

        let source = Syscall::message_source();
        let a = gstd::send_bytes_for_reply(source, b"A", 0, Lock::up_to(100), None, None, None)
            .unwrap();
        let b = gstd::send_bytes_for_reply(source, b"B", 0, Lock::up_to(100), None, None, None)
            .unwrap();

        match select(a, b).await {
            Either::Left(_) => 0,
            Either::Right(_) => 1,
        }
    }

    /// Registers a critical hook, awaits a reply, then panics after resuming.
    /// The trap on a message that reserved system gas makes the runtime invoke
    /// `handle_signal`, which runs the stored critical hook. Used to verify the
    /// `set_critical_hook` -> `handle_signal` path (distinct from the userspace
    /// error reply). Observe the effect via [`Self::critical_hook_counter`].
    #[export]
    pub async fn critical_hook_on_signal(&self) {
        gstd::set_critical_hook(|_msg_id| {
            unsafe { CRITICAL_HOOK_COUNTER += 1 };
            debug!("critical hook fired in handle_signal");
        });
        let source = Syscall::message_source();
        let _ = gstd::send_for_reply::<()>(source, (), 0).unwrap().await;
        panic!("panic after wait to trigger a signal");
    }

    #[export]
    pub fn critical_hook_counter(&self) -> u32 {
        unsafe { CRITICAL_HOOK_COUNTER }
    }
}
