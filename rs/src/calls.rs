use crate::{
    errors::{Error, Result, RtlError},
    prelude::*,
    utils::MaybeUninitBufferWriter,
};
use core::{future::Future, marker::PhantomData, task::Poll};
use gcore::stack_buffer;
use pin_project_lite::pin_project;

pub trait Action {
    type Args;

    #[cfg(not(feature = "ethexe"))]
    fn with_gas_limit(self, gas_limit: GasUnit) -> Self;
    fn with_value(self, value: ValueUnit) -> Self;
    fn with_args<F: FnOnce(Self::Args) -> Self::Args>(self, args_fn: F) -> Self;

    #[cfg(not(feature = "ethexe"))]
    fn gas_limit(&self) -> Option<GasUnit>;
    fn value(&self) -> ValueUnit;
    fn args(&self) -> &Self::Args;
}

#[allow(async_fn_in_trait)]
pub trait Call: Action<Args = <Self::Remoting as Remoting>::Args> {
    type Remoting: Remoting;
    type Output;

    async fn send(self, target: ActorId) -> Result<impl Reply<Output = Self::Output>>;

    async fn send_recv(self, target: ActorId) -> Result<Self::Output>
    where
        Self: Sized,
    {
        self.send(target).await?.recv().await
    }

    fn send_one_way(self, target: ActorId) -> Result<MessageId>
    where
        Self::Remoting: CallOneWay<Args = <Self::Remoting as Remoting>::Args>;
}

pub trait CallOneWay {
    type Args;

    fn send_one_way(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<MessageId>;
}

#[allow(async_fn_in_trait)]
pub trait Activation: Action {
    async fn send<S: AsRef<[u8]>>(
        self,
        code_id: CodeId,
        salt: S,
    ) -> Result<impl Reply<Output = ActorId>>;

    async fn send_recv<S: AsRef<[u8]>>(self, code_id: CodeId, salt: S) -> Result<ActorId>
    where
        Self: Sized,
    {
        self.send(code_id, salt).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Query: Action<Args = <Self::Remoting as Remoting>::Args> {
    type Remoting: Remoting;
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
pub trait Remoting: Clone {
    type Args: Default;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>>;

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>>;

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<Vec<u8>>;
}

pub struct RemotingAction<TRemoting: Remoting, TActionIo: ActionIo> {
    pub(crate) remoting: TRemoting,
    pub(crate) params: TActionIo::Params,
    #[cfg(not(feature = "ethexe"))]
    gas_limit: Option<GasUnit>,
    value: ValueUnit,
    args: TRemoting::Args,
}

impl<TRemoting: Remoting, TActionIo: ActionIo> RemotingAction<TRemoting, TActionIo> {
    pub fn new(remoting: TRemoting, params: TActionIo::Params) -> Self {
        Self {
            remoting,
            params,
            #[cfg(not(feature = "ethexe"))]
            gas_limit: Default::default(),
            value: Default::default(),
            args: Default::default(),
        }
    }
}

impl<TRemoting: Remoting, TActionIo: ActionIo> Action for RemotingAction<TRemoting, TActionIo> {
    type Args = TRemoting::Args;

    #[cfg(not(feature = "ethexe"))]
    fn with_gas_limit(self, gas_limit: GasUnit) -> Self {
        Self {
            gas_limit: Some(gas_limit),
            ..self
        }
    }

    fn with_value(self, value: ValueUnit) -> Self {
        Self { value, ..self }
    }

    fn with_args<F: FnOnce(Self::Args) -> Self::Args>(self, args_fn: F) -> Self {
        let RemotingAction { args, .. } = self;
        let args = args_fn(args);
        Self { args, ..self }
    }

    #[cfg(not(feature = "ethexe"))]
    fn gas_limit(&self) -> Option<GasUnit> {
        self.gas_limit
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
    type Remoting = TRemoting;
    type Output = TActionIo::Reply;

    async fn send(self, target: ActorId) -> Result<impl Reply<Output = TActionIo::Reply>> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_future = self
            .remoting
            .message(
                target,
                payload,
                #[cfg(not(feature = "ethexe"))]
                self.gas_limit,
                self.value,
                self.args,
            )
            .await?;
        Ok(CallTicket::<_, TActionIo>::new(reply_future))
    }

    fn send_one_way(self, target: ActorId) -> Result<MessageId>
    where
        Self::Remoting: CallOneWay<Args = TRemoting::Args>,
    {
        TActionIo::with_optimized_encode(&self.params, |payload| {
            self.remoting.send_one_way(
                target,
                payload,
                #[cfg(not(feature = "ethexe"))]
                self.gas_limit,
                self.value,
                self.args,
            )
        })
    }
}

impl<TRemoting, TActionIo> Activation for RemotingAction<TRemoting, TActionIo>
where
    TRemoting: Remoting,
    TActionIo: ActionIo<Reply = ()>,
{
    async fn send<S: AsRef<[u8]>>(
        self,
        code_id: CodeId,
        salt: S,
    ) -> Result<impl Reply<Output = ActorId>> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_future = self
            .remoting
            .activate(
                code_id,
                salt,
                payload,
                #[cfg(not(feature = "ethexe"))]
                self.gas_limit,
                self.value,
                self.args,
            )
            .await?;
        Ok(ActivationTicket::<_, TActionIo>::new(reply_future))
    }
}

impl<TRemoting, TActionIo> Query for RemotingAction<TRemoting, TActionIo>
where
    TRemoting: Remoting,
    TActionIo: ActionIo,
{
    type Remoting = TRemoting;
    type Output = TActionIo::Reply;

    async fn recv(self, target: ActorId) -> Result<Self::Output> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_bytes = self
            .remoting
            .query(
                target,
                payload,
                #[cfg(not(feature = "ethexe"))]
                self.gas_limit,
                self.value,
                self.args,
            )
            .await?;
        TActionIo::decode_reply(reply_bytes)
    }
}

pub trait ActionIo {
    const ROUTE: &'static [u8];
    type Params: Encode;
    type Reply: Decode;

