use super::*;
use crate::collections::{BinaryHeap, HashMap};
use core::{
    cmp::Reverse,
    hash::{BuildHasherDefault, Hasher},
    pin::Pin,
    task::{Context, Poll},
};
use futures::future::{FusedFuture, FutureExt as _, LocalBoxFuture};
use gstd::{BlockNumber, errors::Error};

/// Identity-hasher for `MessageId`. `MessageId` is itself a 32-byte
/// cryptographic hash, so the first 8 bytes are already uniformly
/// distributed — running them through a general-purpose hash function is
/// wasted gas. Saves the 32-byte mixing on every HashMap op.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // MessageId encode_to writes 32 bytes; we read the first 8.
        let n = bytes.len().min(8);
        let mut buf = [0u8; 8];
        buf[..n].copy_from_slice(&bytes[..n]);
        self.0 = u64::from_ne_bytes(buf);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

type IdMap<V> = HashMap<MessageId, V, BuildHasherDefault<IdHasher>>;

fn tasks() -> &'static mut IdMap<Task> {
    static mut MAP: Option<IdMap<Task>> = None;
    unsafe { &mut *core::ptr::addr_of_mut!(MAP) }.get_or_insert_with(IdMap::default)
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
                // If the user future never polled the matching MessageFuture,
                // its signal entry would sit in Pending forever. Push it
                // through the same timeout transition `WakeSignals::poll`
                // would apply, so the global signals map stays bounded.
                if let Some(reply_to) = reply_to {
                    signals_map.record_timeout(*reply_to, now);
                }
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
                // set the `WakeSignal::Expired` for further processing in `handle_reply`
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
        panic!(
            "`sails_rs::gstd::set_critical_hook()` must not be called in `handle_reply` entrypoint"
        )
    }

    if msg::signal_code().is_ok() {
        panic!(
            "`sails_rs::gstd::set_critical_hook()` must not be called in `handle_signal` entrypoint"
        )
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
            Syscall::system_reserve_gas(gstd::Config::system_reserve()).unwrap();
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
        task.next_lock(now).unwrap().wait(now);
    }
}

pub type Payload = Vec<u8>;

/// The [`WakeSignal`] lifecycle corresponds to waiting for a reply to a sent message
/// and ends when `handle_reply()` is received.
///
/// May outlive parent [`Task`] in [`WakeSignal::Expired`] state.
///
/// Can be created in [`WakeSignal::Expired`] state if there is no [`Task`] to await.
enum WakeSignal {
    /// Reply is still pending; tracks origin message, deadline, and optional hook to run on completion or timeout.
    Pending {
        message_id: MessageId,
        deadline: BlockNumber,
        reply_hook: Option<Box<dyn FnOnce()>>,
    },
    /// Reply handled; captures payload and reply code so the waiting future can resolve.
    Ready {
        payload: Payload,
        reply_code: ReplyCode,
    },
    /// Reply missed its deadline; retains timing data and hook so late arrivals can still be acknowledged.
    Expired {
        expected: BlockNumber,
        now: BlockNumber,
        reply_hook: Option<Box<dyn FnOnce()>>,
    },
}

impl WakeSignal {
    /// Transition a [`WakeSignal::Pending`] signal in place to
    /// [`WakeSignal::Expired`], preserving the optional reply hook. No-op for
    /// already-`Expired`/`Ready` signals.
    ///
    /// Centralizes the `Pending -> Expired` transition shared by
    /// [`WakeSignals::record_timeout`] (natural timeout), [`WakeSignals::poll`]
    /// (deferred timeout) and [`WakeSignals::forget_future`] (cancellation), so
    /// a late reply can still fire the hook and deferred polls observe
    /// `Err(Timeout)` rather than a missing entry.
    #[inline]
    fn expire(&mut self, now: BlockNumber) {
        if let WakeSignal::Pending {
            deadline,
            reply_hook,
            ..
        } = self
        {
            *self = WakeSignal::Expired {
                expected: *deadline,
                now,
                reply_hook: reply_hook.take(),
            };
        }
    }
}

struct WakeSignals {
    signals: IdMap<WakeSignal>,
}

impl WakeSignals {
    pub fn new() -> Self {
        Self {
            signals: IdMap::default(),
        }
    }

