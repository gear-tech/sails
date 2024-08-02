use crate::{calls::Remoting, errors::Result, prelude::*};
use core::future::Future;
use futures::FutureExt;
use gstd::{msg, prog, util};

#[derive(Debug, Default, Clone)]
pub struct GStdArgs {
    reply_deposit: Option<GasUnit>,
}

impl GStdArgs {
    pub fn with_reply_deposit(mut self, reply_deposit: Option<GasUnit>) -> Self {
        self.reply_deposit = reply_deposit;
        self
    }

    pub fn reply_deposit(&self) -> Option<GasUnit> {
        self.reply_deposit
    }
}

#[derive(Debug, Default, Clone)]
pub struct GStdRemoting;

impl GStdRemoting {
    fn send_for_reply(
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: u128,
        args: GStdArgs,
    ) -> Result<msg::MessageFuture, crate::errors::Error> {
        let message_future = if let Some(gas_limit) = gas_limit {
            msg::send_bytes_with_gas_for_reply(
                target,
                payload,
                gas_limit,
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        } else {
            msg::send_bytes_for_reply(
                target,
                payload,
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        };
        Ok(message_future)
    }
}

impl Remoting for GStdRemoting {
    type Args = GStdArgs;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        _gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GStdArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let reply_future = prog::create_program_bytes_for_reply(
            code_id,
            salt,
            payload,
            value,
            args.reply_deposit.unwrap_or_default(),
        )?;
        let reply_future = reply_future.map(|result| result.map_err(Into::into));
        Ok(reply_future)
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GStdArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let reply_future = GStdRemoting::send_for_reply(target, payload, gas_limit, value, args)?;
        let reply_future = reply_future.map(|result| result.map_err(Into::into));
        Ok(reply_future)
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GStdArgs,
    ) -> Result<Vec<u8>> {
        let reply_future = GStdRemoting::send_for_reply(target, payload, gas_limit, value, args)?;
        let reply = reply_future.await?;
        Ok(reply)
    }
}
