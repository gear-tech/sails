use crate::{
    errors::{Result, RtlError},
    ActorId, CodeId, ValueUnit, Vec,
};
use core::{future::Future, marker::PhantomData};
use parity_scale_codec::{Decode, Encode};

pub trait Action<TArgs> {
    fn with_value(self, value: ValueUnit) -> Self;

    fn with_args(self, args: TArgs) -> Self;

    fn value(&self) -> ValueUnit;

    fn args(&self) -> &TArgs;
}

#[allow(async_fn_in_trait)]
pub trait Call<TArgs, TReply>: Action<TArgs> {
    async fn publish(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, TReply>>;
}

#[allow(async_fn_in_trait)]
pub trait Activation<TArgs>: Action<TArgs> {
    async fn publish(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
    ) -> Result<ActivationTicket<impl Future<Output = Result<(ActorId, Vec<u8>)>>>>;
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
            Err(RtlError::ReplyPrefixMismatches)?
        }
        let mut reply_bytes = &reply_bytes[self.route.len()..];
        Ok(TReply::decode(&mut reply_bytes)?)
    }
}

pub struct ActivationTicket<TReplyFuture> {
    route: &'static [u8],
    reply_future: TReplyFuture,
}

impl<TReplyFuture> ActivationTicket<TReplyFuture>
where
    TReplyFuture: Future<Output = Result<(ActorId, Vec<u8>)>>,
{
    pub(crate) fn new(route: &'static [u8], reply_future: TReplyFuture) -> Self {
        Self {
            route,
            reply_future,
        }
    }

    pub async fn reply(self) -> Result<ActorId> {
        let reply = self.reply_future.await?;
        if reply.1 != self.route {
            Err(RtlError::ReplyPrefixMismatches)?
        }
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
}

pub struct RemotingAction<TRemoting, TArgs, TReply> {
    remoting: TRemoting,
    route: &'static [u8],
    payload: Vec<u8>,
    value: ValueUnit,
    args: TArgs,
    _treply: PhantomData<TReply>,
}

impl<TRemoting, TArgs, TReply> RemotingAction<TRemoting, TArgs, TReply>
where
    TArgs: Default,
{
    pub fn new<TParams>(remoting: TRemoting, route: &'static [u8], params: TParams) -> Self
    where
        TParams: Encode,
    {
        let mut payload = route.to_vec();
        params.encode_to(&mut payload);
        Self {
            remoting,
            route,
            payload,
            value: Default::default(),
            args: Default::default(),
            _treply: PhantomData,
        }
    }
}

impl<TRemoting, TArgs, TReply> Action<TArgs> for RemotingAction<TRemoting, TArgs, TReply> {
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

impl<TRemoting, TArgs, TReply> Call<TArgs, TReply> for RemotingAction<TRemoting, TArgs, TReply>
where
    TRemoting: Remoting<TArgs>,
    TReply: Decode,
{
    async fn publish(
        self,
        target: ActorId,
    ) -> Result<CallTicket<impl Future<Output = Result<Vec<u8>>>, TReply>> {
        let reply_future = self
            .remoting
            .message(target, self.payload, self.value, self.args)
            .await?;
        Ok(CallTicket::new(self.route, reply_future))
    }
}

impl<TRemoting, TArgs> Activation<TArgs> for RemotingAction<TRemoting, TArgs, ActorId>
where
    TRemoting: Remoting<TArgs>,
{
    async fn publish(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
    ) -> Result<ActivationTicket<impl Future<Output = Result<(ActorId, Vec<u8>)>>>> {
        let reply_future = self
            .remoting
            .activate(code_id, salt, self.payload, self.value, self.args)
            .await?;
        Ok(ActivationTicket::new(self.route, reply_future))
    }
}
