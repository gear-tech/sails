use super::*;
use crate::{collections::HashMap, gstd::reply_hooks::HooksMap};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::future::{FusedFuture, FutureExt as _, LocalBoxFuture};
use gstd::{BlockNumber, errors::Error};

fn tasks() -> &'static mut crate::collections::HashMap<MessageId, Task> {
    static mut MAP: Option<crate::collections::HashMap<MessageId, Task>> = None;
    unsafe { &mut *core::ptr::addr_of_mut!(MAP) }
        .get_or_insert_with(crate::collections::HashMap::new)
}

fn signals() -> &'static mut WakeSignals {
    static mut MAP: Option<WakeSignals> = None;
    unsafe { &mut *core::ptr::addr_of_mut!(MAP) }.get_or_insert_with(WakeSignals::new)
}

fn reply_hooks() -> &'static mut HooksMap {
    static mut MAP: Option<HooksMap> = None;
    unsafe { &mut *core::ptr::addr_of_mut!(MAP) }.get_or_insert_with(HooksMap::new)
}

/// Matches a task to a some message in order to avoid duplicate execution
/// of code that was running before the program was interrupted by `wait`.
pub struct Task {
    future: LocalBoxFuture<'static, ()>,
    reply_to_locks: Vec<(MessageId, locks::Lock)>,
}

impl Task {
    fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        Self {
            future: future.boxed_local(),
            reply_to_locks: Vec::new(),
        }
    }

    #[inline]
    fn insert_lock(&mut self, reply_to: MessageId, lock: locks::Lock) {
        self.reply_to_locks.push((reply_to, lock));
    }

    #[inline]
    fn remove_lock(&mut self, reply_to: &MessageId) {
        if let Some(index) = self
            .reply_to_locks
            .iter()
            .position(|(mid, _)| mid == reply_to)
        {
            self.reply_to_locks.swap_remove(index);
        }
    }

    #[inline]
    fn signal_reply_timeout(&mut self, now: BlockNumber) {
        let signals_map = signals();

        self.reply_to_locks
            .extract_if(.., |(_, lock)| now >= lock.deadline())
            .for_each(|(reply_to, lock)| {
                signals_map.record_timeout(&reply_to, lock.deadline(), now);
                ::gstd::debug!(
                    "signal_reply_timeout: remove lock for reply_to {reply_to} in message due to timeout"
                );
            });
    }

    #[inline]
    fn wait(&self, now: BlockNumber) {
        self.reply_to_locks
            .iter()
            .map(|(_, lock)| lock)
            .min_by(|lock1, lock2| lock1.cmp(lock2))
            .expect("Cannot find lock to be waited")
            .wait(now);
    }
}

/// The main asynchronous message handling loop.
///
/// Gear allows user and program interaction via
/// messages. This function is the entry point to run the asynchronous message
/// processing.
#[inline]
pub fn message_loop<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let msg_id = ::gcore::msg::id();
    let tasks_map = tasks();
    let task = tasks_map.entry(msg_id).or_insert_with(|| {
        #[cfg(not(feature = "ethexe"))]
        {
            ::gcore::exec::system_reserve_gas(gstd::Config::system_reserve())
                .expect("Failed to reserve gas for system signal");
        }
        Task::new(future)
    });

    // Check if any reply has timed out before polling them.
    let current_block = Syscall::block_height();
    task.signal_reply_timeout(current_block);

    let completed = {
        let mut cx = Context::from_waker(task::Waker::noop());
        ::gstd::debug!("message_loop: polling future for {msg_id}");
        task.future.as_mut().poll(&mut cx).is_ready()
    };

    if completed {
        tasks_map.remove(&msg_id);
        // #[cfg(not(feature = "ethexe"))]
        // let _ = critical::take_hook();
    } else {
        task.wait(current_block);
    }
}

pub type Payload = Vec<u8>;