    /// Registers a reply wait for `waiting_reply_to` while the current message is being processed.
    ///
    /// - stores [`WakeSignal::Pending`] together with an optional hook so `poll`/`record_reply` can resolve it later;
    /// - records the lock deadline for timeout detection and attaches the lock to the owning [`Task`] for
    ///   consistent wake bookkeeping.
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
        // SAFETY: the task for the current `message_id` is inserted by
        // `message_loop` before the user future driving this call is polled, so
        // it is always present here.
        unsafe { tasks().get_mut(&message_id).unwrap_unchecked() }
            .insert_lock(waiting_reply_to, lock);
    }

    /// Registers a reply hook for `waiting_reply_to` without creating a tracked wait.
    ///
    /// - stores a [`WakeSignal::Expired`] entry so `record_reply` will still execute the hook if a reply
    ///   arrives later;
    /// - intended for one-way sends that want to observe replies from outside [`message_loop`].
    ///
    /// # Context
    /// Called from [`send_one_way`] and other synchronous helpers; may be invoked outside [`message_loop`].
    #[cfg(not(feature = "ethexe"))]
    #[inline]
    pub fn register_hook(
        &mut self,
        waiting_reply_to: MessageId,
        reply_hook: Option<Box<dyn FnOnce()>>,
    ) {
        if let Some(reply_hook) = reply_hook {
            let now = Syscall::block_height();
            self.signals.insert(
                waiting_reply_to,
                WakeSignal::Expired {
                    expected: now,
                    now,
                    reply_hook: Some(reply_hook),
                },
            );
        }
    }

    /// Processes an incoming reply for `reply_to` and transitions the stored wake state.
    ///
    /// - upgrades the [`WakeSignal::Pending`] entry to [`WakeSignal::Ready`], capturing payload and reply code;
    /// - executes the optional reply hook once the reply becomes available.
    /// - for the [`WakeSignal::Expired`] entry executes the optional reply hook and remove entry;
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
                    // replace entry with `WakeSignal::Ready`
                    _ = entry.insert(WakeSignal::Ready {
                        payload: Syscall::read_bytes().unwrap(),
                        // SAFETY: `record_reply` runs only in the `handle_reply`
                        // entrypoint, where the reply context always exists.
                        reply_code: unsafe { Syscall::reply_code().unwrap_unchecked() },
                    });
                    ::gstd::debug!(
                        "record_reply: remove lock for reply_to {reply_to} in message {message_id}"
                    );
                    // wake message loop after receiving reply
                    ::gcore::exec::wake(message_id).unwrap();

                    // execute reply hook
                    if let Some(f) = reply_hook {
                        f()
                    }
                }
                WakeSignal::Expired { reply_hook, .. } => {
                    let reply_hook = reply_hook.take();
                    _ = entry.remove();
                    // execute reply hook and remove entry
                    if let Some(f) = reply_hook {
                        f()
                    }
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
    /// - upgrades a [`WakeSignal::Pending`] entry to [`WakeSignal::Expired`], capturing when the reply was expected
    ///   and when the timeout was detected;
    /// - retains the optional reply hook so it can still be executed if a late reply arrives and reuses the
    ///   stored state when `record_reply` is called afterwards.
    ///
    /// # Context
    /// Triggered from [`Task::clear_signals`].
    pub fn record_timeout(&mut self, reply_to: MessageId, now: BlockNumber) {
        // Always transition to Expired (with or without hook). The entry must
        // stay consumable by a deferred `MessageFuture::poll` — that poll has
        // to observe `Err(Timeout)` instead of panicking on a missing entry.
        // Final cleanup happens in `MessageFuture::drop` when the future is no
        // longer reachable.
        if let Some(signal @ WakeSignal::Pending { .. }) = self.signals.get_mut(&reply_to) {
            signal.expire(now);
        } else {
            ::gstd::debug!("A message has timed out after reply");
        }
    }

    /// Release the entry tied to a `MessageFuture` that is being dropped.
    ///
    /// Called from `MessageFuture::drop`. Reclaims entries that no one will
    /// observe via `poll` anymore; preserves entries that still hold a
    /// reply hook so a late reply can fire it through `record_reply`.
    ///
    /// Cancellation must be symmetric with timeout:
    /// - `Pending { reply_hook: Some, .. }` -> `Expired { reply_hook: Some, .. }`
    ///   (same transition `record_timeout` would do, so the hook survives).
    /// - `Pending { reply_hook: None, .. }` -> removed.
    /// - `Ready` -> removed (payload will never be consumed; the hook, if any,
    ///   already ran inside `record_reply`).
    /// - `Expired { reply_hook: Some, .. }` -> kept.
    /// - `Expired { reply_hook: None, .. }` -> removed.
    pub fn forget_future(&mut self, reply_to: &MessageId) {
        if let hashbrown::hash_map::EntryRef::Occupied(mut entry) = self.signals.entry_ref(reply_to)
        {
            match entry.get_mut() {
                // Cancellation before timeout with a live hook: transition to
                // Expired (same as a natural timeout) so a late reply can still
                // fire the hook and a deferred poll observes `Err(Timeout)`.
                signal @ WakeSignal::Pending {
                    reply_hook: Some(_),
                    ..
                } => signal.expire(Syscall::block_height()),
                // Expired entry still holding a hook: keep it for a late reply.
                WakeSignal::Expired {
                    reply_hook: Some(_),
                    ..
                } => {}
                // Nothing left to observe (Pending/Expired without a hook, or a
                // Ready payload no one will consume): drop the entry.
                _ => {
                    entry.remove();
                }
            }
        }
    }

    pub fn waits_for(&self, reply_to: &MessageId) -> bool {
        self.signals
            .get(reply_to)
            .is_some_and(|signal| !matches!(signal, WakeSignal::Expired { .. }))
    }

    /// Polls the stored wake signal for `reply_to`, returning the appropriate future state.
    ///
    /// - inspects the current `WakeSignal` variant, promoting pending entries whose deadline has passed to
    ///   [`WakeSignal::Expired`];
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
            WakeSignal::Pending { deadline, .. } => {
                let now = Syscall::block_height();
                let expected = *deadline;
                if now >= expected {
                    // Transition to Expired and keep the entry. The future
                    // may be polled again (idempotent timeout) or dropped;
                    // `MessageFuture::drop` does the final cleanup.
                    entry.get_mut().expire(now);
                    Poll::Ready(Err(Error::Timeout(expected, now)))
                } else {
                    Poll::Pending
                }
            }
            WakeSignal::Expired { expected, now, .. } => {
                // Entry persists either for a late reply (if there's a hook)
                // or until the owning `MessageFuture` is dropped.
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

impl Drop for MessageFuture {
    /// Reclaim the `WakeSignals` entry tied to this future.
    ///
    /// `WakeSignals::poll` and `record_timeout` deliberately keep entries
    /// alive so a deferred poll can still observe `Err(Timeout)`. That means
    /// the only safe place to delete an entry that no one will ever poll
    /// again is here, when the future itself is being dropped.
    ///
    /// Entries that retain a reply hook (`WakeSignal::Expired` with
    /// `reply_hook.is_some()`) stay in the map so a late reply can still
    /// fire the hook through `record_reply`.
    fn drop(&mut self) {
        signals().forget_future(&self.waiting_reply_to);
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

#[inline]
pub fn send_one_way(
    destination: ActorId,
    payload: &[u8],
    value: ValueUnit,
    #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
    #[cfg(not(feature = "ethexe"))] reply_deposit: Option<GasUnit>,
    #[cfg(not(feature = "ethexe"))] reply_hook: Option<Box<dyn FnOnce()>>,
) -> Result<MessageId, ::gstd::errors::Error> {
    let waiting_reply_to = crate::ok!(send_bytes(
        destination,
        payload,
        value,
        #[cfg(not(feature = "ethexe"))]
        gas_limit,
        #[cfg(not(feature = "ethexe"))]
        reply_deposit
    ));

    #[cfg(not(feature = "ethexe"))]
    signals().register_hook(waiting_reply_to, reply_hook);

    Ok(waiting_reply_to)
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
    let waiting_reply_to = crate::ok!(send_bytes(
        destination,
        payload,
        value,
        gas_limit,
        reply_deposit
    ));

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
    let waiting_reply_to = crate::ok!(send_bytes(destination, payload, value));

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
    // SAFETY: only called from the `handle_reply` entrypoint, where the reply
    // context always exists.
    let reply_to = unsafe { Syscall::reply_to().unwrap_unchecked() };

    signals().record_reply(&reply_to);
}

/// Default signal handler.
#[cfg(not(feature = "ethexe"))]
#[inline]
pub fn handle_signal() {
    // SAFETY: only called from the `handle_signal` entrypoint, where the signal
    // context always exists.
    let msg_id = unsafe { Syscall::signal_from().unwrap_unchecked() };
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
    // SAFETY: `sleep_for` runs inside the user future, which `message_loop`
    // polls only after inserting the task for the current `message_id`.
    unsafe { tasks().get_mut(&message_id).unwrap_unchecked() }.insert_sleep(lock);
    MessageSleepFuture::new(lock.deadline())
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gstd::syscalls::Syscall;
    use core::{sync::atomic::AtomicU64, task, task::Context};
    use std::sync::Mutex;

    // `tasks()` / `signals()` are `static mut` — correct for the single-threaded
    // WASM runtime, but cargo test runs this module on a thread pool. Serialize
    // every test in here through one mutex so the parallel runner never races
    // on the shared HashMaps. Recover from poisoning so a panicking test doesn't
    // cascade into PoisonError for every subsequent one.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn serial<R>(f: impl FnOnce() -> R) -> R {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        f()
    }

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
        serial(|| {
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
        });
    }

    #[test]
    fn signals_poll_converts_pending_into_timeout() {
        serial(|| {
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
        });
    }

    #[test]
    fn task_remove_signal_skip_not_waited_lock() {
        serial(|| {
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
        });
    }

    #[test]
    fn task_insert_sleep_adds_entry_without_reply() {
        serial(|| {
            let message_id = msg_id();
            set_context(message_id, 40);

            let mut task = Task::new(async {});
            let lock = Lock::exactly(4);

            task.insert_sleep(lock);
            assert_eq!(Some(lock), task.next_lock(42));
            assert_eq!(None, task.next_lock(lock.deadline()));
        });
    }

    /// After a timed-out poll, the entry must stay alive as `Expired` so a
    /// deferred re-poll (or a late reply with a hook) can still observe it.
    /// Final cleanup is `MessageFuture::drop`'s job, not `poll`'s.
    #[test]
    fn signals_poll_keeps_expired_for_deferred_repoll() {
        serial(|| {
            let message_id = msg_id();
            set_context(message_id, 50);
            tasks().insert(message_id, Task::new(async {}));

            let reply_to = msg_id();
            let lock = Lock::up_to(2);
            let deadline = lock.deadline();
            signals().register_signal(reply_to, lock, None);

            Syscall::with_block_height(deadline + 1);
            let mut cx = Context::from_waker(task::Waker::noop());

            // First poll: Pending -> Expired, returns Timeout.
            assert!(matches!(
                signals().poll(&reply_to, &mut cx),
                Poll::Ready(Err(Error::Timeout(..)))
            ));
            assert!(
                matches!(
                    signals().signals.get(&reply_to),
                    Some(WakeSignal::Expired { .. })
                ),
                "entry must remain so a re-poll can observe Timeout"
            );

            // Second poll on the same reply_to: same answer, no panic.
            assert!(matches!(
                signals().poll(&reply_to, &mut cx),
                Poll::Ready(Err(Error::Timeout(..)))
            ));

            signals().signals.remove(&reply_to);
            tasks().remove(&message_id);
        });
    }

    /// `Task::next_lock` must lift the signal out of `Pending` when reaping
    /// the matching expired lock — otherwise the future that polls later
    /// would still see `Pending` past its deadline. The entry stays as
    /// `Expired` so a deferred poll can observe `Err(Timeout)`.
    #[test]
    fn next_lock_expiring_lock_transitions_signal_to_expired() {
        serial(|| {
            let message_id = msg_id();
            set_context(message_id, 100);
            tasks().insert(message_id, Task::new(async {}));

            let reply_to = msg_id();
            let lock = Lock::up_to(2);
            signals().register_signal(reply_to, lock, None);

            // Advance past the deadline WITHOUT polling the MessageFuture
            // (simulates select!/join! where another branch finished first).
            let now = lock.deadline() + 5;
            Syscall::with_block_height(now);

            let task = tasks().get_mut(&message_id).unwrap();
            assert_eq!(task.next_lock(now), None, "expired lock must be popped");

            assert!(
                matches!(
                    signals().signals.get(&reply_to),
                    Some(WakeSignal::Expired { .. })
                ),
                "next_lock must transition Pending -> Expired (not leave Pending, not delete)"
            );

            // Deferred poll: the user future finally awaits this MessageFuture.
            // It must produce Err(Timeout), not panic on a missing entry.
            let mut cx = Context::from_waker(task::Waker::noop());
            assert!(matches!(
                signals().poll(&reply_to, &mut cx),
                Poll::Ready(Err(Error::Timeout(..)))
            ));

            signals().signals.remove(&reply_to);
            tasks().remove(&message_id);
        });
    }

    /// `MessageFuture::drop` must remove a `Pending` entry — the user gave
    /// up before polling, no one will ever observe a reply or timeout.
    #[test]
    fn message_future_drop_removes_pending_entry() {
        serial(|| {
            let message_id = msg_id();
            set_context(message_id, 200);
            tasks().insert(message_id, Task::new(async {}));

            let reply_to = msg_id();
            signals().register_signal(reply_to, Lock::up_to(5), None);
            assert!(signals().signals.contains_key(&reply_to));

            drop(MessageFuture {
                waiting_reply_to: reply_to,
            });

            assert!(
                !signals().signals.contains_key(&reply_to),
                "Pending entry must be removed when MessageFuture is dropped"
            );

            tasks().remove(&message_id);
        });
    }

    /// `MessageFuture::drop` must remove an `Expired` entry that holds no
    /// reply hook — keeping it serves no late-delivery purpose.
    #[test]
    fn message_future_drop_removes_expired_entry_when_no_hook() {
        serial(|| {
            let message_id = msg_id();
            set_context(message_id, 300);
            tasks().insert(message_id, Task::new(async {}));

            let reply_to = msg_id();
            let lock = Lock::up_to(2);
            let deadline = lock.deadline();
            signals().register_signal(reply_to, lock, None);

            // Drive Pending -> Expired (no hook).
            Syscall::with_block_height(deadline + 1);
            let mut cx = Context::from_waker(task::Waker::noop());
            assert!(matches!(
                signals().poll(&reply_to, &mut cx),
                Poll::Ready(Err(Error::Timeout(..)))
            ));
            assert!(matches!(
                signals().signals.get(&reply_to),
                Some(WakeSignal::Expired { .. })
            ));

            drop(MessageFuture {
                waiting_reply_to: reply_to,
            });

            assert!(
                !signals().signals.contains_key(&reply_to),
                "Expired-no-hook entry must be removed when MessageFuture is dropped"
            );

            tasks().remove(&message_id);
        });
    }

    /// Cancellation symmetry: dropping a `MessageFuture` while still
    /// `Pending` with a reply hook must keep the hook alive (transition to
    /// Expired-with-hook). Otherwise, whether `with_reply_hook` fires on a
    /// late reply would depend on whether the future was dropped before or
    /// after the timeout — a footgun.
    #[test]
    fn message_future_drop_preserves_pending_entry_with_hook() {
        serial(|| {
            let message_id = msg_id();
            set_context(message_id, 500);
            tasks().insert(message_id, Task::new(async {}));

            let reply_to = msg_id();
            let lock = Lock::up_to(5);
            let deadline = lock.deadline();
            signals().register_signal(reply_to, lock, Some(Box::new(|| {})));

            // Drop while still well within the deadline — natural timeout has
            // not fired. The hook must survive.
            assert!(Syscall::block_height() < deadline);
            drop(MessageFuture {
                waiting_reply_to: reply_to,
            });

            assert!(
                matches!(
                    signals().signals.get(&reply_to),
                    Some(WakeSignal::Expired {
                        reply_hook: Some(_),
                        ..
                    })
                ),
                "Pending-with-hook drop must transition to Expired-with-hook \
                 so a late reply can still fire the hook"
            );

            signals().signals.remove(&reply_to);
            tasks().remove(&message_id);
        });
    }

    /// Conversely, `MessageFuture::drop` MUST preserve an `Expired` entry
    /// that still holds a reply hook — a late reply via `record_reply` is
    /// the only thing that will ever fire that hook.
    #[test]
    fn message_future_drop_preserves_expired_entry_with_hook() {
        serial(|| {
            let message_id = msg_id();
            set_context(message_id, 400);
            tasks().insert(message_id, Task::new(async {}));

            let reply_to = msg_id();
            let lock = Lock::up_to(2);
            let deadline = lock.deadline();
            signals().register_signal(reply_to, lock, Some(Box::new(|| {})));

            Syscall::with_block_height(deadline + 1);
            let mut cx = Context::from_waker(task::Waker::noop());
            assert!(matches!(
                signals().poll(&reply_to, &mut cx),
                Poll::Ready(Err(Error::Timeout(..)))
            ));

            drop(MessageFuture {
                waiting_reply_to: reply_to,
            });

            assert!(
                matches!(
                    signals().signals.get(&reply_to),
                    Some(WakeSignal::Expired {
                        reply_hook: Some(_),
                        ..
                    })
                ),
                "Expired entry with a hook must survive future drop so a late \
                 reply can still fire the hook"
            );

            signals().signals.remove(&reply_to);
            tasks().remove(&message_id);
        });
    }
}
