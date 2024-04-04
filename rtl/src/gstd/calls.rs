use crate::calls::Sender as SenderTrait;
use crate::{errors::Result, prelude::*};
use core::future::Future;
use futures::FutureExt;
use gstd::msg;

#[derive(Debug, Default, Clone)]
pub struct Args {
    pub reply_deposit: GasUnit,
}

#[derive(Debug, Default, Clone)]
pub struct Sender;

impl SenderTrait<Args> for Sender {
    async fn send_to(
        self,
        _target: ActorId,
        payload: Vec<u8>,
        value: ValueUnit,
        args: Args,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let response_future =
            msg::send_bytes_for_reply(gstd::ActorId::zero(), payload, value, args.reply_deposit)?;
        let response_future = response_future.map(|result| result.map_err(Into::into));
        Ok(response_future)
    }
}