    fn encode_call(value: &Self::Params) -> Vec<u8> {
        let mut result = Vec::with_capacity(Self::ROUTE.len() + Encode::size_hint(value));
        result.extend_from_slice(Self::ROUTE);
        Encode::encode_to(value, &mut result);
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

    fn with_optimized_encode<R>(value: &Self::Params, f: impl FnOnce(&[u8]) -> R) -> R {
        let size = Self::ROUTE.len() + Encode::encoded_size(value);
        stack_buffer::with_byte_buffer(size, |buffer| {
            let mut buffer_writer = MaybeUninitBufferWriter::new(buffer);
            buffer_writer.write(Self::ROUTE);
            Encode::encode_to(value, &mut buffer_writer);
            buffer_writer.with_buffer(f)
        })
    }
}

#[allow(async_fn_in_trait)]
pub trait Deploy<P: Program>: Action<Args = <Self::Remoting as Remoting>::Args> {
    type Remoting: Remoting;

    async fn deploy<S: AsRef<[u8]>>(self, code_id: CodeId, salt: S) -> Result<P>;
}

pub trait Program {
    type Remoting: Remoting;

    fn new(remoting: Self::Remoting, program_id: ActorId) -> Self;
    fn program_id(&self) -> ActorId;
}

pub struct DeployAction<A: ActionIo, P: Program> {
    pub(crate) remoting: P::Remoting,
    pub(crate) params: A::Params,
    #[cfg(not(feature = "ethexe"))]
    gas_limit: Option<GasUnit>,
    value: ValueUnit,
    args: <P::Remoting as Remoting>::Args,
    _program: PhantomData<P>,
}

impl<TActionIo: ActionIo, P: Program> DeployAction<TActionIo, P> {
    pub fn new(remoting: P::Remoting, params: TActionIo::Params) -> Self {
        Self {
            remoting,
            params,
            #[cfg(not(feature = "ethexe"))]
            gas_limit: Default::default(),
            value: Default::default(),
            args: Default::default(),
            _program: PhantomData,
        }
    }
}

impl<TActionIo: ActionIo, P: Program> Action for DeployAction<TActionIo, P> {
    type Args = <P::Remoting as Remoting>::Args;

