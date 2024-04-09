use crate::calls::Remoting as RemotingTrait;
use crate::{errors::Result, prelude::*};
use core::future::Future;
use futures::FutureExt;
use gstd::{exec, msg, prog};

#[derive(Debug, Default, Clone)]
pub struct Args {
    reply_deposit: GasUnit,
}

impl Args {
    pub fn with_reply_deposit(mut self, reply_deposit: GasUnit) -> Self {
        self.reply_deposit = reply_deposit;
        self
    }

    pub fn reply_deposit(&self) -> GasUnit {
        self.reply_deposit
    }
}

#[derive(Debug, Default, Clone)]
pub struct Remoting;

impl RemotingTrait<Args> for Remoting {
    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: Args,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let reply_future = prog::create_program_bytes_for_reply(
            code_id.into(),
            salt,
            payload,
            value,
            args.reply_deposit,
        )?;
        let reply_future = reply_future.map(|result| {
            result
                .map(|(actor_id, data)| (actor_id.into(), data))
                .map_err(Into::into)
        });
        Ok(reply_future)
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: Args,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let reply_future =
            msg::send_bytes_for_reply(target.into(), payload, value, args.reply_deposit)?;
        if args.reply_deposit > 0 {
            exec::reply_deposit(reply_future.waiting_reply_to, args.reply_deposit)?;
        }
        let reply_future = reply_future.map(|result| result.map_err(Into::into));
        Ok(reply_future)
    }
}
