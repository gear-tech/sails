use crate::errors::{Result, RtlError};
use crate::prelude::*;
use core::{future::Future, marker::PhantomData};

pub trait Call<TArgs, TReply, const PREFIX_SIZE: usize> {
    #[allow(async_fn_in_trait)]
    async fn send_to(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, TReply, PREFIX_SIZE>>;

    fn with_value(self, value: ValueUnit) -> Self;

    fn with_args(self, args: TArgs) -> Self;

    fn value(&self) -> ValueUnit;

    fn args(&self) -> &TArgs;
}

pub struct CallTicket<TReplyFuture, TReply, const PREFIX_SIZE: usize> {
    reply_prefix: &'static [u8; PREFIX_SIZE],
    reply_future: TReplyFuture,
    _treply: PhantomData<TReply>,
}

impl<TReplyFuture, TReply, const PREFIX_SIZE: usize> CallTicket<TReplyFuture, TReply, PREFIX_SIZE>
where
    TReplyFuture: Future<Output = Result<Vec<u8>>>,
    TReply: Decode,
{
    pub(crate) fn new(
        reply_prefix: &'static [u8; PREFIX_SIZE],
        reply_future: TReplyFuture,
    ) -> Self {
        Self {
            reply_prefix,
            reply_future,
            _treply: PhantomData,
        }
    }

    pub async fn reply(self) -> Result<TReply> {
        let reply_bytes = self.reply_future.await?;
        if !reply_bytes.starts_with(self.reply_prefix) {
            Err(RtlError::UnexpectedReply)?
        }
        let mut reply_bytes = &reply_bytes[PREFIX_SIZE..];
        Ok(TReply::decode(&mut reply_bytes)?)
    }
}

pub trait Sender<TArgs> {
    #[allow(async_fn_in_trait)]
    async fn send_to(
        self,
        target: ActorId,
        payload: Vec<u8>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>>;
}

pub struct CallViaSender<TSender, TArgs, TReply, const PREFIX_SIZE: usize> {
    sender: TSender,
    payload: Vec<u8>,
    value: ValueUnit,
    args: TArgs,
    reply_prefix: &'static [u8; PREFIX_SIZE],
    _treply: PhantomData<TReply>,
}

impl<TSender, TArgs, TReply, const PREFIX_SIZE: usize>
    CallViaSender<TSender, TArgs, TReply, PREFIX_SIZE>
where
    TArgs: Default,
{
    pub fn new<TParams>(
        sender: TSender,
        params: TParams,
        reply_prefix: &'static [u8; PREFIX_SIZE],
    ) -> Self
    where
        TParams: Encode,
    {
        let mut payload = Vec::new();
        params.encode_to(&mut payload);
        Self {
            sender,
            payload,
            value: Default::default(),
            args: Default::default(),
            reply_prefix,
            _treply: PhantomData,
        }
    }
}

impl<TSender, TArgs, TReply, const PREFIX_SIZE: usize> Call<TArgs, TReply, PREFIX_SIZE>
    for CallViaSender<TSender, TArgs, TReply, PREFIX_SIZE>
where
    TSender: Sender<TArgs>,
    TReply: Decode,
{
    async fn send_to(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, TReply, PREFIX_SIZE>> {
        let reply_future = self
            .sender
            .send_to(target, self.payload, self.value, self.args)
            .await?;
        Ok(CallTicket::new(self.reply_prefix, reply_future))
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