    #[cfg(not(feature = "ethexe"))]
    fn with_gas_limit(self, gas_limit: GasUnit) -> Self {
        Self {
            gas_limit: Some(gas_limit),
            ..self
        }
    }

    fn with_value(self, value: ValueUnit) -> Self {
        Self { value, ..self }
    }

    fn with_args<F: FnOnce(Self::Args) -> Self::Args>(self, args_fn: F) -> Self {
        let DeployAction { args, .. } = self;
        let args = args_fn(args);
        Self { args, ..self }
    }

    #[cfg(not(feature = "ethexe"))]
    fn gas_limit(&self) -> Option<GasUnit> {
        self.gas_limit
    }

    fn value(&self) -> ValueUnit {
        self.value
    }

    fn args(&self) -> &Self::Args {
        &self.args
    }
}

impl<A: ActionIo<Reply = ()>, P: Program> Deploy<P> for DeployAction<A, P> {
    type Remoting = P::Remoting;

    async fn deploy<S: AsRef<[u8]>>(self, code_id: CodeId, salt: S) -> Result<P> {
        let remoting = self.remoting.clone();
        let program_id = Activation::send_recv(self, code_id, salt).await?;
        Ok(P::new(remoting, program_id))
    }
}

impl<A, P: Program> Activation for DeployAction<A, P>
where
    A: ActionIo<Reply = ()>,
{
    async fn send<S: AsRef<[u8]>>(
        self,
        code_id: CodeId,
        salt: S,
    ) -> Result<impl Reply<Output = ActorId>> {
        let payload = A::encode_call(&self.params);
        let reply_future = self
            .remoting
            .activate(
                code_id,
                salt,
                payload,
                #[cfg(not(feature = "ethexe"))]
                self.gas_limit,
                self.value,
                self.args,
            )
            .await?;
        Ok(ActivationTicket::<_, A>::new(reply_future))
    }
}

pub trait CallFuture: Action<Args = <Self::Remoting as RemotingMessage>::Args> + Future {
    type Remoting: RemotingMessage;

    fn send_one_way(self) -> Result<MessageId>
    where
        Self::Remoting: CallOneWay<Args = <Self::Remoting as RemotingMessage>::Args>;
}

pin_project! {
    #[project = Projection]
    pub struct CallAction<R: RemotingMessage, A: ActionIo> {
        pub(crate) remoting: R,
        pub(crate) target: ActorId,
        pub(crate) params: A::Params,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: Option<R::Args>,
        #[pin]
        future: Option<<R as RemotingMessage>::MessageFuture>,
    }
}

impl<R: RemotingMessage, A: ActionIo> CallAction<R, A> {
    pub fn new(remoting: R, target: ActorId, params: A::Params) -> Self {
        Self {
            remoting,
            target,
            params,
            #[cfg(not(feature = "ethexe"))]
            gas_limit: Default::default(),
            value: Default::default(),
            args: Some(Default::default()),
            future: None,
        }
    }
}

impl<R: RemotingMessage, A: ActionIo> Action for CallAction<R, A> {
    type Args = R::Args;

    #[cfg(not(feature = "ethexe"))]
    fn with_gas_limit(self, gas_limit: GasUnit) -> Self {
        Self {
            gas_limit: Some(gas_limit),
            ..self
        }
    }

    fn with_value(self, value: ValueUnit) -> Self {
        Self { value, ..self }
    }

    fn with_args<F: FnOnce(Self::Args) -> Self::Args>(self, args_fn: F) -> Self {
        let CallAction { args, .. } = self;
        let args = args.unwrap_or_default();
        let args = args_fn(args);
        Self {
            args: Some(args),
            ..self
        }
    }

    #[cfg(not(feature = "ethexe"))]
    fn gas_limit(&self) -> Option<GasUnit> {
        self.gas_limit
    }

    fn value(&self) -> ValueUnit {
        self.value
    }

