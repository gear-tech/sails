use super::*;
use crate::collections::{BinaryHeap, HashMap};
use core::{
    cmp::Reverse,
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

/// Matches a task to a some message in order to avoid duplicate execution
/// of code that was running before the program was interrupted by `wait`.
///
/// The [`Task`] lifecycle matches to the single message processing in the `handle()` entry-point
/// and ends when all internal futures are resolved or `handle_signal()` received for this `message_id`.
pub struct Task {
    future: LocalBoxFuture<'static, ()>,
    locks: BinaryHeap<(Reverse<Lock>, Option<MessageId>)>,
    #[cfg(not(feature = "ethexe"))]
    critical_hook: Option<Box<dyn FnOnce(MessageId)>>,
}

impl Task {
    fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        Self {
            future: future.boxed_local(),
            locks: Default::default(),
            #[cfg(not(feature = "ethexe"))]
            critical_hook: None,
        }
    }

    /// Stores the lock associated with an outbound reply, keeping it ordered by deadline.
    ///
    /// - pushes `(lock, Some(reply_to))` into the binary heap so the task can efficiently retrieve the
    ///   earliest lock when deciding how long to sleep.
    ///
    /// # Context
    /// Called from [`WakeSignals::register_signal`] when the [`message_loop`] schedules a reply wait.
    #[inline]
    fn insert_lock(&mut self, reply_to: MessageId, lock: Lock) {
        self.locks.push((Reverse(lock), Some(reply_to)));
    }

    /// Tracks a sleep-specific lock (without a reply identifier) for the inbound message.
    ///
    /// # Context
    /// Used when the task needs to suspend itself via `exec::wait_*` without tying the wait to a
    /// particular reply id.
    fn insert_sleep(&mut self, lock: Lock) {
        self.locks.push((Reverse(lock), None));
    }

    /// Returns the earliest lock still awaiting completion, removing stale or cleared entries.
    ///
    /// # Context
    /// Called from [`message_loop`] whenever the user future remains pending after polling.
    #[inline]
    fn next_lock(&mut self, now: BlockNumber) -> Option<Lock> {
        let signals_map = signals();
        while let Some((Reverse(lock), reply_to)) = self.locks.peek() {
            // 1. skip and remove expired
            if now >= lock.deadline() {
                self.locks.pop();
                continue;
            }
            // 2. skip and remove if not waits for reply_to
            if let Some(reply_to) = reply_to
                && !signals_map.waits_for(reply_to)
            {
                self.locks.pop();
                continue;
            }
            // 3. keep lock in `self.locks` for `WakeSignal::Pending` in case of `clear_signals`
            return Some(*lock);
        }
        None
    }

    /// Removes all outstanding reply locks from the signal registry without waiting on them.
    ///
    /// - iterates every stored `(_, reply_to)` pair and asks [`WakeSignals`] to drop the wake entry;
    /// - used as part of task teardown to avoid keeping stale replies alive.
    ///
    /// # Context
    /// Called from [`handle_signal`].
    #[cfg(not(feature = "ethexe"))]
    #[inline]
    fn clear_signals(&self) {
        let now = Syscall::block_height();
        let signals_map = signals();
        self.locks.iter().for_each(|(_, reply_to)| {
            if let Some(reply_to) = reply_to {
                // set the `WakeSignal::Timeout` for further processing in `handle_reply`
                signals_map.record_timeout(*reply_to, now);
            }
        });
    }
}

/// Sets a critical hook.
///
/// # Context
/// If called in the `handle_reply` or `handle_signal` entrypoints.
///
/// # SAFETY
/// Ensure that sufficient `gstd::Config::SYSTEM_RESERVE` is set in your
/// program, as this gas is locked during each async call to provide resources
/// for hook execution in case it is triggered.
#[cfg(not(feature = "ethexe"))]
pub fn set_critical_hook<F: FnOnce(MessageId) + 'static>(f: F) {
    if msg::reply_code().is_ok() {
        panic!("`gstd::critical::set_hook()` must not be called in `handle_reply` entrypoint")
    }

    if msg::signal_code().is_ok() {
        panic!("`gstd::critical::set_hook()` must not be called in `handle_signal` entrypoint")
    }
    let message_id = Syscall::message_id();

    tasks()
        .get_mut(&message_id)
        .expect("A message task must exist")
        .critical_hook = Some(Box::new(f));
}

