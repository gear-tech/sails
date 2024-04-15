use crate::{
    calls::Remoting,
    errors::{Result, RtlError},
    prelude::*,
    rc::Rc,
};
use core::future::Future;
use gear_core_errors::{ReplyCode, SuccessReplyReason};
use gtest::System;

#[derive(Debug, Clone)]
pub struct GTestArgs {
    actor_id: ActorId,
}

impl GTestArgs {
    pub fn new(actor_id: ActorId) -> Self {
        Self { actor_id }
    }

    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.actor_id = actor_id;
        self
    }

    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }
}

#[derive(Default, Clone)]
pub struct GTestRemoting {
    system: Rc<System>,
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

impl Remoting<GTestArgs> for GTestRemoting {
    async fn activate(
        self,
        _code_id: CodeId,
        _salt: impl AsRef<[u8]>,
        _payload: impl AsRef<[u8]>,
        _value: ValueUnit,
        _args: GTestArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        todo!();
        #[allow(unreachable_code)]
        Ok(async { Ok((ActorId::from([0; 32]), vec![])) })
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GTestArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let program = self.system.get_program(*target.as_ref());
        let run_result = program.send_bytes_with_value(*args.actor_id.as_ref(), payload, value);
        Ok(async move {
            let mut reply_iter = run_result
                .log()
                .iter()
                .filter(|entry| entry.reply_to() == Some(run_result.sent_message_id()));
            let reply = reply_iter.next().ok_or(RtlError::ReplyIsMissing)?;
            if reply_iter.next().is_some() {
                Err(RtlError::ReplyIsAmbiguous)?
            }
            let reply_code = reply.reply_code().ok_or(RtlError::ReplyCodeIsMissing)?;
            if let ReplyCode::Error(error) = reply_code {
                Err(error)?
            }
            if reply_code != ReplyCode::Success(SuccessReplyReason::Manual) {
                Err(RtlError::ReplyIsMissing)?
            }
            Ok(reply.payload().to_vec())
        })
    }
}
