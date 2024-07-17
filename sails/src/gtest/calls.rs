use crate::{
    calls::Remoting,
    errors::{Result, RtlError},
    events::Listener,
    prelude::*,
    rc::Rc,
};
use core::{cell::RefCell, future::Future};
use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    Stream,
};
use gear_core_errors::{ReplyCode, SuccessReplyReason};
use gtest::{Program, RunResult, System};

type EventSender = UnboundedSender<(ActorId, Vec<u8>)>;

#[derive(Debug, Default, Clone)]
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

#[derive(Clone)]
pub struct GTestRemoting {
    system: Rc<System>,
    event_senders: Rc<RefCell<Vec<EventSender>>>,
}

impl Default for GTestRemoting {
    fn default() -> Self {
        Self::new()
    }
}

impl GTestRemoting {
    pub fn new() -> Self {
        Self {
            system: Rc::new(System::new()),
            event_senders: Default::default(),
        }
    }

    pub fn system(&self) -> &System {
        &self.system
    }
}

impl GTestRemoting {
    fn extract_reply(run_result: &RunResult) -> Result<Vec<u8>> {
        let mut reply_iter = run_result
            .log()
            .iter()
            .filter(|entry| entry.reply_to() == Some(run_result.sent_message_id()));
        let reply = reply_iter.next().ok_or(RtlError::ReplyIsMissing)?;
        if reply_iter.next().is_some() {
            Err(RtlError::ReplyIsAmbiguous)?
        }
        let reply_code = reply.reply_code().ok_or(RtlError::ReplyCodeIsMissing)?;
        if let ReplyCode::Error(reason) = reply_code {
            Err(RtlError::ReplyHasError(reason))?
        }
        if reply_code != ReplyCode::Success(SuccessReplyReason::Manual) {
            Err(RtlError::ReplyIsMissing)?
        }
        Ok(reply.payload().to_vec())
    }

    fn extract_and_send_events(run_result: &RunResult, senders: &mut Vec<EventSender>) {
        let events: Vec<(ActorId, Vec<u8>)> = run_result
            .log()
            .iter()
            .filter(|entry| entry.destination() == ActorId::zero())
            .map(|entry| (entry.source(), entry.payload().to_vec()))
            .collect();
        senders.retain(|c| !c.is_closed());
        for sender in senders.iter() {
            events.clone().into_iter().for_each(move |ev| {
                _ = sender.unbounded_send(ev);
            });
        }
    }

    fn send_and_get_result(
        &self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<RunResult> {
        let program = self
            .system
            .get_program(target.as_ref())
            .ok_or(RtlError::ProgramIsNotFound)?;
        let actor_id = args.actor_id.ok_or(RtlError::ActorIsNotSet)?;
        Ok(program.send_bytes_with_value(actor_id.as_ref(), payload.as_ref().to_vec(), value))
    }
}

impl Remoting for GTestRemoting {
    type Args = GTestArgs;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let code = self
            .system
            .submitted_code(code_id)
            .ok_or(RtlError::ProgramCodeIsNotFound)?;
        let program_id = gtest::calculate_program_id(code_id, salt.as_ref(), None);
        let program = Program::from_binary_with_id(&self.system, program_id, code);
        let actor_id = args.actor_id.ok_or(RtlError::ActorIsNotSet)?;
        let run_result =
            program.send_bytes_with_value(actor_id.as_ref(), payload.as_ref().to_vec(), value);
        Ok(async move {
            let reply = Self::extract_reply(&run_result)?;
            Ok((program_id, reply))
        })
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let run_result = self.send_and_get_result(target, payload, value, args)?;
        Self::extract_and_send_events(&run_result, self.event_senders.borrow_mut().as_mut());
        Ok(async move { Self::extract_reply(&run_result) })
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<Vec<u8>> {
        let run_result = self.send_and_get_result(target, payload, value, args)?;
        Self::extract_reply(&run_result)
    }
}

impl Listener<Vec<u8>> for GTestRemoting {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, Vec<u8>)>> {
        let (tx, rx) = unbounded::<(ActorId, Vec<u8>)>();
        self.event_senders.borrow_mut().push(tx);
        Ok(rx)
    }
}
