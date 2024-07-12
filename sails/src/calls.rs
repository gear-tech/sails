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
    type TParams: EncodeDecodeWithRoute<Reply = TReply>;

    async fn send(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, Self::TParams, TReply>>;

    async fn send_recv(self, target: ActorId) -> Result<TReply>
    where
        Self: Sized,
    {
        self.send(target).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Activation<TArgs>: Action<TArgs> {
    type TParams: EncodeDecodeWithRoute<Reply = ()>;

    async fn send(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
    ) -> Result<ActivationTicket<impl Future<Output = Result<(ActorId, Vec<u8>)>>, Self::TParams>>;

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

pub struct CallTicket<TReplyFuture, TParams, TReply> {
    reply_future: TReplyFuture,
    _params: PhantomData<TParams>,
    _reply: PhantomData<TReply>,
}

impl<TReplyFuture, TParams, TReply> CallTicket<TReplyFuture, TParams, TReply>
where
    TReplyFuture: Future<Output = Result<Vec<u8>>>,
    TParams: EncodeDecodeWithRoute<Reply = TReply>,
{
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _params: PhantomData,
            _reply: PhantomData,
        }
    }

    pub async fn recv(self) -> Result<TReply> {
        let reply_bytes = self.reply_future.await?;
        TParams::decode_reply(reply_bytes)
    }
}

pub struct ActivationTicket<TReplyFuture, TParams> {
    reply_future: TReplyFuture,
    _params: PhantomData<TParams>,
}

impl<TReplyFuture, TParams> ActivationTicket<TReplyFuture, TParams>
where
    TReplyFuture: Future<Output = Result<(ActorId, Vec<u8>)>>,
    TParams: EncodeDecodeWithRoute<Reply = ()>,
{
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _params: PhantomData,
        }
    }

    pub async fn recv(self) -> Result<ActorId> {
        let reply = self.reply_future.await?;
        TParams::decode_reply(reply.1)?;
        Ok(reply.0)
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

pub struct RemotingAction<TRemoting, TArgs, TParams, TReply> {
    remoting: TRemoting,
    params: TParams,
    value: ValueUnit,
    args: TArgs,
    _treply: PhantomData<TReply>,
}

impl<TRemoting, TArgs, TParams, TReply> RemotingAction<TRemoting, TArgs, TParams, TReply>
where
    TArgs: Default,
{
    pub fn new(remoting: TRemoting, params: TParams) -> Self {
        Self {
            remoting,
            params,
            value: Default::default(),
            args: Default::default(),
            _treply: PhantomData,
        }
    }
}

impl<TRemoting, TArgs, TParams, TReply> Action<TArgs>
    for RemotingAction<TRemoting, TArgs, TParams, TReply>
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
    for RemotingAction<TRemoting, TArgs, TParams, TReply>
where
    TRemoting: Remoting<TArgs>,
    TParams: EncodeDecodeWithRoute<Reply = TReply>,
{
    type TParams = TParams;

    async fn send(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, TParams, TReply>> {
        let payload = self.params.encode_call();
        let reply_future = self
            .remoting
            .message(target, payload, self.value, self.args)
            .await?;
        Ok(CallTicket::new(reply_future))
    }
}

impl<TRemoting, TArgs, TParams> Activation<TArgs>
    for RemotingAction<TRemoting, TArgs, TParams, ActorId>
where
    TRemoting: Remoting<TArgs>,
    TParams: EncodeDecodeWithRoute<Reply = ()>,
{
    type TParams = TParams;

    async fn send(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
    ) -> Result<ActivationTicket<impl Future<Output = Result<(ActorId, Vec<u8>)>>, TParams>> {
        let payload = self.params.encode_call();
        let reply_future = self
            .remoting
            .activate(code_id, salt, payload, self.value, self.args)
            .await?;
        Ok(ActivationTicket::new(reply_future))
    }
}

impl<TRemoting, TArgs, TParams, TReply> Query<TArgs, TReply>
    for RemotingAction<TRemoting, TArgs, TParams, TReply>
where
    TRemoting: Remoting<TArgs>,
    TParams: EncodeDecodeWithRoute<Reply = TReply>,
{
    async fn recv(self, target: ActorId) -> Result<TReply> {
        let payload = self.params.encode_call();
        let reply_bytes = self
            .remoting
            .query(target, payload, self.value, self.args)
            .await?;
        TParams::decode_reply(reply_bytes)
    }
}

pub trait EncodeDecodeWithRoute: Encode {
    const ROUTE: &'static [u8];
    type Reply: Decode;

    fn encode_call(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(Self::ROUTE.len() + self.encoded_size());
        result.extend_from_slice(Self::ROUTE);
        self.encode_to(&mut result);
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
