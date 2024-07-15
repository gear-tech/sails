use crate::{
    errors::{Error, Result, RtlError},
    prelude::*,
};
use core::{future::Future, marker::PhantomData};

pub trait Action<TArgs> {
    fn with_value(self, value: ValueUnit) -> Self;

    fn with_args(self, args: TArgs) -> Self;

    fn value(&self) -> ValueUnit;

    fn args(&self) -> &TArgs;
}

#[allow(async_fn_in_trait)]
pub trait Call<TArgs, TReply>: Action<TArgs> {
    async fn send(self, target: ActorId) -> Result<impl Reply<TReply>>;

    async fn send_recv(self, target: ActorId) -> Result<TReply>
    where
        Self: Sized,
    {
        self.send(target).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Activation<TArgs>: Action<TArgs> {
    async fn send(self, code_id: CodeId, salt: impl AsRef<[u8]>) -> Result<impl Reply<ActorId>>;

    async fn send_recv(self, code_id: CodeId, salt: impl AsRef<[u8]>) -> Result<ActorId>
    where
        Self: Sized,
    {
        self.send(code_id, salt).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Query<TArgs, TReply>: Action<TArgs> {
    async fn recv(self, target: ActorId) -> Result<TReply>;
}

#[allow(async_fn_in_trait)]
pub trait Reply<TReply> {
    async fn recv(self) -> Result<TReply>;
}

pub struct CallTicket<TReplyFuture, TParams> {
    reply_future: TReplyFuture,
    _params: PhantomData<TParams>,
}

impl<TReplyFuture, TParams> CallTicket<TReplyFuture, TParams> {
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _params: PhantomData,
        }
    }
}

impl<TReplyFuture, TParams, TReply> Reply<TReply> for CallTicket<TReplyFuture, TParams>
where
    TReplyFuture: Future<Output = Result<Vec<u8>>>,
    TParams: ActionIo<Reply = TReply>,
{
    async fn recv(self) -> Result<TReply> {
        let reply_bytes = self.reply_future.await?;
        TParams::decode_reply(reply_bytes)
    }
}

pub struct ActivationTicket<TReplyFuture, TParams> {
    reply_future: TReplyFuture,
    _params: PhantomData<TParams>,
}

impl<TReplyFuture, TParams> ActivationTicket<TReplyFuture, TParams> {
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _params: PhantomData,
        }
    }
}

impl<TReplyFuture, TParams> Reply<ActorId> for ActivationTicket<TReplyFuture, TParams>
where
    TReplyFuture: Future<Output = Result<(ActorId, Vec<u8>)>>,
    TParams: ActionIo<Reply = ()>,
{
    async fn recv(self) -> Result<ActorId> {
        let (actor_id, payload) = self.reply_future.await?;
        TParams::decode_reply(payload)?;
        Ok(actor_id)
    }
}

#[allow(async_fn_in_trait)]
pub trait Remoting<TArgs> {
    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>>;

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>>;

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<Vec<u8>>;
}

pub struct RemotingAction<TRemoting, TArgs, TParams: ActionIo> {
    remoting: TRemoting,
    params: TParams::Params,
    value: ValueUnit,
    args: TArgs,
}

impl<TRemoting, TArgs, TParams: ActionIo> RemotingAction<TRemoting, TArgs, TParams>
where
    TArgs: Default,
{
    pub fn new(remoting: TRemoting, params: TParams::Params) -> Self {
        Self {
            remoting,
            params,
            value: Default::default(),
            args: Default::default(),
        }
    }
}

impl<TRemoting, TArgs, TParams: ActionIo> Action<TArgs>
    for RemotingAction<TRemoting, TArgs, TParams>
{
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

impl<TRemoting, TArgs, TParams, TReply> Call<TArgs, TReply>
    for RemotingAction<TRemoting, TArgs, TParams>
where
    TRemoting: Remoting<TArgs>,
    TParams: ActionIo<Reply = TReply>,
{
    async fn send(self, target: ActorId) -> Result<impl Reply<TParams::Reply>> {
        let payload = TParams::encode_call(&self.params);
        let reply_future = self
            .remoting
            .message(target, payload, self.value, self.args)
            .await?;
        Ok(CallTicket::<_, TParams>::new(reply_future))
    }
}

impl<TRemoting, TArgs, TParams> Activation<TArgs> for RemotingAction<TRemoting, TArgs, TParams>
where
    TRemoting: Remoting<TArgs>,
    TParams: ActionIo<Reply = ()>,
{
    async fn send(self, code_id: CodeId, salt: impl AsRef<[u8]>) -> Result<impl Reply<ActorId>> {
        let payload = TParams::encode_call(&self.params);
        let reply_future = self
            .remoting
            .activate(code_id, salt, payload, self.value, self.args)
            .await?;
        Ok(ActivationTicket::<_, TParams>::new(reply_future))
    }
}

impl<TRemoting, TArgs, TParams, TReply> Query<TArgs, TReply>
    for RemotingAction<TRemoting, TArgs, TParams>
where
    TRemoting: Remoting<TArgs>,
    TParams: ActionIo<Reply = TReply>,
{
    async fn recv(self, target: ActorId) -> Result<TReply> {
        let payload = TParams::encode_call(&self.params);
        let reply_bytes = self
            .remoting
            .query(target, payload, self.value, self.args)
            .await?;
        TParams::decode_reply(reply_bytes)
    }
}

pub trait ActionIo {
    const ROUTE: &'static [u8];
    type Params: Encode;
    type Reply: Decode;

    fn encode_call(value: &Self::Params) -> Vec<u8> {
        let mut result = Vec::with_capacity(Self::ROUTE.len() + value.encoded_size());
        result.extend_from_slice(Self::ROUTE);
        value.encode_to(&mut result);
        result
    }

    fn decode_reply(payload: impl AsRef<[u8]>) -> Result<Self::Reply> {
        let mut value = payload.as_ref();
        if !value.starts_with(Self::ROUTE) {
            return Err(Error::Rtl(RtlError::ReplyPrefixMismatches));
        }
        value = &value[Self::ROUTE.len()..];
        Decode::decode(&mut value).map_err(Error::Codec)
    }
}
