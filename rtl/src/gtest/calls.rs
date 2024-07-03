use crate::{
    calls::Remoting,
    errors::{Result, RtlError},
    rc::Rc,
    ActorId, CodeId, ValueUnit, Vec,
};
use core::future::Future;
use gear_core_errors::{ReplyCode, SuccessReplyReason};
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
        let program = self
            .system
            .get_program(target.as_ref())
            .ok_or(RtlError::ProgramIsNotFound)?;
        let actor_id = args.actor_id.ok_or(RtlError::ActorIsNotSet)?;
        let run_result =
            program.send_bytes_with_value(actor_id.as_ref(), payload.as_ref().to_vec(), value);
        Ok(async move { Self::extract_reply(run_result) })
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<Vec<u8>> {
        let program = self
            .system
            .get_program(target.as_ref())
            .ok_or(RtlError::ProgramIsNotFound)?;
        let actor_id = args.actor_id.ok_or(RtlError::ActorIsNotSet)?;
        let run_result =
            program.send_bytes_with_value(actor_id.as_ref(), payload.as_ref().to_vec(), value);
        Self::extract_reply(run_result)
    }
}
