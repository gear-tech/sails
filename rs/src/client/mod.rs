use crate::{calls::ActionIo, prelude::*};
use core::{
    error::Error,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(not(target_arch = "wasm32"))]
mod mock_env;

#[cfg(feature = "gtest")]
#[cfg(not(target_arch = "wasm32"))]
pub mod gtest_env;

pub trait GearEnv: Clone {
    type Params: Default;
    type Error: Error;
    type MessageState;
}

type Route = &'static str;

pub struct Deployment<E: GearEnv, A> {
    env: E,
    code_id: CodeId,
    salt: Vec<u8>,
    _phantom: PhantomData<A>,
}

impl<E: GearEnv, A> Deployment<E, A> {
    pub fn new(env: E, code_id: CodeId, salt: Vec<u8>) -> Self {
        Deployment {
            env,
            code_id,
            salt,
            _phantom: PhantomData,
        }
    }

    pub fn with_env<N: GearEnv>(self, env: N) -> Deployment<N, A> {
        let Self {
            env: _,
            code_id,
            salt,
            _phantom: _,
        } = self;
        Deployment {
            env,
            code_id,
            salt,
            _phantom: PhantomData,
        }
    }

    pub fn pending_ctor<T: ActionIo>(self, args: T::Params) -> PendingCtor<E, A, T> {
        PendingCtor::new(self.env, self.code_id, self.salt, args)
    }
}

pub struct Actor<A, E: GearEnv> {
    env: E,
    id: ActorId,
    _phantom: PhantomData<A>,
}