/// Drives asynchronous handling for the currently executing inbound message.
///
/// - locates or creates the `Task` holding the user future for the current message id;
/// - polls the future once and, if it completes, tears down the bookkeeping;
/// - when the future stays pending, arms the shortest wait lock so the runtime suspends until a wake.
///
/// # Context
/// Called from the contract's `handle` entry point while [`message_loop`] runs single-threaded inside the
/// actor. It must be invoked exactly once per incoming message to advance the async state machine.
///
/// # Panics
/// Panics propagated from the user future bubble up, and the function will panic if no wait lock is
/// registered when a pending future requests suspension, signalling a contract logic bug.
#[inline]
pub fn message_loop<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let msg_id = Syscall::message_id();
    let tasks_map = tasks();
    let task = tasks_map.entry(msg_id).or_insert_with(|| {
        #[cfg(not(feature = "ethexe"))]
        {
            Syscall::system_reserve_gas(gstd::Config::system_reserve())
                .expect("Failed to reserve gas for system signal");
        }
        Task::new(future)
    });

    let completed = {
        let mut cx = Context::from_waker(task::Waker::noop());
        ::gstd::debug!("message_loop: polling future for {msg_id}");
        task.future.as_mut().poll(&mut cx).is_ready()
    };

    if completed {
        tasks_map.remove(&msg_id);
    } else {
        let now = Syscall::block_height();
        task.next_lock(now)
            .expect("Cannot find lock to be waited")
            .wait(now);
    }
}

pub type Payload = Vec<u8>;