enum WakeSignal {
    Pending {
        message_id: MessageId,
    },
    Ready {
        payload: Payload,
        reply_code: ReplyCode,
    },
    Timeout {
        expected: BlockNumber,
        now: BlockNumber,
    },
}

struct WakeSignals {
    signals: HashMap<MessageId, WakeSignal>,
}

impl WakeSignals {
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
        }
    }

    pub fn register_signal(&mut self, waiting_reply_to: MessageId, lock: locks::Lock) {
        let message_id = ::gcore::msg::id();

        self.signals
            .insert(waiting_reply_to, WakeSignal::Pending { message_id });

        ::gstd::debug!(
            "register_signal: add lock for reply_to {waiting_reply_to} in message {message_id}"
        );
        tasks()
            .get_mut(&message_id)
            .expect("A message task must exist")
            .insert_lock(waiting_reply_to, lock);
    }

    pub fn record_reply(&mut self, reply_to: &MessageId) {
        if let Some(signal @ WakeSignal::Pending { .. }) = self.signals.get_mut(reply_to) {
            let message_id = match signal {
                WakeSignal::Pending { message_id } => *message_id,
                _ => unreachable!(),
            };
            *signal = WakeSignal::Ready {
                payload: ::gstd::msg::load_bytes().expect("Failed to load bytes"),
                reply_code: ::gcore::msg::reply_code()
                    .expect("Shouldn't be called with incorrect context"),
            };

            ::gstd::debug!(
                "record_reply: remove lock for reply_to {reply_to} in message {message_id}"
            );
            tasks()
                .get_mut(&message_id)
                .expect("A message task must exist")
                .remove_lock(&reply_to);

            // wake message processign after handle reply
            ::gcore::exec::wake(message_id).expect("Failed to wake the message")
        } else {
            ::gstd::debug!(
                "A message has received a reply though it wasn't to receive one, or a processed message has received a reply"
            );
        }
    }

    pub fn record_timeout(
        &mut self,
        reply_to: &MessageId,
        expected: BlockNumber,
        now: BlockNumber,
    ) {
        if let Some(signal @ WakeSignal::Pending { .. }) = self.signals.get_mut(reply_to) {
            *signal = WakeSignal::Timeout { expected, now };
        } else {
            ::gstd::debug!("A message has timed out after reply");
        }
    }

    pub fn waits_for(&self, reply_to: &MessageId) -> bool {
        self.signals.contains_key(reply_to)
    }

    pub fn poll(
        &mut self,
        reply_to: &MessageId,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Vec<u8>, Error>> {
        let entry = self.signals.entry_ref(reply_to);
        let entry = match entry {
            hashbrown::hash_map::EntryRef::Occupied(occupied_entry) => occupied_entry,
            hashbrown::hash_map::EntryRef::Vacant(_) => panic!("Poll not registered feature"),
        };
        if let WakeSignal::Pending { .. } = entry.get() {
            return Poll::Pending;
        }
        match entry.remove() {
            WakeSignal::Ready {
                payload,
                reply_code,
            } => match reply_code {
                ReplyCode::Success(_) => Poll::Ready(Ok(payload)),
                ReplyCode::Error(reason) => {
                    Poll::Ready(Err(Error::ErrorReply(payload.into(), reason)))
                }
                ReplyCode::Unsupported => Poll::Ready(Err(Error::UnsupportedReply(payload))),
            },
            WakeSignal::Timeout { expected, now, .. } => {
                Poll::Ready(Err(Error::Timeout(expected, now)))
            }
            _ => unreachable!(),
        }
    }
}

pub struct MessageFuture {
    /// A message identifier for an expected reply.
    ///
    /// This identifier is generated by the corresponding send function (e.g.
    /// [`send_bytes`](super::send_bytes)).
    pub waiting_reply_to: MessageId,
}

impl Unpin for MessageFuture {}

impl Future for MessageFuture {
    type Output = Result<Vec<u8>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll(&self.waiting_reply_to, cx)
    }
}

