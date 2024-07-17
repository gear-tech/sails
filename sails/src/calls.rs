use crate::{
    errors::{Error, Result, RtlError},
    prelude::*,
};
use core::{future::Future, marker::PhantomData};

pub trait Action {
    type Args;

    fn with_value(self, value: ValueUnit) -> Self;

    fn with_args(self, args: Self::Args) -> Self;

    fn value(&self) -> ValueUnit;

    fn args(&self) -> &Self::Args;
}

#[allow(async_fn_in_trait)]
pub trait Call: Action {
    type Output;

    async fn send(self, target: ActorId) -> Result<impl Reply<Output = Self::Output>>;

    async fn send_recv(self, target: ActorId) -> Result<Self::Output>
    where
        Self: Sized,
    {
        self.send(target).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Activation: Action {
    async fn send(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
    ) -> Result<impl Reply<Output = ActorId>>;

    async fn send_recv(self, code_id: CodeId, salt: impl AsRef<[u8]>) -> Result<ActorId>
    where
        Self: Sized,
    {
        self.send(code_id, salt).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Query: Action {
    type Output;

    async fn recv(self, target: ActorId) -> Result<Self::Output>;
}

#[allow(async_fn_in_trait)]
pub trait Reply {
    type Output;

    async fn recv(self) -> Result<Self::Output>;
}

struct CallTicket<TReplyFuture, TActionIo> {
    reply_future: TReplyFuture,
    _io: PhantomData<TActionIo>,
}

impl<TReplyFuture, TActionIo> CallTicket<TReplyFuture, TActionIo> {
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _io: PhantomData,
        }
    }
}

impl<TReplyFuture, TActionIo> Reply for CallTicket<TReplyFuture, TActionIo>
where
    TReplyFuture: Future<Output = Result<Vec<u8>>>,
    TActionIo: ActionIo,
{
    type Output = TActionIo::Reply;

    async fn recv(self) -> Result<Self::Output> {
        let reply_bytes = self.reply_future.await?;
        TActionIo::decode_reply(reply_bytes)
    }
}

struct ActivationTicket<TReplyFuture, TActionIo> {
    reply_future: TReplyFuture,
    _io: PhantomData<TActionIo>,
}

impl<TReplyFuture, TActionIo> ActivationTicket<TReplyFuture, TActionIo> {
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _io: PhantomData,
        }
    }
}

impl<TReplyFuture, TActionIo> Reply for ActivationTicket<TReplyFuture, TActionIo>
where
    TReplyFuture: Future<Output = Result<(ActorId, Vec<u8>)>>,
    TActionIo: ActionIo<Reply = ()>,
{
    type Output = ActorId;

    async fn recv(self) -> Result<Self::Output> {
        let (actor_id, payload) = self.reply_future.await?;
        TActionIo::decode_reply(payload)?;
        Ok(actor_id)
    }
}

#[allow(async_fn_in_trait)]
pub trait Remoting {
    type Args: Default;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>>;

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>>;

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<Vec<u8>>;
}

pub struct RemotingAction<TRemoting: Remoting, TActionIo: ActionIo> {
    remoting: TRemoting,
    params: TActionIo::Params,
    value: ValueUnit,
    args: TRemoting::Args,
}

impl<TRemoting: Remoting, TActionIo: ActionIo> RemotingAction<TRemoting, TActionIo> {
    pub fn new(remoting: TRemoting, params: TActionIo::Params) -> Self {
        Self {
            remoting,
            params,
            value: Default::default(),
            args: Default::default(),
        }
    }
}

impl<TRemoting: Remoting, TActionIo: ActionIo> Action for RemotingAction<TRemoting, TActionIo> {
    type Args = TRemoting::Args;

    fn with_value(self, value: ValueUnit) -> Self {
        Self { value, ..self }
    }

    fn with_args(self, args: Self::Args) -> Self {
        Self { args, ..self }
    }

    fn value(&self) -> ValueUnit {
        self.value
    }

    fn args(&self) -> &Self::Args {
        &self.args
    }
}

impl<TRemoting, TActionIo> Call for RemotingAction<TRemoting, TActionIo>
where
    TRemoting: Remoting,
    TActionIo: ActionIo,
{
    type Output = TActionIo::Reply;

    async fn send(self, target: ActorId) -> Result<impl Reply<Output = TActionIo::Reply>> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_future = self
            .remoting
            .message(target, payload, self.value, self.args)
            .await?;
        Ok(CallTicket::<_, TActionIo>::new(reply_future))
    }
}

impl<TRemoting, TActionIo> Activation for RemotingAction<TRemoting, TActionIo>
where
    TRemoting: Remoting,
    TActionIo: ActionIo<Reply = ()>,
{
    async fn send(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
    ) -> Result<impl Reply<Output = ActorId>> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_future = self
            .remoting
            .activate(code_id, salt, payload, self.value, self.args)
            .await?;
        Ok(ActivationTicket::<_, TActionIo>::new(reply_future))
    }
}

impl<TRemoting, TActionIo> Query for RemotingAction<TRemoting, TActionIo>
where
    TRemoting: Remoting,
    TActionIo: ActionIo,
{
    type Output = TActionIo::Reply;

    async fn recv(self, target: ActorId) -> Result<Self::Output> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_bytes = self
            .remoting
            .query(target, payload, self.value, self.args)
            .await?;
        TActionIo::decode_reply(reply_bytes)
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