    fn args(&self) -> &Self::Args {
        self.args.as_ref().unwrap()
    }
}

impl<R, A> Future for CallAction<R, A>
where
    A: ActionIo,
    R: RemotingMessage,
{
    type Output = Result<A::Reply>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let mut this = self.project();

        if this.future.is_none() {
            // If future is not set, create it.
            let args = this.args.take().unwrap_or_default();
            let res = this.remoting.clone().send_message(
                *this.target,
                A::encode_call(&this.params),
                #[cfg(not(feature = "ethexe"))]
                *this.gas_limit,
                *this.value,
                args,
            );
            match res {
                Err(e) => return Poll::Ready(Err(e)),
                Ok(future) => {
                    this.future.set(Some(future));
                }
            }
        }

        if let Some(mut future) = this.future.as_pin_mut() {
            // If future is already set, poll it.
            match future.as_mut().poll(cx) {
                Poll::Ready(Ok(payload)) => Poll::Ready(A::decode_reply(payload)),
                Poll::Ready(Err(err)) => Poll::Ready(Err(err.into())),
                Poll::Pending => Poll::Pending,
            }
        } else {
            panic!("CallAction polled after completion");
        }
    }
}

impl<R, A> CallFuture for CallAction<R, A>
where
    R: RemotingMessage,
    A: ActionIo,
{
    type Remoting = R;

    fn send_one_way(mut self) -> Result<MessageId>
    where
        Self::Remoting: CallOneWay<Args = <Self::Remoting as RemotingMessage>::Args>,
    {
        let args = self.args.take().unwrap_or_default();
        A::with_optimized_encode(&self.params, |payload| {
            self.remoting.send_one_way(
                self.target,
                payload,
                #[cfg(not(feature = "ethexe"))]
                self.gas_limit,
                self.value,
                args,
            )
        })
    }
}

pub trait MessageFuture: Future<Output = Result<Vec<u8>, Self::Error>> {
    type Error: Into<Error>;

    fn message_id(&self) -> MessageId;
}

pub trait DeployFuture: MessageFuture {
    fn program_id(&self) -> ActorId;
}

pub trait RemotingMessage: Clone {
    type Args: Default;
    type MessageFuture: MessageFuture;
    // type DeployFuture: DeployFuture<Remoting = Self>;

    fn send_message(
        self,
        target: ActorId,
        payload: Vec<u8>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<Self::MessageFuture>;

    // fn deploy<P: AsRef<[u8]>>(
    //     self,
    //     code_id: CodeId,
    //     salt: impl AsRef<[u8]>,
    //     payload: P,
    //     #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
    //     value: ValueUnit,
    //     args: Self::Args,
    // ) -> Result<Self::DeployFuture>;
}

// impl<T, A> Remoting for T
// where
//     A: Default,
//     T: RemotingMessage<Args = A>,
// {
//     type Args = A;

//     async fn activate(
//         self,
//         code_id: CodeId,
//         salt: impl AsRef<[u8]>,
//         payload: impl AsRef<[u8]>,
//         #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
//         value: ValueUnit,
//         args: Self::Args,
//     ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
//         let future = self.deploy(code_id, salt, payload, gas_limit, value, args)?;
//         let program_id = future.program_id();
//         Ok(future.map(move |res| res.map(|vec| (program_id, vec))))
//     }

//     async fn message(
//         self,
//         target: ActorId,
//         payload: impl AsRef<[u8]>,
//         #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
//         value: ValueUnit,
//         args: Self::Args,
//     ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
//         let future = self.message(target, payload, gas_limit, value, args)?;
//         Ok(future)
//     }

//     async fn query(
//         self,
//         target: ActorId,
//         payload: impl AsRef<[u8]>,
//         #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
//         value: ValueUnit,
//         args: Self::Args,
//     ) -> Result<Vec<u8>> {
//         let message_future = self.message(
//             target,
//             payload,
//             #[cfg(not(feature = "ethexe"))]
//             gas_limit,
//             value,
//             args,
//         )?;
//         message_future.await
//     }
// }
