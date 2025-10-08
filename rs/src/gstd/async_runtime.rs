use super::*;
use crate::collections::HashMap;
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

/// Matches a task to a some message in order to avoid duplicate execution
/// of code that was running before the program was interrupted by `wait`.
pub struct Task {
    future: LocalBoxFuture<'static, ()>,
    reply_to_locks: Vec<(MessageId, locks::Lock)>,
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
            reply_to_locks: Vec::new(),
            #[cfg(not(feature = "ethexe"))]
            critical_hook: None,
        }
    }

    /// Registers the wait/timeout lock associated with an outgoing message reply.
    ///
    /// - stores the `(reply_to, lock)` pair so the task can later detect timeouts;
    ///
    /// # Context
    /// Called exclusively from [WakeSignals::register_signal] while the outer `message_loop`
    /// prepares to await a reply for the current inbound message.
    #[inline]
    fn insert_lock(&mut self, reply_to: MessageId, lock: locks::Lock) {
        self.reply_to_locks.push((reply_to, lock));
    }

    /// Removes the stored lock for the given reply identifier, if present.
    ///
    /// - searches for the `(reply_to, lock)` pair and removes it.
    ///
    /// # Context
    /// Called from [WakeSignals::record_reply] once a response is received, as well
    /// as during cleanup when a task finishes.
    #[inline]
    fn remove_lock(&mut self, reply_to: &MessageId) {
        self.reply_to_locks
            .iter()
            .position(|(mid, _)| mid == reply_to)
            .map(|index| self.reply_to_locks.swap_remove(index));
    }

    /// Notifies the signal registry about replies that have exceeded their deadlines.
    ///
    /// - scans all tracked locks, extracting those whose deadlines are at or before `now`;
    /// - informs [WakeSignals] about the timeout so it can update the wake state and
    ///   potentially execute a deferred reply hook.
    ///
    /// # Context
    /// Invoked from [message_loop] before polling the user future to ensure that timeouts
    /// are processed promptly for the current message.
    #[inline]
    fn signal_reply_timeout(&mut self, now: BlockNumber) {
        let signals_map = signals();

        self.reply_to_locks
            .extract_if(.., |(_, lock)| now >= lock.deadline())
            .for_each(|(reply_to, lock)| {
                signals_map.record_timeout(reply_to, lock.deadline(), now);
                // ::gstd::debug!(
                //     "signal_reply_timeout: remove lock for reply_to {reply_to} in message due to timeout"
                // );
            });
    }

    /// Arms the most urgent wait lock so the executor suspends until a wake signal.
    ///
    /// - finds the lock with the smallest deadline and delegates to `Lock::wait` to set
    ///   the runtime suspension point.
    ///
    /// # Context
    /// Called from [message_loop] whenever the user future remains pending after polling.
    ///
    /// # Panics
    /// Panics if no locks are registered for the current task, which indicates a logic error
    /// (e.g. awaiting a reply without having registered one).
    #[inline]
    fn next_lock(&self) -> Option<&Lock> {
        self.reply_to_locks
            .iter()
            .map(|(_, lock)| lock)
            .min_by(|lock1, lock2| lock1.cmp(lock2))
    }

    /// Removes all outstanding reply locks from the signal registry without waiting on them.
    ///
    /// # What it does
    /// - iterates every stored `(reply_to, _)` pair and asks [`WakeSignals`] to drop the wake entry;
    /// - used as part of task teardown to avoid keeping stale replies alive.
    ///
    /// # Context
    /// Called from [`handle_signal`].
    #[cfg(not(feature = "ethexe"))]
    #[inline]
    fn clear(&self) {
        let signals_map = signals();
        self.reply_to_locks.iter().for_each(|(reply_to, _)| {
            signals_map.remove(reply_to);
        });
    }
}

/// Sets a critical hook.
///
/// # Panics
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
/// Called from the contract's `handle` entry point while [message_loop] runs single-threaded inside the
/// actor. It must be invoked exactly once per incoming message to advance the async state machine.
///
/// # Panics
/// Panics propagated from the user future bubble up, and the function will panic if no wait lock is
/// registered when a pending future requests suspension (see `Task::wait`), signalling a contract logic bug.
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
    } else {
        task.next_lock()
            .expect("Cannot find lock to be waited")
            .wait(current_block);
    }
}

pub type Payload = Vec<u8>;

