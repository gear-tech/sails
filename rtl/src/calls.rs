use crate::errors::{Result, RtlError};
use crate::prelude::*;
use core::{future::Future, marker::PhantomData};

pub trait Call<TArgs, TReply> {
    #[allow(async_fn_in_trait)]
    async fn send_to(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, TReply>>;

    fn with_value(self, value: ValueUnit) -> Self;

    fn with_args(self, args: TArgs) -> Self;

    fn value(&self) -> ValueUnit;

    fn args(&self) -> &TArgs;
}

pub struct CallTicket<TReplyFuture, TReply> {
    route: &'static [u8],
    reply_future: TReplyFuture,
    _treply: PhantomData<TReply>,
}

impl<TReplyFuture, TReply> CallTicket<TReplyFuture, TReply>
where
    TReplyFuture: Future<Output = Result<Vec<u8>>>,
    TReply: Decode,
{
    pub(crate) fn new(route: &'static [u8], reply_future: TReplyFuture) -> Self {
        Self {
            route,
            reply_future,
            _treply: PhantomData,
        }
    }

    pub async fn reply(self) -> Result<TReply> {
        let reply_bytes = self.reply_future.await?;
        if !reply_bytes.starts_with(self.route) {
            Err(RtlError::UnexpectedReply)?
        }
        let mut reply_bytes = &reply_bytes[self.route.len()..];
        Ok(TReply::decode(&mut reply_bytes)?)
    }
}

pub trait Sender<TArgs> {
    #[allow(async_fn_in_trait)]
    async fn send_to(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>>;
}

pub struct CallViaSender<TSender, TArgs, TReply> {
    sender: TSender,
    route: &'static [u8],
    payload: Vec<u8>,
    value: ValueUnit,
    args: TArgs,
    _treply: PhantomData<TReply>,
}

impl<TSender, TArgs, TReply> CallViaSender<TSender, TArgs, TReply>
where
    TArgs: Default,
{
    pub fn new<TParams>(sender: TSender, route: &'static [u8], params: TParams) -> Self
    where
        TParams: Encode,
    {
        let mut payload = route.to_vec();
        params.encode_to(&mut payload);
        Self {
            sender,
            route,
            payload,
            value: Default::default(),
            args: Default::default(),
            _treply: PhantomData,
        }
    }
}

impl<TSender, TArgs, TReply> Call<TArgs, TReply> for CallViaSender<TSender, TArgs, TReply>
where
    TSender: Sender<TArgs>,
    TReply: Decode,
{
    async fn send_to(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, TReply>> {
        let reply_future = self
            .sender
            .send_to(target, self.payload, self.value, self.args)
            .await?;
        Ok(CallTicket::new(self.route, reply_future))
    }

    fn with_value(self, value: ValueUnit) -> Self {
        Self { value, ..self }
    }

    fn with_args(self, args: TArgs) -> Self {
        Self { args, ..self }
    }

    fn value(&self) -> ValueUnit {
        self.value
    }

    fn args(&self) -> &TArgs {
        &self.args
    }
}