/// The [`WakeSignal`] lifecycle corresponds to waiting for a reply to a sent message
/// and ends when `handle_reply()` is received.
///
/// May outlive parent [`Task`] in state [`WakeSignal::Timeout`].
enum WakeSignal {
    Pending {
        message_id: MessageId,
        deadline: BlockNumber,
        reply_hook: Option<Box<dyn FnOnce()>>,
    },
    Ready {
        payload: Payload,
        reply_code: ReplyCode,
    },
    Timeout {
        expected: BlockNumber,
        now: BlockNumber,
        reply_hook: Option<Box<dyn FnOnce()>>,
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

    /// Registers a pending reply for `waiting_reply_to` while the current message is being processed.
    ///
    /// - stores pending state (and an optional reply hook) so `poll`/`record_reply` can resolve it later;
    /// - records the lock deadline for timeout detection and attaches the lock to the owning [`Task`] so
    ///   wait bookkeeping stays consistent.
    ///
    /// # Context
    /// Called from helpers such as `send_bytes_for_reply` / `create_program_for_reply` while the message
    /// handler executes inside [`message_loop`] in `handle()` entry point, see [Gear Protocol](https://wiki.vara.network/docs/build/introduction).
    /// The current `message_id` is read from the runtime and used to fetch the associated `Task` entry.
    ///
    /// # Panics
    /// Panics if the `Task` for the current `message_id` cannot be found, which indicates the function
    /// was invoked outside the [`message_loop`] context (programmer error).
    pub fn register_signal(
        &mut self,
        waiting_reply_to: MessageId,
        lock: locks::Lock,
        reply_hook: Option<Box<dyn FnOnce()>>,
    ) {
        let message_id = Syscall::message_id();
        let deadline = lock.deadline();

        self.signals.insert(
            waiting_reply_to,
            WakeSignal::Pending {
                message_id,
                deadline,
                reply_hook,
            },
        );

        // ::gstd::debug!(
        //     "register_signal: add lock for reply_to {waiting_reply_to} in message {message_id}"
        // );
        tasks()
            .get_mut(&message_id)
            .expect("A message task must exist")
            .insert_lock(waiting_reply_to, lock);
    }

    /// Processes an incoming reply for `reply_to` and transitions the stored wake state.
    ///
    /// - upgrades the [`WakeSignal::Pending`] entry to [`WakeSignal::Ready`], capturing payload and reply code;
    /// - executes the optional reply hook once the reply becomes available.
    /// - for the [`WakeSignal::Timeout`] entry executes the optional reply hook and remove entry;
    ///
    /// # Context
    /// Invoked by [`handle_reply_with_hook`] when a reply arrives during `handle_reply()` execution. The
    /// runtime supplies `reply_to`.
    ///
    /// # Panics
    /// Panics if it encounters an already finalised entry [`WakeSignal::Ready`] or the associated task is
    /// missing. Both scenarios indicate logic bugs or duplicate delivery.
    pub fn record_reply(&mut self, reply_to: &MessageId) {
        if let hashbrown::hash_map::EntryRef::Occupied(mut entry) = self.signals.entry_ref(reply_to)
        {
            match entry.get_mut() {
                WakeSignal::Pending {
                    message_id,
                    deadline: _,
                    reply_hook,
                } => {
                    let message_id = *message_id;
                    let reply_hook = reply_hook.take();
                    // replase entry with `WakeSignal::Ready`
                    _ = entry.insert(WakeSignal::Ready {
                        payload: Syscall::read_bytes().expect("Failed to read bytes"),
                        reply_code: Syscall::reply_code()
                            .expect("Shouldn't be called with incorrect context"),
                    });
                    ::gstd::debug!(
                        "record_reply: remove lock for reply_to {reply_to} in message {message_id}"
                    );
                    // wake message loop after receiving reply
                    ::gcore::exec::wake(message_id).expect("Failed to wake the message");

                    // execute reply hook
                    if let Some(f) = reply_hook {
                        f()
                    }
                }
                WakeSignal::Timeout { reply_hook, .. } => {
                    // execute reply hook and remove entry
                    if let Some(f) = reply_hook.take() {
                        f()
                    }
                    _ = entry.remove();
                }
                WakeSignal::Ready { .. } => panic!("A reply has already received"),
            };
        } else {
            ::gstd::debug!(
                "A message has received a reply though it wasn't to receive one, or a processed message has received a reply"
            );
        }
    }

    /// Marks a pending reply as timed out and preserves context for later handling.
    ///
    /// - upgrades a [`WakeSignal::Pending`] entry to [`WakeSignal::Timeout`], capturing when the reply was expected
    ///   and when the timeout was detected;
    /// - retains the optional reply hook so it can still be executed if a late reply arrives and reuses the
    ///   stored state when `record_reply` is called afterwards.
    ///
    /// # Context
    /// Triggered from [`Task::clear_signals`].
    #[cfg(not(feature = "ethexe"))]
    pub fn record_timeout(&mut self, reply_to: MessageId, now: BlockNumber) {
        if let hashbrown::hash_map::Entry::Occupied(mut entry) = self.signals.entry(reply_to)
            && let WakeSignal::Pending {
                reply_hook,
                deadline,
                ..
            } = entry.get_mut()
        {
            let expected = *deadline;
            // move `reply_hook` to `WakeSignal::Timeout` state
            let reply_hook = reply_hook.take();
            entry.insert(WakeSignal::Timeout {
                expected,
                now,
                reply_hook,
            });
        } else {
            ::gstd::debug!("A message has timed out after reply");
        }
    }

    pub fn waits_for(&self, reply_to: &MessageId) -> bool {
        self.signals
            .get(reply_to)
            .is_some_and(|signal| !matches!(signal, WakeSignal::Timeout { .. }))
    }

    /// Polls the stored wake signal for `reply_to`, returning the appropriate future state.
    ///
    /// - inspects the current `WakeSignal` variant, promoting pending entries whose deadline has passed to
    ///   [`WakeSignal::Timeout`];
    /// - returns `Pending`, a `Ready` payload, or propagates a timeout error; when `Ready`, the entry is
    ///   removed so subsequent polls observe completion.
    ///
    /// # Context
    /// Called by [`MessageFuture::poll`] (and any wrappers) while a consumer awaits a reply produced by
    /// [`message_loop`]. It runs on the same execution thread and must be non-blocking.
    ///
    /// # Panics
    /// Panics if the signal was never registered for `reply_to`, which indicates misuse of the async API
    /// (polling without having called one of the [`send_bytes_for_reply`]/[`create_program_for_reply`] methods first).
    pub fn poll(
        &mut self,
        reply_to: &MessageId,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Vec<u8>, Error>> {
        let hashbrown::hash_map::EntryRef::Occupied(mut entry) = self.signals.entry_ref(reply_to)
        else {
            panic!("Poll not registered feature")
        };

        match entry.get_mut() {
            WakeSignal::Pending {
                deadline,
                reply_hook,
                ..
            } => {
                let now = Syscall::block_height();
                let expected = *deadline;
                if now >= expected {
                    let reply_hook = reply_hook.take();
                    _ = entry.insert(WakeSignal::Timeout {
                        expected,
                        now,
                        reply_hook,
                    });
                    Poll::Ready(Err(Error::Timeout(expected, now)))
                } else {
                    Poll::Pending
                }
            }
            WakeSignal::Timeout { expected, now, .. } => {
                // DO NOT remove entry if `WakeSignal::Timeout`
                // will be removed in `record_reply`
                Poll::Ready(Err(Error::Timeout(*expected, *now)))
            }
            WakeSignal::Ready { .. } => {
                // remove entry if `WakeSignal::Ready`
                let WakeSignal::Ready {
                    payload,
                    reply_code,
                } = entry.remove()
                else {
                    // SAFETY: checked in the code above.
                    unsafe { hint::unreachable_unchecked() }
                };
                match reply_code {
                    ReplyCode::Success(_) => Poll::Ready(Ok(payload)),
                    ReplyCode::Error(reason) => {
                        Poll::Ready(Err(Error::ErrorReply(payload.into(), reason)))
                    }
                    ReplyCode::Unsupported => Poll::Ready(Err(Error::UnsupportedReply(payload))),
                }
            }
        }
    }
}

pub struct MessageFuture {
    /// A message identifier for an expected reply.
    ///
    /// This identifier is generated by the corresponding send function (e.g.
    /// [`gcore::msg::send`](::gcore::msg::send)).
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
pub fn send_for_reply<E: Encode>(
    destination: ActorId,
    payload: E,
    value: ValueUnit,
) -> Result<MessageFuture, ::gstd::errors::Error> {
    let size = Encode::encoded_size(&payload);
    stack_buffer::with_byte_buffer(size, |buffer: &mut [mem::MaybeUninit<u8>]| {
        let mut buffer_writer = MaybeUninitBufferWriter::new(buffer);
        Encode::encode_to(&payload, &mut buffer_writer);
        buffer_writer.with_buffer(|buffer| {
            send_bytes_for_reply(
                destination,
                buffer,
                value,
                Default::default(),
                #[cfg(not(feature = "ethexe"))]
                None,
                #[cfg(not(feature = "ethexe"))]
                None,
                #[cfg(not(feature = "ethexe"))]
                None,
            )
        })
    })
}

#[cfg(not(feature = "ethexe"))]
#[inline]
pub fn send_bytes_for_reply(
    destination: ActorId,
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
    gas_limit: Option<GasUnit>,
    reply_deposit: Option<GasUnit>,
    reply_hook: Option<Box<dyn FnOnce()>>,
) -> Result<MessageFuture, ::gstd::errors::Error> {
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

    if let Some(reply_deposit) = reply_deposit {
        _ = ::gcore::exec::reply_deposit(waiting_reply_to, reply_deposit);
    }

    signals().register_signal(waiting_reply_to, wait, reply_hook);

    Ok(MessageFuture { waiting_reply_to })
}

#[cfg(feature = "ethexe")]
#[inline]
pub fn send_bytes_for_reply(
    destination: ActorId,
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
) -> Result<MessageFuture, ::gstd::errors::Error> {
    let waiting_reply_to = crate::ok!(::gcore::msg::send(destination, payload, value));

    signals().register_signal(waiting_reply_to, wait, None);

    Ok(MessageFuture { waiting_reply_to })
}

#[cfg(not(feature = "ethexe"))]
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn create_program_for_reply(
    code_id: CodeId,
    salt: &[u8],
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
    gas_limit: Option<GasUnit>,
    reply_deposit: Option<GasUnit>,
    reply_hook: Option<Box<dyn FnOnce()>>,
) -> Result<(MessageFuture, ActorId), ::gstd::errors::Error> {
    let (waiting_reply_to, program_id) = if let Some(gas_limit) = gas_limit {
        crate::ok!(::gcore::prog::create_program_with_gas(
            code_id, salt, payload, gas_limit, value
        ))
    } else {
        crate::ok!(::gcore::prog::create_program(code_id, salt, payload, value))
    };

    if let Some(reply_deposit) = reply_deposit {
        _ = ::gcore::exec::reply_deposit(waiting_reply_to, reply_deposit);
    }

    signals().register_signal(waiting_reply_to, wait, reply_hook);

    Ok((MessageFuture { waiting_reply_to }, program_id))
}

#[cfg(feature = "ethexe")]
#[inline]
pub fn create_program_for_reply(
    code_id: CodeId,
    salt: &[u8],
    payload: &[u8],
    value: ValueUnit,
    wait: Lock,
) -> Result<(MessageFuture, ActorId), ::gstd::errors::Error> {
    let (waiting_reply_to, program_id) =
        crate::ok!(::gcore::prog::create_program(code_id, salt, payload, value));

    signals().register_signal(waiting_reply_to, wait, None);

    Ok((MessageFuture { waiting_reply_to }, program_id))
}

/// Default reply handler.
#[inline]
pub fn handle_reply_with_hook() {
    let reply_to = Syscall::reply_to().expect("Shouldn't be called with incorrect context");

    signals().record_reply(&reply_to);
}

/// Default signal handler.
#[cfg(not(feature = "ethexe"))]
#[inline]
pub fn handle_signal() {
    let msg_id = Syscall::signal_from().expect(
        "`gstd::async_runtime::handle_signal()` must be called only in `handle_signal` entrypoint",
    );
    // Remove Task and all associated signals, execute critical hook
    if let Some(mut task) = tasks().remove(&msg_id) {
        if let Some(critical_hook) = task.critical_hook.take() {
            critical_hook(msg_id);
        }
        task.clear_signals();
    }
}

pub fn poll(message_id: &MessageId, cx: &mut Context<'_>) -> Poll<Result<Vec<u8>, Error>> {
    signals().poll(message_id, cx)
}

pub fn is_terminated(message_id: &MessageId) -> bool {
    !signals().waits_for(message_id)
}

struct MessageSleepFuture {
    deadline: BlockNumber,
}

impl MessageSleepFuture {
    fn new(deadline: BlockNumber) -> Self {
        Self { deadline }
    }
}

impl Unpin for MessageSleepFuture {}

impl Future for MessageSleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = Syscall::block_height();

        if now >= self.deadline {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Delays message execution in asynchronous way for the specified number of blocks.
///
/// It works pretty much like the [`gcore::exec::wait_for`] function, but
/// allows to continue execution after the delay in the same handler. It is
/// worth mentioning that the program state gets persisted inside the call, and
/// the execution resumes with potentially different state.
pub fn sleep_for(block_count: BlockCount) -> impl Future<Output = ()> {
    let message_id = Syscall::message_id();
    let lock = Lock::exactly(block_count);
    tasks()
        .get_mut(&message_id)
        .expect("A message task must exist")
        .insert_sleep(lock);
    MessageSleepFuture::new(lock.deadline())
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gstd::syscalls::Syscall;
    use core::{sync::atomic::AtomicU64, task, task::Context};

    static MSG_ID: AtomicU64 = AtomicU64::new(1);

    fn msg_id() -> MessageId {
        MessageId::from(MSG_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst))
    }

    fn set_context(message_id: MessageId, block_height: u32) {
        Syscall::with_message_id(message_id);
        Syscall::with_block_height(block_height);
    }

    #[test]
    fn task_insert_lock_adds_entry() {
        set_context(msg_id(), 10);

        let mut task = Task::new(async {});
        let reply_to = msg_id();
        let lock = Lock::up_to(3);

        task.insert_lock(reply_to, lock);
        task.insert_lock(msg_id(), Lock::exactly(5));

        let Some((Reverse(next_lock), next_reply_to)) = task.locks.peek() else {
            unreachable!()
        };

        assert_eq!(task.locks.len(), 2);
        assert_eq!(Some(&reply_to), next_reply_to.as_ref());
        assert_eq!(next_lock, &lock);
    }

    #[test]
    fn signals_poll_converts_pending_into_timeout() {
        let message_id = msg_id();
        set_context(message_id, 20);
        tasks().insert(message_id, Task::new(async {}));

        let reply_to = msg_id();
        let lock = Lock::up_to(5);
        let deadline = lock.deadline();

        signals().register_signal(reply_to, lock, None);

        Syscall::with_block_height(deadline - 1);
        let mut cx = Context::from_waker(task::Waker::noop());
        assert!(matches!(signals().poll(&reply_to, &mut cx), Poll::Pending));

        Syscall::with_block_height(deadline);
        let mut cx = Context::from_waker(task::Waker::noop());
        match signals().poll(&reply_to, &mut cx) {
            Poll::Ready(Err(Error::Timeout(expected, now))) => {
                assert_eq!(expected, deadline);
                assert_eq!(now, deadline);
            }
            other => panic!("expected timeout, got {other:?}"),
        }

        signals().signals.remove(&reply_to);
        tasks().remove(&message_id);
    }

    #[test]
    fn task_remove_signal_skip_not_waited_lock() {
        let message_id = msg_id();
        set_context(message_id, 30);
        let reply_to = msg_id();
        let lock = Lock::up_to(5);

        tasks().insert(message_id, Task::new(async {}));
        let task = tasks().get_mut(&message_id).unwrap();
        signals().register_signal(reply_to, lock, None);

        assert_eq!(1, task.locks.len());

        signals().signals.remove(&reply_to);

        assert_eq!(1, task.locks.len());
        assert_eq!(None, task.next_lock(31));
        tasks().remove(&message_id);
    }

    #[test]
    fn task_insert_sleep_adds_entry_without_reply() {
        let message_id = msg_id();
        set_context(message_id, 40);

        let mut task = Task::new(async {});
        let lock = Lock::exactly(4);

        task.insert_sleep(lock);
        assert_eq!(Some(lock), task.next_lock(42));
        assert_eq!(None, task.next_lock(lock.deadline()));
    }
}
