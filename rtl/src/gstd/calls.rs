use crate::{calls::Remoting, errors::Result, prelude::*};
use core::future::Future;
use futures::FutureExt;
use gstd::{msg, prog};

#[derive(Debug, Default, Clone)]
pub struct GStdArgs {
    reply_deposit: Option<GasUnit>,
    gas_limit: Option<GasUnit>,
}

impl GStdArgs {
    pub fn with_reply_deposit(mut self, reply_deposit: Option<GasUnit>) -> Self {
        self.reply_deposit = reply_deposit;
        self
    }

    pub fn with_gas_limit(mut self, gas_limit: Option<GasUnit>) -> Self {
        self.gas_limit = gas_limit;
        self
    }

    pub fn reply_deposit(&self) -> Option<GasUnit> {
        self.reply_deposit
    }

    pub fn gas_limit(&self) -> Option<GasUnit> {
        self.gas_limit
    }
}

#[derive(Debug, Default, Clone)]
pub struct GStdRemoting;

impl Remoting<GStdArgs> for GStdRemoting {
    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GStdArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let reply_future = prog::create_program_bytes_for_reply(
            code_id.into(),
            salt,
            payload,
            value,
            args.reply_deposit.unwrap_or_default(),
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
        args: GStdArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let reply_future = if let Some(gas_limit) = args.gas_limit {
            msg::send_bytes_with_gas_for_reply(
                target.into(),
                payload,
                gas_limit,
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        } else {
            msg::send_bytes_for_reply(
                target.into(),
                payload,
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        };
        let reply_future = reply_future.map(|result| result.map_err(Into::into));
        Ok(reply_future)
    }
}
