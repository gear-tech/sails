use crate::{
    calls::Remoting,
    collections::HashMap,
    errors::{Result, RtlError},
    events::Listener,
    gtest::{BlockRunResult, Program, System},
    prelude::*,
    rc::Rc,
};
use core::{cell::RefCell, future::Future};
use futures::{
    channel::{
        mpsc::{unbounded, UnboundedSender},
        oneshot,
    },
    FutureExt, Stream, TryFutureExt,
};
use gear_core_errors::{ReplyCode, SuccessReplyReason};

type EventSender = UnboundedSender<(ActorId, Vec<u8>)>;
type ReplySender = oneshot::Sender<Result<Vec<u8>>>;

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

    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    pub fn actor_id(&self) -> Option<ActorId> {
        self.actor_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockRunMode {
    Manual,
    Auto,
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
        let mut event_senders = self.event_senders.borrow_mut();
        let mut reply_senders = self.block_reply_senders.borrow_mut();
        // remove closed event senders
        event_senders.retain(|c| !c.is_closed());
        // iterate over log
        for entry in run_result.log().iter() {
            if entry.destination() == ActorId::zero() {
                for sender in event_senders.iter() {
                    _ = sender.unbounded_send((entry.source(), entry.payload().to_vec()));
                }
                continue;
            }
            if let Some(message_id) = entry.reply_to() {
                if let Some(sender) = reply_senders.remove(&message_id) {
                    let reply: result::Result<Vec<u8>, _> = match entry.reply_code() {
                        None => Err(RtlError::ReplyCodeIsMissing.into()),
                        Some(ReplyCode::Error(reason)) => {
                            Err(RtlError::ReplyHasError(reason).into())
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
        // drain reply senders that not founded in block
        for (_message_id, sender) in reply_senders.drain() {
            _ = sender.send(Err(RtlError::ReplyIsMissing.into()));
        }
    }

    fn send_message(
        &self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<MessageId> {
        let gas_limit = gas_limit.unwrap_or(gtest::constants::GAS_ALLOWANCE);
        let program = self
            .system
            .get_program(target.as_ref())
            .ok_or(RtlError::ProgramIsNotFound)?;
        let actor_id = args.actor_id.unwrap_or(self.actor_id);
        let message_id = program.send_bytes_with_gas(
            actor_id.as_ref(),
            payload.as_ref().to_vec(),
            gas_limit,
            value,
        );
        Ok(message_id)
    }

    fn message_reply_from_next_block(
        &self,
        message_id: MessageId,
    ) -> impl Future<Output = Result<Vec<u8>>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>>>();
        self.block_reply_senders.borrow_mut().insert(message_id, tx);

        if self.block_run_mode == BlockRunMode::Auto {
            _ = self.run_next_block_and_extract();
        }

        rx.unwrap_or_else(|_| Err(RtlError::ReplyIsMissing.into()))
    }

    fn run_next_block_and_extract(&self) -> BlockRunResult {
        let run_result = self.system.run_next_block();
        self.extract_events_and_replies(&run_result);
        run_result
    }
}

impl Remoting for GTestRemoting {
    type Args = GTestArgs;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let gas_limit = gas_limit.unwrap_or(gtest::constants::GAS_ALLOWANCE);
        let code = self
            .system
            .submitted_code(code_id)
            .ok_or(RtlError::ProgramCodeIsNotFound)?;
        let program_id = gtest::calculate_program_id(code_id, salt.as_ref(), None);
        let program = Program::from_binary_with_id(&self.system, program_id, code);
        let actor_id = args.actor_id.unwrap_or(self.actor_id);
        let message_id = program.send_bytes_with_gas(
            actor_id.as_ref(),
            payload.as_ref().to_vec(),
            gas_limit,
            value,
        );
        Ok(self
            .message_reply_from_next_block(message_id)
            .map(move |result| result.map(|reply| (program_id, reply))))
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let message_id = self.send_message(target, payload, gas_limit, value, args)?;
        Ok(self.message_reply_from_next_block(message_id))
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<Vec<u8>> {
        let message_id = self.send_message(target, payload, gas_limit, value, args)?;
        self.message_reply_from_next_block(message_id).await
    }
}

impl Listener<Vec<u8>> for GTestRemoting {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, Vec<u8>)>> {
        let (tx, rx) = unbounded::<(ActorId, Vec<u8>)>();
        self.event_senders.borrow_mut().push(tx);
        Ok(rx)
    }
}
