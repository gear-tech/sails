use super::*;
use ::gtest::{BlockRunResult, System, TestError};
use core::cell::RefCell;
use futures::channel::{mpsc, oneshot};
use hashbrown::HashMap;
use std::rc::Rc;

const GAS_LIMIT_DEFAULT: ::gtest::constants::Gas = ::gtest::constants::MAX_USER_GAS_LIMIT;
type Error = TestError;
type EventSender = mpsc::UnboundedSender<(ActorId, Vec<u8>)>;
type ReplySender = oneshot::Sender<Result<Vec<u8>, Error>>;
type ReplyReceiver = oneshot::Receiver<Result<Vec<u8>, Error>>;

#[derive(Debug, Default)]
pub struct GtestParams {
    actor_id: Option<ActorId>,
    #[cfg(not(feature = "ethexe"))]
    gas_limit: Option<GasUnit>,
    value: ValueUnit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockRunMode {
    /// Run blocks automatically until all pending replies are received.
    Auto,
    /// Run only next block and exract events and replies from it.
    /// If there is no reply in this block then `RtlError::ReplyIsMissing` error will be returned.
    Next,
    /// Sending messages does not cause blocks to run.
    /// Use `GTestRemoting::run_next_block` method to run the next block and extract events and replies.
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

impl GtestEnv {
    /// Create new `GTestRemoting` instance from `gtest::System` with specified `actor_id`
    /// and `Auto` block run mode
    pub fn new(system: System, actor_id: ActorId) -> Self {
        let system = Rc::new(system);
        Self {
            system,
            actor_id,
            event_senders: Default::default(),
            block_run_mode: BlockRunMode::Auto,
            block_reply_senders: Default::default(),
        }
    }

    // Avoid calling methods of `System` related to block execution.
    // Use `GTestRemoting::run_next_block` instead. This method can be used
    // for obtaining reference data like balance, timestamp, etc.
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
        // remove closed event senders
        event_senders.retain(|c| !c.is_closed());
        // iterate over log
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
            if let Some(message_id) = entry.reply_to() {
                if let Some(sender) = reply_senders.remove(&message_id) {
                    log::debug!("Extract reply from entry {entry:?}");
                    let reply: result::Result<Vec<u8>, _> = match entry.reply_code() {
                        None => Err(Error::InvalidReturnType),
                        // TODO handle error reply
                        Some(ReplyCode::Error(reason)) => {
                            panic!("Unexpected error reply: {reason:?}")
                        }
                        Some(ReplyCode::Success(_)) => Ok(entry.payload().to_vec()),
                        _ => Err(Error::InvalidReturnType),
                    };
                    _ = sender.send(reply);
                }
            }
        }
    }

    fn activate(
        &self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        params: GtestParams,
    ) -> Result<(ActorId, MessageId), Error> {
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = params.gas_limit.unwrap_or(GAS_LIMIT_DEFAULT);
        #[cfg(feature = "ethexe")]
        let gas_limit = GAS_LIMIT_DEFAULT;
        let code = self
            .system
            .submitted_code(code_id)
            // TODO Errors
            .ok_or(Error::Instrumentation)?;
        let program_id = ::gtest::calculate_program_id(code_id, salt.as_ref(), None);
        let program = ::gtest::Program::from_binary_with_id(&self.system, program_id, code);
        let actor_id = params.actor_id.unwrap_or(self.actor_id);
        let message_id =
            program.send_bytes_with_gas(actor_id, payload.as_ref(), gas_limit, params.value);
        log::debug!("Send activation id: {message_id}, to program: {program_id}");
        Ok((program_id, message_id))
    }

    fn send_message(
        &self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        params: GtestParams,
    ) -> Result<MessageId, Error> {
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = params.gas_limit.unwrap_or(GAS_LIMIT_DEFAULT);
        #[cfg(feature = "ethexe")]
        let gas_limit = GAS_LIMIT_DEFAULT;
        let program = self
            .system
            .get_program(target)
            // TODO Errors
            .ok_or(Error::Instrumentation)?;
        let actor_id = params.actor_id.unwrap_or(self.actor_id);
        let message_id =
            program.send_bytes_with_gas(actor_id, payload.as_ref(), gas_limit, params.value);
        log::debug!(
            "Send message id: {message_id}, to: {target}, payload: {}",
            hex::encode(payload.as_ref())
        );
        Ok(message_id)
    }

