use crate::{
    calls::{Action, Remoting},
    collections::HashMap,
    errors::{Result, RtlError},
    events::Listener,
    futures::*,
    gtest::{BlockRunResult, Program, System},
    prelude::*,
    rc::Rc,
};
use core::{cell::RefCell, future::Future};
use gear_core_errors::{ReplyCode, SuccessReplyReason};

type EventSender = channel::mpsc::UnboundedSender<(ActorId, Vec<u8>)>;
type ReplySender = channel::oneshot::Sender<Result<Vec<u8>>>;

const GAS_LIMIT_DEFAULT: gtest::constants::Gas = gtest::constants::MAX_USER_GAS_LIMIT;

#[derive(Debug, Default)]
pub struct GTestArgs {
    actor_id: Option<ActorId>,
}

impl GTestArgs {
    pub fn new(actor_id: ActorId) -> Self {
        Self {
            actor_id: Some(actor_id),
        }
    }

    pub fn with_actor_id(self, actor_id: ActorId) -> Self {
        Self {
            actor_id: Some(actor_id),
        }
    }

    pub fn actor_id(&self) -> Option<ActorId> {
        self.actor_id
    }
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
pub struct GTestRemoting {
    system: Rc<System>,
    actor_id: ActorId,
    event_senders: Rc<RefCell<Vec<EventSender>>>,
    block_run_mode: BlockRunMode,
    block_reply_senders: Rc<RefCell<HashMap<MessageId, ReplySender>>>,
}

impl GTestRemoting {
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

impl GTestRemoting {
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
                log::debug!("Extract event from entry {:?}", entry);
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
                    log::debug!("Extract reply from entry {:?}", entry);
                    let reply: result::Result<Vec<u8>, _> = match entry.reply_code() {
                        None => Err(RtlError::ReplyCodeIsMissing.into()),
                        Some(ReplyCode::Error(reason)) => {
                            Err(RtlError::ReplyHasError(reason, entry.payload().to_vec()).into())
                        }
                        Some(ReplyCode::Success(SuccessReplyReason::Manual)) => {
                            Ok(entry.payload().to_vec())
                        }
                        _ => Err(RtlError::ReplyIsMissing.into()),
                    };
                    _ = sender.send(reply);
                }
            }
        }
    }

    fn send_message(
        &self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<MessageId> {
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = gas_limit.unwrap_or(GAS_LIMIT_DEFAULT);
        #[cfg(feature = "ethexe")]
        let gas_limit = GAS_LIMIT_DEFAULT;
        let program = self
            .system
            .get_program(target)
            .ok_or(RtlError::ProgramIsNotFound)?;
        let actor_id = args.actor_id.unwrap_or(self.actor_id);
        let message_id = program.send_bytes_with_gas(actor_id, payload.as_ref(), gas_limit, value);
        log::debug!("Send message id: {message_id}, to: {target}");
        Ok(message_id)
    }

    fn message_reply_from_next_blocks(
        &self,
        message_id: MessageId,
    ) -> impl Future<Output = Result<Vec<u8>>> + use<> {
        let (tx, rx) = channel::oneshot::channel::<Result<Vec<u8>>>();
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

        rx.unwrap_or_else(|_| Err(RtlError::ReplyIsMissing.into()))
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
            log::debug!("Reply is missing in block for message {}", message_id);
            _ = sender.send(Err(RtlError::ReplyIsMissing.into()));
        }
    }
}

impl Remoting for GTestRemoting {
    type Args = GTestArgs;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = gas_limit.unwrap_or(GAS_LIMIT_DEFAULT);
        #[cfg(feature = "ethexe")]
        let gas_limit = GAS_LIMIT_DEFAULT;
        let code = self
            .system
            .submitted_code(code_id)
            .ok_or(RtlError::ProgramCodeIsNotFound)?;
        let program_id = gtest::calculate_program_id(code_id, salt.as_ref(), None);
        let program = Program::from_binary_with_id(&self.system, program_id, code);
        let actor_id = args.actor_id.unwrap_or(self.actor_id);
        let message_id = program.send_bytes_with_gas(actor_id, payload.as_ref(), gas_limit, value);
        log::debug!("Send activation id: {message_id}, to program: {program_id}");
        Ok(self
            .message_reply_from_next_blocks(message_id)
            .map(move |result| result.map(|reply| (program_id, reply))))
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let message_id = self.send_message(
            target,
            payload,
            #[cfg(not(feature = "ethexe"))]
            gas_limit,
            value,
            args,
        )?;
        Ok(self.message_reply_from_next_blocks(message_id))
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<Vec<u8>> {
        let message_id = self.send_message(
            target,
            payload,
            #[cfg(not(feature = "ethexe"))]
            gas_limit,
            value,
            args,
        )?;
        self.message_reply_from_next_blocks(message_id).await
    }
}

impl Listener<Vec<u8>> for GTestRemoting {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, Vec<u8>)>> {
        let (tx, rx) = channel::mpsc::unbounded::<(ActorId, Vec<u8>)>();
        self.event_senders.borrow_mut().push(tx);
        Ok(rx)
    }
}

pub trait WithArgs {
    fn with_actor_id(self, actor_id: ActorId) -> Self;
}

impl<T> WithArgs for T
where
    T: Action<Args = GTestArgs>,
{
    fn with_actor_id(self, actor_id: ActorId) -> Self {
        self.with_args(|args| args.with_actor_id(actor_id))
    }
}