impl FusedFuture for MessageFuture {
    fn is_terminated(&self) -> bool {
        is_terminated(&self.waiting_reply_to)
    }
}

#[inline]
pub fn send_bytes_for_reply(
    destination: ActorId,
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
    gas_limit: Option<GasUnit>,
    reply_deposit: Option<GasUnit>,
    reply_hook: Option<Box<dyn FnOnce() + Send + 'static>>,
) -> Result<MessageFuture, ::gstd::errors::Error> {
    #[cfg(not(feature = "ethexe"))]
    let waiting_reply_to = if let Some(gas_limit) = gas_limit {
        crate::ok!(::gcore::msg::send_with_gas(
            destination,
            payload,
            gas_limit,
            value
        ))
    } else {
        crate::ok!(::gcore::msg::send(destination, payload, value))
    };
    #[cfg(feature = "ethexe")]
    let waiting_reply_to = ::gcore::msg::send(destination, payload, value);

    #[cfg(not(feature = "ethexe"))]
    if let Some(reply_deposit) = reply_deposit {
        let deposited = ::gcore::exec::reply_deposit(waiting_reply_to, reply_deposit).is_ok();
        if deposited && let Some(reply_hook) = reply_hook {
            reply_hooks().register(waiting_reply_to, reply_hook);
        }
    }

    signals().register_signal(waiting_reply_to, wait);

    Ok(MessageFuture { waiting_reply_to })
}

#[inline]
pub fn create_program_for_reply(
    code_id: CodeId,
    salt: &[u8],
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
    gas_limit: Option<GasUnit>,
    reply_deposit: Option<GasUnit>,
    reply_hook: Option<Box<dyn FnOnce() + Send + 'static>>,
) -> Result<(MessageFuture, ActorId), ::gstd::errors::Error> {
    #[cfg(not(feature = "ethexe"))]
    let (waiting_reply_to, program_id) = if let Some(gas_limit) = gas_limit {
        crate::ok!(::gcore::prog::create_program_with_gas(
            code_id, salt, payload, gas_limit, value
        ))
    } else {
        crate::ok!(::gcore::prog::create_program(code_id, salt, payload, value))
    };
    #[cfg(feature = "ethexe")]
    let (waiting_reply_to, program_id) =
        crate::ok!(::gcore::prog::create_program(code_id, salt, payload, value));

    #[cfg(not(feature = "ethexe"))]
    if let Some(reply_deposit) = reply_deposit {
        let deposited = ::gcore::exec::reply_deposit(waiting_reply_to, reply_deposit).is_ok();
        if deposited && let Some(reply_hook) = reply_hook {
            reply_hooks().register(waiting_reply_to, reply_hook);
        }
    }

    signals().register_signal(waiting_reply_to, wait);

    Ok((MessageFuture { waiting_reply_to }, program_id))
}

/// Default reply handler.
#[inline]
pub fn handle_reply_with_hook() {
    let reply_to = ::gcore::msg::reply_to().expect("Shouldn't be called with incorrect context");

    signals().record_reply(&reply_to);

    #[cfg(not(feature = "ethexe"))]
    reply_hooks().execute_and_remove(&reply_to);

    // #[cfg(feature = "ethexe")]
    // let _ = replied_to;
}

/// Default signal handler.
#[cfg(not(feature = "ethexe"))]
#[inline]
pub fn handle_signal() {
    let msg_id = ::gcore::msg::signal_from().expect(
        "`gstd::async_runtime::handle_signal()` must be called only in `handle_signal` entrypoint",
    );

    // critical::take_and_execute();

    tasks().remove(&msg_id);
    reply_hooks().remove(&msg_id)
}

pub fn poll(message_id: &MessageId, cx: &mut Context<'_>) -> Poll<Result<Vec<u8>, Error>> {
    signals().poll(message_id, cx)
}

pub fn is_terminated(message_id: &MessageId) -> bool {
    !signals().waits_for(message_id)
}
