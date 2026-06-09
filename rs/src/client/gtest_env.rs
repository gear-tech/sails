use super::*;
pub use ::gtest::constants::{
    DEFAULT_USER_ALICE, DEFAULT_USER_BOB, DEFAULT_USER_CHARLIE, DEFAULT_USER_EVE,
    DEFAULT_USERS_INITIAL_BALANCE, EPOCH_DURATION_IN_BLOCKS, MAX_USER_GAS_LIMIT,
};
#[cfg(feature = "ethexe")]
use ::gtest::ethexe::{Program, System};
use ::gtest::{BlockRunResult, TestError};
#[cfg(not(feature = "ethexe"))]
use ::gtest::{Program, System};
use core::{cell::RefCell, task::ready};
use futures::{
    Stream,
    channel::{mpsc, oneshot},
};
pub use gear_core_errors::{ErrorReplyReason, SimpleExecutionError};
use hashbrown::HashMap;
use std::rc::Rc;
use tokio_stream::StreamExt;

const GAS_LIMIT_DEFAULT: ::gtest::constants::Gas = ::gtest::constants::MAX_USER_GAS_LIMIT;
/// Executable balance credited to programs created through this env under `ethexe`,
/// where execution gas is paid from the program's executable balance rather than an
/// explicit per-message gas limit. Generous default so tests don't run out of gas.
#[cfg(feature = "ethexe")]
const ETHEXE_EXECUTABLE_BALANCE: ValueUnit = 100_000_000_000_000;
type EventSender = mpsc::UnboundedSender<(ActorId, Vec<u8>)>;
type ReplySender = oneshot::Sender<Result<Vec<u8>, GtestError>>;
type ReplyReceiver = oneshot::Receiver<Result<Vec<u8>, GtestError>>;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GtestError {
    #[error(transparent)]
    Env(#[from] TestError),
    #[error("reply error: {0}")]
    ReplyHasError(ErrorReplyReason, crate::Vec<u8>),
    #[error("reply is missing")]
    ReplyIsMissing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockRunMode {
    /// Run blocks until all pending replies are received.
    Auto,
    /// Run up to the given number of blocks. Any pending reply not produced
    /// within that limit resolves with `GtestError::ReplyIsMissing`.
    UpTo(BlockCount),
    /// Run only the next block. Any reply not produced in that block
    /// resolves with `GtestError::ReplyIsMissing`.
    Next,
    /// Sending messages does not advance the chain.
    /// Use `GtestEnv::run_next_block` to advance manually.
    Manual,
}

#[derive(Clone)]
pub struct GtestEnv {
    system: Rc<System>,
    actor_id: ActorId,
    event_senders: Rc<RefCell<Vec<EventSender>>>,
    block_run_mode: BlockRunMode,
    block_reply_senders: Rc<RefCell<HashMap<MessageId, ReplySender>>>,
}

crate::params_struct_impl!(
    GtestEnv,
    GtestParams {
        actor_id: ActorId,
        #[cfg(not(feature = "ethexe"))]
        gas_limit: GasUnit,
        value: ValueUnit,
    }
);

impl GtestEnv {
    /// Create new `GtestEnv` instance from `gtest::System` with specified `actor_id`
    /// and `UpTo(EPOCH_DURATION_IN_BLOCKS)` block run mode.
    pub fn new(system: System, actor_id: ActorId) -> Self {
        let system = Rc::new(system);
        Self {
            system,
            actor_id,
            event_senders: Default::default(),
            block_run_mode: BlockRunMode::UpTo(EPOCH_DURATION_IN_BLOCKS),
            block_reply_senders: Default::default(),
        }
    }

    /// Create `GtestEnv` instance with new `System` and `DEFAULT_USER_ALICE` actor
    pub fn system_default() -> Self {
        let system = System::new();
        system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
        system.mint_to(DEFAULT_USER_ALICE, DEFAULT_USERS_INITIAL_BALANCE);

        GtestEnv::new(system, DEFAULT_USER_ALICE.into())
    }

    /// Avoid calling methods of `System` related to block execution.
    /// Use `GtestEnv::run_next_block` instead. This method can be used
    /// for obtaining reference data like balance, timestamp, etc.
    pub fn system(&self) -> &System {
        &self.system
    }

    pub fn with_block_run_mode(self, block_run_mode: BlockRunMode) -> Self {
        Self {
            block_run_mode,
            ..self
        }
    }

    pub fn with_actor_id(self, actor_id: ActorId) -> Self {
        Self { actor_id, ..self }
    }

    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    pub fn run_next_block(&self) {
        _ = self.run_next_block_and_extract();
    }
}

impl GtestEnv {
    fn extract_events_and_replies(&self, run_result: &BlockRunResult) {
        log::debug!(
            "Process block #{} run result, mode {:?}",
            run_result.block_info.height,
            &self.block_run_mode
        );
        let mut event_senders = self.event_senders.borrow_mut();
        let mut reply_senders = self.block_reply_senders.borrow_mut();
        event_senders.retain(|c| !c.is_closed());
        for entry in run_result.log().iter() {
            if entry.destination() == ActorId::zero() {
                log::debug!("Extract event from entry {entry:?}");
                for sender in event_senders.iter() {
                    _ = sender.unbounded_send((entry.source(), entry.payload().to_vec()));
                }
                continue;
            }
            #[cfg(feature = "ethexe")]
            if entry.destination() == crate::solidity::ETH_EVENT_ADDR {
                log::debug!("Extract event from entry {:?}", entry);
                for sender in event_senders.iter() {
                    _ = sender.unbounded_send((entry.source(), entry.payload().to_vec()));
                }
                continue;
            }
            if let Some(message_id) = entry.reply_to()
                && let Some(sender) = reply_senders.remove(&message_id)
            {
                log::debug!("Extract reply from entry {entry:?}");
                let reply: result::Result<Vec<u8>, _> = match entry.reply_code() {
                    Some(ReplyCode::Success(_)) => Ok(entry.payload().to_vec()),
                    Some(ReplyCode::Error(reason)) => {
                        Err(GtestError::ReplyHasError(reason, entry.payload().to_vec()))
                    }
                    _ => Err(GtestError::ReplyIsMissing),
                };
                _ = sender.send(reply);
            }
        }
    }

    pub fn create_program(
        &self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        params: GtestParams,
    ) -> Result<(ActorId, MessageId), GtestError> {
        let value = params.value.unwrap_or(0);
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = params.gas_limit.unwrap_or(GAS_LIMIT_DEFAULT);
        #[cfg(feature = "ethexe")]
        let gas_limit = GAS_LIMIT_DEFAULT;
        let code = self
            .system
            .submitted_code(code_id)
            .ok_or(TestError::Instrumentation)?;
        let program_id = ::gtest::calculate_program_id(code_id, salt.as_ref(), None);
        let program = Program::from_binary_with_id(&self.system, program_id, code);
        // On `ethexe`, execution gas is drawn from the program's executable balance.
        #[cfg(feature = "ethexe")]
        self.system
            .top_up_executable_balance(program_id, ETHEXE_EXECUTABLE_BALANCE);
        let actor_id = params.actor_id.unwrap_or(self.actor_id);
        let message_id = program.send_bytes_with_gas(actor_id, payload.as_ref(), gas_limit, value);
        log::debug!("Send activation id: {message_id}, to program: {program_id}");
        Ok((program_id, message_id))
    }

    pub fn send_one_way(
        &self,
        destination: ActorId,
        payload: impl AsRef<[u8]>,
        params: GtestParams,
    ) -> Result<MessageId, GtestError> {
        let value = params.value.unwrap_or(0);
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = params.gas_limit.unwrap_or(GAS_LIMIT_DEFAULT);
        #[cfg(feature = "ethexe")]
        let gas_limit = GAS_LIMIT_DEFAULT;
        let program = self
            .system
            .get_program(destination)
            .ok_or(TestError::ActorNotFound(destination))?;
        let actor_id = params.actor_id.unwrap_or(self.actor_id);
        let message_id = program.send_bytes_with_gas(actor_id, payload.as_ref(), gas_limit, value);
        log::debug!(
            "Send message id: {message_id}, to: {destination}, payload: {}",
            hex::encode(payload.as_ref())
        );
        Ok(message_id)
    }

    pub async fn send_for_reply(
        &self,
        destination: ActorId,
        payload: impl AsRef<[u8]>,
        params: GtestParams,
    ) -> Result<Vec<u8>, GtestError> {
        let message_id = self.send_one_way(destination, payload, params)?;
        self.message_reply_from_next_blocks(message_id)
            .await
            .unwrap_or(Err(GtestError::ReplyIsMissing))
    }

    pub fn message_reply_from_next_blocks(&self, message_id: MessageId) -> ReplyReceiver {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, GtestError>>();
        self.block_reply_senders.borrow_mut().insert(message_id, tx);

        match self.block_run_mode {
            BlockRunMode::Auto => {
                self.run_until_extract_replies();
            }
            BlockRunMode::UpTo(block_limit) => {
                self.run_until_extract_replies_up_to(block_limit);
            }
            BlockRunMode::Next => {
                self.run_next_block_and_extract();
                self.drain_reply_senders();
            }
            BlockRunMode::Manual => (),
        };
        rx
    }

    /// Read-only state query. Not available under `ethexe`: ethexe gtest mode
    /// does not support `calculate_reply_for_handle`. As with `GstdEnv`, call the
    /// query method as a message instead (`.await` / `send_for_reply`).
    #[cfg(not(feature = "ethexe"))]
    pub fn query(
        &self,
        destination: ActorId,
        payload: impl AsRef<[u8]>,
        params: GtestParams,
    ) -> Result<Vec<u8>, GtestError> {
        let value = params.value.unwrap_or(0);
        let gas_limit = params.gas_limit.unwrap_or(GAS_LIMIT_DEFAULT);

        let actor_id = params.actor_id.unwrap_or(self.actor_id);
        let reply_info = self
            .system
            .calculate_reply_for_handle(actor_id, destination, payload.as_ref(), gas_limit, value)
            .map_err(|_s| GtestError::ReplyIsMissing)?;

        match reply_info.code {
            ReplyCode::Success(_) => Ok(reply_info.payload),
            ReplyCode::Error(err) => Err(GtestError::ReplyHasError(err, reply_info.payload)),
            _ => {
                log::debug!("Unexpected reply code: {:?}", reply_info.code);
                Err(GtestError::ReplyIsMissing)
            }
        }
    }

    fn run_next_block_and_extract(&self) -> BlockRunResult {
        let run_result = self.system.run_next_block();
        self.extract_events_and_replies(&run_result);
        run_result
    }

    fn run_until_extract_replies(&self) {
        while !self.block_reply_senders.borrow().is_empty() {
            self.run_next_block_and_extract();
        }
    }

    fn run_until_extract_replies_up_to(&self, block_limit: BlockCount) {
        for _ in 0..block_limit {
            if self.block_reply_senders.borrow().is_empty() {
                return;
            }
            self.run_next_block_and_extract();
        }
        self.drain_reply_senders();
    }

    fn drain_reply_senders(&self) {
        let mut reply_senders = self.block_reply_senders.borrow_mut();
        for (message_id, sender) in reply_senders.drain() {
            log::debug!("Reply is missing in block for message {message_id}");
            _ = sender.send(Err(GtestError::ReplyIsMissing));
        }
    }
}

impl ReplyError for GtestError {
    fn from_codec_error(err: parity_scale_codec::Error) -> Self {
        TestError::ScaleCodecError(err).into()
    }

    fn userspace_panic_payload(&self) -> Option<&[u8]> {
        match self {
            GtestError::ReplyHasError(
                ErrorReplyReason::Execution(SimpleExecutionError::UserspacePanic),
                payload,
            ) => Some(payload),
            _ => None,
        }
    }
}

impl GearEnv for GtestEnv {
    type Params = GtestParams;
    type Error = GtestError;
    type MessageState = ReplyReceiver;
}

impl EnvWithCtor for GtestEnv {}

impl<T: ServiceCall> PendingCall<T, GtestEnv> {
    pub fn send_one_way(&mut self) -> Result<MessageId, GtestError> {
        if self.state.is_some() {
            panic!("{PENDING_CALL_INVALID_STATE}");
        }
        let (payload, params) = self.take_encoded_args_and_params();
        let message_id = self.env.send_one_way(self.destination, payload, params)?;
        log::debug!("PendingCall: send message {message_id:?}");
        Ok(message_id)
    }

    pub fn send_for_reply(mut self) -> Result<Self, GtestError> {
        let message_id = self.send_one_way()?;
        self.state = Some(self.env.message_reply_from_next_blocks(message_id));
        Ok(self)
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn query(mut self) -> Result<T::Output, GtestError> {
        let (payload, params) = self.take_encoded_args_and_params();
        let reply = self.env.query(self.destination, payload, params);
        decode_reply_or_throw::<T, _>(&self.route, reply)
    }
}

impl<T: ServiceCall> Future for PendingCall<T, GtestEnv> {
    type Output = Result<T::Output, <GtestEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            let (payload, params) = self.take_encoded_args_and_params();
            let send_res = self.env.send_one_way(self.destination, payload, params);
            match send_res {
                Ok(message_id) => {
                    log::debug!("PendingCall: send message {message_id:?}");
                    self.state = Some(self.env.message_reply_from_next_blocks(message_id));
                }
                Err(err) => {
                    log::error!("PendingCall: failed to send message: {err}");
                    return Poll::Ready(Err(err));
                }
            }
        }
        let this = self.as_mut().project();
        let reply_receiver = this
            .state
            .as_pin_mut()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        match ready!(reply_receiver.poll(cx)) {
            Ok(res) => Poll::Ready(decode_reply_or_throw::<T, _>(this.route, res)),
            Err(_) => Poll::Ready(Err(GtestError::ReplyIsMissing)),
        }
    }
}

impl<A, T: ServiceCall> PendingCtor<A, T, GtestEnv> {
    pub fn create_program(mut self) -> Result<Self, GtestError> {
        if self.state.is_some() {
            panic!("{PENDING_CTOR_INVALID_STATE}");
        }
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
        let payload = T::encode_call(&self.route, &args);
        let params = self.params.take().unwrap_or_default();
        let salt = self.salt.take().unwrap_or_default();
        let send_res = self
            .env
            .create_program(self.code_id, salt, payload.as_slice(), params);
        match send_res {
            Ok((program_id, message_id)) => {
                log::debug!("PendingCtor: send message {message_id:?}");
                self.state = Some(self.env.message_reply_from_next_blocks(message_id));
                self.program_id = Some(program_id);
                Ok(self)
            }
            Err(err) => {
                log::error!("PendingCtor: failed to send message: {err}");
                Err(err)
            }
        }
    }
}

impl<A, T> Future for PendingCtor<A, T, GtestEnv>
where
    T: ServiceCall,
    T::Output: PendingCtorOutput<A, GtestEnv>,
{
    type Output =
        Result<<T::Output as PendingCtorOutput<A, GtestEnv>>::Output, <GtestEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            let args = self
                .args
                .take()
                .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
            let payload = T::encode_call(&self.route, &args);
            let params = self.params.take().unwrap_or_default();
            let salt = self.salt.take().unwrap_or_default();
            let send_res = self
                .env
                .create_program(self.code_id, salt, payload.as_slice(), params);
            match send_res {
                Ok((program_id, message_id)) => {
                    log::debug!("PendingCtor: send message {message_id:?}");
                    self.state = Some(self.env.message_reply_from_next_blocks(message_id));
                    self.program_id = Some(program_id);
                }
                Err(err) => {
                    log::error!("PendingCtor: failed to send message: {err}");
                    return Poll::Ready(Err(err));
                }
            }
        }
        let route = self.route.clone();
        let this = self.as_mut().project();
        let reply_receiver = this
            .state
            .as_pin_mut()
            .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
        match ready!(reply_receiver.poll(cx)) {
            Ok(res) => match decode_reply_or_throw::<T, _>(&route, res) {
                Ok(output) => {
                    let program_id = this
                        .program_id
                        .take()
                        .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
                    Poll::Ready(Ok(output.map_result(this.env.clone(), program_id)))
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            Err(_) => Poll::Ready(Err(GtestError::ReplyIsMissing)),
        }
    }
}

impl Listener for GtestEnv {
    type Error = <GtestEnv as GearEnv>::Error;

    async fn listen<E, F: FnMut((ActorId, Vec<u8>)) -> Option<(ActorId, E)>>(
        &self,
        f: F,
    ) -> Result<impl Stream<Item = (ActorId, E)> + Unpin + use<E, F>, Self::Error> {
        let (tx, rx) = mpsc::unbounded::<(ActorId, Vec<u8>)>();
        self.event_senders.borrow_mut().push(tx);
        Ok(rx.filter_map(f))
    }
}

impl<T> Actor<T, GtestEnv> {
    pub fn balance(&self) -> ValueUnit {
        self.env.system().balance_of(self.id)
    }
}