impl<A, E: GearEnv> Actor<A, E> {
    pub fn new(id: ActorId, env: E) -> Self {
        Actor {
            env,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn with_env<N: GearEnv>(self, env: N) -> Actor<A, N> {
        let Self {
            env: _,
            id,
            _phantom: _,
        } = self;
        Actor {
            env,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn service<S>(&self, route: Route) -> Service<S, E> {
        Service::new(self.id, route, self.env.clone())
    }
}

pub struct Service<S, E: GearEnv> {
    env: E,
    actor_id: ActorId,
    route: Route,
    _phantom: PhantomData<S>,
}

impl<S, E: GearEnv> Service<S, E> {
    pub fn new(actor_id: ActorId, route: Route, env: E) -> Self {
        Service {
            env,
            actor_id,
            route,
            _phantom: PhantomData,
        }
    }

    pub fn pending_call<T: Encode, O: Decode>(&self, route: Route, args: T) -> PendingCall<E, O> {
        let payload = (self.route, route, args).encode();

        PendingCall::new(self.actor_id, self.env.clone(), payload)
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCall<E: GearEnv, O: Decode> {
        env: E,
        destination: ActorId,
        params: Option<E::Params>,
        payload: Option<Vec<u8>>,
        _output: PhantomData<O>,
        #[pin]
        state: Option<E::MessageState>
    }
}

impl<E: GearEnv, O: Decode> PendingCall<E, O> {
    pub fn new(destination: ActorId, env: E, payload: Vec<u8>) -> Self {
        PendingCall {
            env,
            destination,
            params: None,
            payload: Some(payload),
            _output: PhantomData,
            state: None,
        }
    }

    pub fn with_params(mut self, f: impl FnOnce(E::Params) -> E::Params) -> Self {
        self.params = Some(f(self.params.unwrap_or_default()));
        self
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCtor<E: GearEnv, A, T: ActionIo> {
        env: E,
        code_id: CodeId,
        params: Option<E::Params>,
        salt: Option<Vec<u8>>,
        args: Option<T::Params>,
        _actor: PhantomData<A>,
        #[pin]
        state: Option<E::MessageState>,
        program_id: Option<ActorId>,
    }
}

impl<E: GearEnv, A, T: ActionIo> PendingCtor<E, A, T> {
    pub fn new(env: E, code_id: CodeId, salt: Vec<u8>, args: T::Params) -> Self {
        PendingCtor {
            env,
            code_id,
            params: None,
            salt: Some(salt),
            args: Some(args),
            _actor: PhantomData,
            state: None,
            program_id: None,
        }
    }

    pub fn with_params(mut self, f: impl FnOnce(E::Params) -> E::Params) -> Self {
        self.params = Some(f(self.params.unwrap_or_default()));
        self
    }
}

#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
pub mod gclient {
    use super::*;
    use ::gclient::{Error, EventListener, EventProcessor as _, GearApi};

    #[derive(Debug, Default)]
    pub struct GclientParams {
        gas_limit: Option<GasUnit>,
        value: Option<ValueUnit>,
        at_block: Option<H256>,
    }

    #[derive(Clone)]
    pub struct GclientEnv {
        api: GearApi,
    }

    impl GclientEnv {
        pub fn new(api: GearApi) -> Self {
            Self { api }
        }

        pub fn with_suri(self, suri: impl AsRef<str>) -> Self {
            let api = self.api.with(suri).unwrap();
            Self { api }
        }

        async fn query_calculate_reply(
            self,
            target: ActorId,
            payload: impl AsRef<[u8]>,
            params: GclientParams,
        ) -> Result<Vec<u8>, Error> {
            let api = self.api;

            // Get Max gas amount if it is not explicitly set
            #[cfg(not(feature = "ethexe"))]
            let gas_limit = if let Some(gas_limit) = params.gas_limit {
                gas_limit
            } else {
                api.block_gas_limit()?
            };
            #[cfg(feature = "ethexe")]
            let gas_limit = 0;
            let value = params.value.unwrap_or(0);
            let origin = H256::from_slice(api.account_id().as_ref());
            let payload = payload.as_ref().to_vec();

            let reply_info = api
                .calculate_reply_for_handle_at(
                    Some(origin),
                    target,
                    payload,
                    gas_limit,
                    value,
                    params.at_block,
                )
                .await?;

            match reply_info.code {
                ReplyCode::Success(_) => Ok(reply_info.payload),
                // TODO
                ReplyCode::Error(_reason) => Err(Error::EventNotFound),
                ReplyCode::Unsupported => Err(Error::EventNotFound),
            }
        }
    }

    impl GearEnv for GclientEnv {
        type Params = GclientParams;
        type Error = Error;
        type MessageState = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>>;
    }

    async fn send_message(
        api: GearApi,
        target: ActorId,
        payload: Vec<u8>,
        params: GclientParams,
    ) -> Result<Vec<u8>, Error> {
        let value = params.value.unwrap_or(0);
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = if let Some(gas_limit) = params.gas_limit {
            gas_limit
        } else {
            // Calculate gas amount needed for handling the message
            let gas_info = api
                .calculate_handle_gas(None, target, payload.clone(), value, true)
                .await?;
            gas_info.min_limit
        };
        #[cfg(feature = "ethexe")]
        let gas_limit = 0;

        let mut listener = api.subscribe().await?;
        let (message_id, ..) = api
            .send_message_bytes(target, payload, gas_limit, value)
            .await?;
        let (_, reply_code, payload, _) = wait_for_reply(&mut listener, message_id).await?;
        // TODO handle errors
        match reply_code {
            ReplyCode::Success(_) => Ok(payload),
            ReplyCode::Error(error_reply_reason) => todo!(),
            ReplyCode::Unsupported => todo!(),
        }
    }

    async fn wait_for_reply(
        listener: &mut EventListener,
        message_id: MessageId,
    ) -> Result<(MessageId, ReplyCode, Vec<u8>, ValueUnit), Error> {
        let message_id: ::gclient::metadata::runtime_types::gprimitives::MessageId =
            message_id.into();
        listener.proc(|e| {
            if let ::gclient::Event::Gear(::gclient::GearEvent::UserMessageSent {
                message:
                    ::gclient::metadata::runtime_types::gear_core::message::user::UserMessage {
                        id,
                        payload,
                        value,
                        details: Some(::gclient::metadata::runtime_types::gear_core::message::common::ReplyDetails { to, code }),
                        ..
                    },
                ..
            }) = e
            {
                to.eq(&message_id).then(|| {
                    let reply_code = ReplyCode::from(code);

                    (id.into(), reply_code, payload.0.clone(), value)
                })
            } else {
                None
            }
        })
        .await
    }
    impl<O: Decode> PendingCall<GclientEnv, O> {
        async fn send(self) -> Result<MessageId, Error> {
            let api = &self.env.api;
            let params = self.params.unwrap_or_default();
            let payload = self.payload.unwrap_or_default();
            let value = params.value.unwrap_or(0);
            #[cfg(not(feature = "ethexe"))]
            let gas_limit = if let Some(gas_limit) = params.gas_limit {
                gas_limit
            } else {
                // Calculate gas amount needed for handling the message
                let gas_info = api
                    .calculate_handle_gas(None, self.destination, payload.clone(), value, true)
                    .await?;
                gas_info.min_limit
            };
            #[cfg(feature = "ethexe")]
            let gas_limit = 0;

            let (message_id, ..) = api
                .send_message_bytes(self.destination, payload, gas_limit, value)
                .await?;
            Ok(message_id)
        }

        async fn query(self) -> Result<O, Error> {
            let params = self.params.unwrap_or_default();
            let payload = self.payload.unwrap_or_default();

            // Calculate reply
            let reply_bytes = self
                .env
                .query_calculate_reply(self.destination, payload, params)
                .await?;

            // Decode reply
            match O::decode(&mut reply_bytes.as_slice()) {
                Ok(decoded) => Ok(decoded),
                Err(err) => Err(Error::Codec(err)),
            }
        }
    }

    impl<O: Decode> Future for PendingCall<GclientEnv, O> {
        type Output = Result<O, <GclientEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.state.is_none() {
                // Send message
                let params = self.params.take().unwrap_or_default();
                let payload = self.payload.take().unwrap_or_default();
                let send_future =
                    send_message(self.env.api.clone(), self.destination, payload, params);
                self.state = Some(Box::pin(send_future));
            }
            if let Some(message_future) = self.project().state.as_pin_mut() {
                // Poll message future
                match message_future.poll(cx) {
                    Poll::Ready(Ok(bytes)) => match O::decode(&mut bytes.as_slice()) {
                        Ok(decoded) => Poll::Ready(Ok(decoded)),
                        Err(err) => Poll::Ready(Err(Error::Codec(err))),
                    },
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            } else {
                panic!("PendingCall polled after completion or invalid state");
            }
        }
    }
}

mod gstd {
    use super::*;
    use ::gstd::errors::Error;
    use ::gstd::msg;
    use ::gstd::msg::MessageFuture;

    #[derive(Default)]
    pub struct GstdParams {
        #[cfg(not(feature = "ethexe"))]
        gas_limit: Option<GasUnit>,
        value: Option<ValueUnit>,
        wait_up_to: Option<BlockCount>,
        #[cfg(not(feature = "ethexe"))]
        reply_deposit: Option<GasUnit>,
        #[cfg(not(feature = "ethexe"))]
        reply_hook: Option<Box<dyn FnOnce() + Send + 'static>>,
        redirect_on_exit: bool,
    }

    impl GstdParams {
        pub fn with_wait_up_to(self, wait_up_to: Option<BlockCount>) -> Self {
            Self { wait_up_to, ..self }
        }

        pub fn with_redirect_on_exit(self, redirect_on_exit: bool) -> Self {
            Self {
                redirect_on_exit,
                ..self
            }
        }

        pub fn wait_up_to(&self) -> Option<BlockCount> {
            self.wait_up_to
        }

        pub fn redirect_on_exit(&self) -> bool {
            self.redirect_on_exit
        }
    }

    #[cfg(not(feature = "ethexe"))]
    impl GstdParams {
        pub fn with_reply_deposit(self, reply_deposit: Option<GasUnit>) -> Self {
            Self {
                reply_deposit,
                ..self
            }
        }

        pub fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self {
            Self {
                reply_hook: Some(Box::new(f)),
                ..self
            }
        }

        pub fn reply_deposit(&self) -> Option<GasUnit> {
            self.reply_deposit
        }
    }

    #[derive(Debug, Default, Clone)]
    pub struct GstdEnv;

    impl GearEnv for GstdEnv {
        type Params = GstdParams;
        type Error = Error;
        type MessageState = MessageFuture;
    }

    #[cfg(not(feature = "ethexe"))]
    pub(crate) fn send_for_reply_future(
        target: ActorId,
        payload: &[u8],
        params: GstdParams,
    ) -> Result<MessageFuture, Error> {
        let value = params.value.unwrap_or(0);
        // here can be a redirect target
        let mut message_future = if let Some(gas_limit) = params.gas_limit {
            msg::send_bytes_with_gas_for_reply(
                target,
                payload,
                gas_limit,
                value,
                params.reply_deposit.unwrap_or_default(),
            )?
        } else {
            msg::send_bytes_for_reply(
                target,
                payload,
                value,
                params.reply_deposit.unwrap_or_default(),
            )?
        };

        message_future = message_future.up_to(params.wait_up_to)?;

        if let Some(reply_hook) = params.reply_hook {
            message_future = message_future.handle_reply(reply_hook)?;
        }
        Ok(message_future)
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn send_for_reply_future(
        target: ActorId,
        payload: &[u8],
        args: GstdParams,
    ) -> Result<msg::MessageFuture> {
        let value = params.value.unwrap_or(0);
        // here can be a redirect target
        let mut message_future = msg::send_bytes_for_reply(target, payload, value)?;

        message_future = message_future.up_to(params.wait_up_to)?;

        Ok(message_future)
    }

    impl<O: Decode> PendingCall<GstdEnv, O> {
        pub fn send(self) -> Result<MessageId, Error> {
            let params = self.params.unwrap_or_default();
            let payload = self.payload.unwrap_or_default();
            let value = params.value.unwrap_or(0);
            if let Some(gas_limit) = params.gas_limit {
                ::gcore::msg::send_with_gas(self.destination, payload.as_slice(), gas_limit, value)
                    .map_err(|err| Error::Core(err))
            } else {
                ::gcore::msg::send(self.destination, payload.as_slice(), value)
                    .map_err(|err| Error::Core(err))
            }
        }
    }

    impl<O: Decode> Future for PendingCall<GstdEnv, O> {
        type Output = Result<O, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.state.is_none() {
                // Send message
                let params = self.params.take().unwrap_or_default();
                let payload = self.payload.take().unwrap_or_default();
                let send_res = send_for_reply_future(self.destination, payload.as_slice(), params);
                match send_res {
                    Ok(message_fut) => {
                        self.state = Some(message_fut);
                    }
                    Err(err) => {
                        return Poll::Ready(Err(err));
                    }
                }
            }
            if let Some(message_fut) = self.project().state.as_pin_mut() {
                // Poll message future
                match message_fut.poll(cx) {
                    Poll::Ready(Ok(bytes)) => match O::decode(&mut bytes.as_slice()) {
                        Ok(decoded) => Poll::Ready(Ok(decoded)),
                        Err(err) => Poll::Ready(Err(Error::Decode(err))),
                    },
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            } else {
                panic!("PendingCall polled after completion or invalid state");
            }
        }
    }
}

// mod client {

//     use super::service::Service;
//     use super::*;

//     pub struct MyServiceImpl;

//     pub trait MyService<E: GearEnv> {
//         fn mint(&mut self, to: ActorId, amount: u128) -> PendingCall<E, bool>;
//         fn burn(&mut self, from: ActorId) -> PendingCall<E, u8>;
//         fn total(&self) -> PendingCall<E, u128>;
//     }

//     impl<E: GearEnv> MyService<E> for Service<MyServiceImpl, E> {
//         fn mint(&mut self, to: ActorId, amount: u128) -> PendingCall<E, bool> {
//             self.pending_call("Mint", (to, amount))
//         }

//         fn burn(&mut self, from: ActorId) -> PendingCall<E, u8> {
//             self.pending_call("Burn", (from,))
//         }

//         fn total(&self) -> PendingCall<E, u128> {
//             self.pending_call("Total", ())
//         }
//     }

//     #[cfg(feature = "mockall")]
//     #[cfg(not(target_arch = "wasm32"))]
//     mockall::mock! {
//         pub MyService<E: GearEnv> {}

//         impl<E: GearEnv> MyService<E> for MyService<E> {
//             fn mint(&mut self, to: ActorId, amount: u128) -> PendingCall<E, bool>;
//             fn burn(&mut self, from: ActorId) -> PendingCall<E, u8>;
//             fn total(&self) -> PendingCall<E, u128>;
//         }
//     }
// }

// #[cfg(feature = "mockall")]
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn sample() -> Result<(), Box<dyn Error>> {
//         use client::*;

//         let mut my_service = MockMyService::new();
//         my_service.expect_total().returning(move || 137.into());
//         my_service.expect_mint().returning(move |_, _| true.into());

//         assert_eq!(my_service.total().await?, 137);

//         let mut my_service = my_service;

//         assert!(my_service.mint(ActorId::from(137), 1_000).await?);

//         Ok(())
//     }
// }