enum WakeSignal {
    Pending {
        message_id: MessageId,
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
    /// - attaches the provided `lock` to the owning `Task` so wait/timeout bookkeeping stays consistent.
    ///
    /// # Context
    /// Called from helpers such as `send_bytes_for_reply` / `create_program_for_reply` while the message
    /// handler executes inside [message_loop] in `handle()` entry point, see [Gear Protocol](https://wiki.vara.network/docs/build/introduction).
    /// The current `message_id` is read from the runtime and used to fetch the associated `Task` entry.
    ///
    /// # Panics
    /// Panics if the `Task` for the current `message_id` cannot be found, which indicates the function
    /// was invoked outside the [message_loop] context (programmer error).
    pub fn register_signal(
        &mut self,
        waiting_reply_to: MessageId,
        lock: locks::Lock,
        reply_hook: Option<Box<dyn FnOnce()>>,
    ) {
        let message_id = Syscall::message_id();

        self.signals.insert(
            waiting_reply_to,
            WakeSignal::Pending {
                message_id,
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
    /// - upgrades the pending entry to `Ready`, capturing payload and reply code;
    /// - detaches the wait lock from the owning `Task`, then wakes the suspended message loop;
    /// - executes the optional reply hook once the reply becomes available.
    ///
    /// # Context
    /// Invoked by `handle_reply_with_hook` when a reply arrives during `handle_reply()` execution. The
    /// runtime supplies `reply_to`, and this method synchronises bookkeeping before waking the task.
    ///
    /// # Panics
    /// Panics if it encounters an already finalised entry (`WakeSignal::Ready`) or the associated task is
    /// missing. Both scenarios indicate logic bugs or duplicate delivery.
    pub fn record_reply(&mut self, reply_to: &MessageId) {
        if let hashbrown::hash_map::EntryRef::Occupied(mut entry) = self.signals.entry_ref(reply_to)
        {
            match entry.get_mut() {
                WakeSignal::Pending {
                    message_id,
                    reply_hook,
                } => {
                    let message_id = *message_id;
                    let reply_hook = reply_hook.take();
                    // replase entry with `WakeSignal::Ready`
                    _ = entry.insert(WakeSignal::Ready {
                        payload: ::gstd::msg::load_bytes().expect("Failed to load bytes"),
                        reply_code: Syscall::reply_code()
                            .expect("Shouldn't be called with incorrect context"),
                    });
                    ::gstd::debug!(
                        "record_reply: remove lock for reply_to {reply_to} in message {message_id}"
                    );
                    tasks()
                        .get_mut(&message_id)
                        .expect("A message task must exist")
                        .remove_lock(reply_to);
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
    /// - upgrades a `WakeSignal::Pending` entry to `WakeSignal::Timeout`, capturing when the reply was expected
    ///   and when the timeout was detected;
    /// - retains the optional reply hook so it can still be executed if a late reply arrives and reuses the
    ///   stored state when `record_reply` is called afterwards.
    ///
    /// # Context
    /// Triggered from `Task::signal_reply_timeout` whenever the runtime observes that a waiting reply exceeded
    /// its deadline while executing inside [message_loop].
    pub fn record_timeout(&mut self, reply_to: MessageId, expected: BlockNumber, now: BlockNumber) {
        if let hashbrown::hash_map::Entry::Occupied(mut entry) = self.signals.entry(reply_to)
            && let WakeSignal::Pending { reply_hook, .. } = entry.get_mut()
        {
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
        self.signals.contains_key(reply_to)
    }

    /// Polls the stored wake signal for `reply_to`, returning the appropriate future state.
    ///
    /// # What it does
    /// - inspects the current `WakeSignal` variant and returns `Pending`, a `Ready` payload, or propagates
    ///   a timeout error; when `Ready`, the entry is removed so subsequent polls observe completion.
    ///
    /// # Context
    /// Called by [MessageFuture::poll] (and any wrappers) while a consumer awaits a reply produced by
    /// [message_loop]. It runs on the same execution thread and must be non-blocking.
    ///
    /// # Panics
    /// Panics if the signal was never registered for `reply_to`, which indicates misuse of the async API
    /// (polling without having called one of the [send_bytes_for_reply]/[create_program_for_reply] methods first).
    pub fn poll(
        &mut self,
        reply_to: &MessageId,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Vec<u8>, Error>> {
        let hashbrown::hash_map::EntryRef::Occupied(entry) = self.signals.entry_ref(reply_to)
        else {
            panic!("Poll not registered feature")
        };
        match entry.get() {
            WakeSignal::Pending { .. } => Poll::Pending,
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

    #[cfg(not(feature = "ethexe"))]
    fn remove(&mut self, reply_to: &MessageId) -> Option<WakeSignal> {
        self.signals.remove(reply_to)
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
        task.clear()
    }
}

pub fn poll(message_id: &MessageId, cx: &mut Context<'_>) -> Poll<Result<Vec<u8>, Error>> {
    signals().poll(message_id, cx)
}

pub fn is_terminated(message_id: &MessageId) -> bool {
    !signals().waits_for(message_id)
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gstd::locks;
    use crate::gstd::syscalls::Syscall;

    fn set_context(message_id: MessageId, block_height: u32) {
        Syscall::with_message_id(message_id);
        Syscall::with_block_height(block_height);
    }

    #[test]
    fn insert_lock_adds_entry() {
        set_context(MessageId::from(1), 10);

        let mut task = Task::new(async {});
        let reply_to = MessageId::from(2);
        let lock = locks::Lock::up_to(3);

        task.insert_lock(reply_to, lock);

        assert_eq!(task.reply_to_locks.len(), 1);
        assert_eq!(task.reply_to_locks[0].0, reply_to);
        assert_eq!(task.reply_to_locks[0].1.deadline(), lock.deadline());
    }

    #[test]
    fn remove_lock_drops_matching_entry() {
        set_context(MessageId::from(3), 5);

        let mut task = Task::new(async {});
        let reply_to = MessageId::from(4);
        let lock = locks::Lock::up_to(1);

        task.insert_lock(reply_to, lock);
        task.remove_lock(&reply_to);

        assert!(task.reply_to_locks.is_empty());
    }

    #[test]
    fn signal_reply_timeout_promotes_expired_locks() {
        // arrange
        let message_id = MessageId::from(5);
        set_context(message_id, 20);
        let reply_to = MessageId::from(6);
        let lock = locks::Lock::up_to(5);
        let signals_map = signals();
        let task = Task::new(async {});

        tasks().insert(message_id, task);
        let task = tasks().get_mut(&message_id).unwrap();
        signals_map.register_signal(reply_to, lock, None);

        // act
        task.signal_reply_timeout(30);

        // assert
        match signals_map.signals.get(&reply_to) {
            Some(WakeSignal::Timeout { expected, now, .. }) => {
                assert_eq!(*expected, lock.deadline());
                assert_eq!(*now, 30);
            }
            _ => unreachable!(),
        }

        assert!(task.reply_to_locks.is_empty());
    }
}
