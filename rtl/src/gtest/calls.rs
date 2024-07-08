use crate::{
    calls::Remoting,
    errors::{Result, RtlError},
    events::EventListener,
    prelude::*,
    rc::Rc,
};
use core::{cell::RefCell, future::Future, ops::Deref, pin::Pin, task::Poll};
use futures::Stream;
use gear_core_errors::{ReplyCode, SuccessReplyReason};
use gstd::rc::Weak;
use gtest::{Program, RunResult, System};

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
    listeners: Rc<RefCell<Vec<Weak<GTestEventListener>>>>,
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
            listeners: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn system(&self) -> &System {
        &self.system
    }
}

impl GTestRemoting {
    fn extract_reply(run_result: RunResult) -> Result<Vec<u8>> {
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

    fn extract_events(self, run_result: &RunResult) {
        let events: Vec<(ActorId, MessageId, Vec<u8>)> = run_result
            .log()
            .iter()
            .filter(|entry| entry.destination() == ActorId::zero())
            .map(|entry| (entry.source(), entry.id(), entry.payload().to_vec()))
            .collect();
        for listener in self.listeners.borrow_mut().iter_mut() {
            if let Some(listener) = listener.upgrade() {
                listener.events.borrow_mut().append(&mut events.clone());
            }
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

impl Remoting<GTestArgs> for GTestRemoting {
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
            let reply = Self::extract_reply(run_result)?;
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
        Self::extract_events(self, &run_result);
        Ok(async move { Self::extract_reply(run_result) })
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<Vec<u8>> {
        let run_result = self.send_and_get_result(target, payload, value, args)?;
        Self::extract_reply(run_result)
    }
}

impl EventListener<Vec<u8>> for GTestRemoting {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, Vec<u8>)>> {
        let listener = Rc::new(GTestEventListener::default());
        self.listeners.borrow_mut().push(Rc::downgrade(&listener));
        Ok(GTestEventStream(Pin::new(listener)))
    }
}

#[derive(Default, Debug)]
pub struct GTestEventListener {
    events: RefCell<Vec<(ActorId, MessageId, Vec<u8>)>>,
}

impl GTestEventListener {
    pub fn events(&self) -> impl Deref<Target = Vec<(ActorId, MessageId, Vec<u8>)>> + '_ {
        self.events.borrow()
    }
}

pub struct GTestEventStream(Pin<Rc<GTestEventListener>>);

impl Stream for GTestEventStream {
    type Item = (ActorId, Vec<u8>);

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.0.events.borrow().is_empty() {
            Poll::Ready(None)
        } else {
            let (source, _, payload) = self.0.events.borrow_mut().remove(0);
            Poll::Ready(Some((source, payload)))
        }
    }
}
