use crate::prelude::*;
use core::{future::Future, marker::PhantomData};

pub trait Call<TArgs, TReply> {
    #[allow(async_fn_in_trait)]
    async fn send_to(self, target: ActorId) -> CallTicket<impl Future<Output = Vec<u8>>, TReply>;

    fn with_value(self, value: Value) -> Self;

    fn with_args(self, args: TArgs) -> Self;

    fn value(&self) -> Value;

    fn args(&self) -> &TArgs;
}

pub struct CallTicket<TReplyFuture, TReply> {
    reply_future: TReplyFuture,
    _treply: PhantomData<TReply>,
}

impl<TReplyFuture, TReply> CallTicket<TReplyFuture, TReply>
where
    TReplyFuture: Future<Output = Vec<u8>>,
    TReply: Decode,
{
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _treply: PhantomData,
        }
    }

    pub async fn reply(self) -> TReply {
        let reply_bytes = self.reply_future.await;
        TReply::decode(&mut reply_bytes.as_slice()).expect("Unable to decode reply")
    }
}

pub trait Sender<TArgs> {
    #[allow(async_fn_in_trait)]
    async fn send_to(self, target: ActorId, payload: Vec<u8>, value: Value, args: TArgs)
        -> Vec<u8>;
}

pub(crate) struct CallViaSender<TSender, TArgs, TReply> {
    sender: TSender,
    payload: Vec<u8>,
    value: Value,
    args: TArgs,
    _marker: PhantomData<TReply>,
}

impl<TSender, TArgs, TReply> Call<TArgs, TReply> for CallViaSender<TSender, TArgs, TReply>
where
    TSender: Sender<TArgs>,
    TReply: Decode,
{
    async fn send_to(self, target: ActorId) -> CallTicket<impl Future<Output = Vec<u8>>, TReply> {
        let future = self
            .sender
            .send_to(target, self.payload, self.value, self.args);
        CallTicket::new(future)
    }

    fn with_value(self, value: Value) -> Self {
        Self { value, ..self }
    }

    fn with_args(self, args: TArgs) -> Self {
        Self { args, ..self }
    }

    fn value(&self) -> Value {
        self.value
    }

    fn args(&self) -> &TArgs {
        &self.args
    }
}
