use crate::calls::Sender as SenderTrait;
use crate::{errors::Result, prelude::*};
use core::future::Future;
use futures::FutureExt;
use gstd::{exec, msg};

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
pub struct Sender;

impl SenderTrait<Args> for Sender {
    async fn send_to(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: Args,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let response_future =
            msg::send_bytes_for_reply(target.into(), payload, value, args.reply_deposit)?;
        if args.reply_deposit > 0 {
            exec::reply_deposit(response_future.waiting_reply_to, args.reply_deposit)?;
        }
        let response_future = response_future.map(|result| result.map_err(Into::into));
        Ok(response_future)
    }
}