    fn message_reply_from_next_blocks(&self, message_id: MessageId) -> ReplyReceiver {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, Error>>();
        self.block_reply_senders.borrow_mut().insert(message_id, tx);

        match self.block_run_mode {
            BlockRunMode::Auto => {
                self.run_until_extract_replies();
            }
            BlockRunMode::Next => {
                self.run_next_block_and_extract();
                self.drain_reply_senders();
            }
            BlockRunMode::Manual => (),
        };
        rx
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

    fn drain_reply_senders(&self) {
        let mut reply_senders = self.block_reply_senders.borrow_mut();
        // drain reply senders that not founded in block
        for (message_id, sender) in reply_senders.drain() {
            log::debug!("Reply is missing in block for message {message_id}");
            // TODO handle error
            _ = sender.send(Err(Error::UnsupportedFunction(
                "Reply is missing in block for message {message_id}".into(),
            )));
        }
    }
}

impl GearEnv for GtestEnv {
    type Params = GtestParams;
    type Error = Error;
    type MessageState = ReplyReceiver;
}

impl<O: Decode> Future for PendingCall<GtestEnv, O> {
    type Output = Result<O, <GtestEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            // Send message
            let params = self.params.take().unwrap_or_default();
            let payload = self.payload.take().unwrap_or_default();
            let send_res = self.env.send_message(self.destination, payload, params);
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
        if let Some(reply_receiver) = self.project().state.as_pin_mut() {
            // Poll reply receiver
            match reply_receiver.poll(cx) {
                Poll::Ready(Ok(res)) => match res {
                    // TODO handle reply prefix
                    Ok(bytes) => match <(String, String, O)>::decode(&mut bytes.as_slice()) {
                        Ok((_, _, decoded)) => Poll::Ready(Ok(decoded)),
                        Err(err) => Poll::Ready(Err(Error::ScaleCodecError(err))),
                    },
                    Err(err) => Poll::Ready(Err(err)),
                },
                // TODO handle error
                Poll::Ready(Err(_err)) => Poll::Ready(Err(Error::InvalidReturnType)),
                Poll::Pending => Poll::Pending,
            }
        } else {
            panic!("PendingCall polled after completion or invalid state");
        }
    }
}

impl<A, T: ActionIo> Future for PendingCtor<GtestEnv, A, T> {
    type Output = Result<Actor<A, GtestEnv>, <GtestEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            // Send message
            let params = self.params.take().unwrap_or_default();
            let salt = self.salt.take().unwrap_or_default();
            let args = self
                .args
                .take()
                .unwrap_or_else(|| panic!("PendingCtor polled after completion or invalid state"));
            let payload = T::encode_call(&args);
            let send_res = self
                .env
                .activate(self.code_id, salt, payload.as_slice(), params);
            match send_res {
                Ok((program_id, message_id)) => {
                    log::debug!("PendingCall: send message {message_id:?}");
                    self.state = Some(self.env.message_reply_from_next_blocks(message_id));
                    self.program_id = Some(program_id);
                }
                Err(err) => {
                    log::error!("PendingCall: failed to send message: {err}");
                    return Poll::Ready(Err(err));
                }
            }
        }
        let this = self.project();
        if let Some(reply_receiver) = this.state.as_pin_mut() {
            // Poll reply receiver
            match reply_receiver.poll(cx) {
                Poll::Ready(Ok(res)) => match res {
                    // TODO handle reply prefix
                    Ok(_) => {
                        let program_id = this.program_id.unwrap();
                        let env = this.env.clone();
                        Poll::Ready(Ok(Actor::new(program_id, env)))
                    }
                    Err(err) => Poll::Ready(Err(err)),
                },
                // TODO handle error
                Poll::Ready(Err(_err)) => Poll::Ready(Err(Error::InvalidReturnType)),
                Poll::Pending => Poll::Pending,
            }
        } else {
            panic!("PendingCtor polled after completion or invalid state");
        }
    }
}

impl<A, T: ActionIo> PendingCtor<GtestEnv, A, T> {
    pub fn with_actor_id(self, actor_id: ActorId) -> Self {
        self.with_params(|mut params| {
            params.actor_id = Some(actor_id);
            params
        })
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn with_gas_limit(self, gas_limit: GasUnit) -> Self {
        self.with_params(|mut params| {
            params.gas_limit = Some(gas_limit);
            params
        })
    }
    pub fn with_value(self, value: ValueUnit) -> Self {
        self.with_params(|mut params| {
            params.value = value;
            params
        })
    }
}

impl<O: Decode> PendingCall<GtestEnv, O> {
    pub fn with_actor_id(self, actor_id: ActorId) -> Self {
        self.with_params(|mut params| {
            params.actor_id = Some(actor_id);
            params
        })
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn with_gas_limit(self, gas_limit: GasUnit) -> Self {
        self.with_params(|mut params| {
            params.gas_limit = Some(gas_limit);
            params
        })
    }
    pub fn with_value(self, value: ValueUnit) -> Self {
        self.with_params(|mut params| {
            params.value = value;
            params
        })
    }
}
