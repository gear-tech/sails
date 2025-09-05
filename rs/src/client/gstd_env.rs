use super::*;
use ::gstd::{
    errors::Error,
    msg,
    msg::{CreateProgramFuture, MessageFuture},
};
use core::task::ready;

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

impl Clone for GstdParams {
    fn clone(&self) -> Self {
        Self {
            gas_limit: self.gas_limit.clone(),
            value: self.value.clone(),
            wait_up_to: self.wait_up_to.clone(),
            reply_deposit: self.reply_deposit.clone(),
            reply_hook: None,
            redirect_on_exit: self.redirect_on_exit.clone(),
        }
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

impl<T: CallEncodeDecode> PendingCall<GstdEnv, T> {
    pub fn with_wait_up_to(self, wait_up_to: Option<BlockCount>) -> Self {
        self.with_params(|params| params.with_wait_up_to(wait_up_to))
    }

    /// Set `redirect_on_exit` flag to `true``
    ///
    /// This flag is used to redirect a message to a new program when the target program exits
    /// with an inheritor.
    ///
    /// WARNING: When this flag is set, the message future captures the payload and other arguments,
    /// potentially resulting in multiple messages being sent. This can lead to increased gas consumption.
    ///
    /// This flag is set to `false`` by default.
    pub fn with_redirect_on_exit(self, redirect_on_exit: bool) -> Self {
        self.with_params(|params| params.with_redirect_on_exit(redirect_on_exit))
    }
}

#[derive(Debug, Default, Clone)]
pub struct GstdEnv;

impl GearEnv for GstdEnv {
    type Params = GstdParams;
    type Error = Error;
    #[cfg(target_arch = "wasm32")]
    type MessageState = GtsdFuture;
    #[cfg(not(target_arch = "wasm32"))]
    type MessageState = core::future::Ready<Result<Vec<u8>, Self::Error>>;
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
    params: GstdParams,
) -> Result<msg::MessageFuture, Error> {
    let value = params.value.unwrap_or(0);
    // here can be a redirect target
    let mut message_future = msg::send_bytes_for_reply(target, payload, value)?;

    message_future = message_future.up_to(params.wait_up_to)?;

    Ok(message_future)
}

pub(crate) fn send_for_reply<T: AsRef<[u8]>>(
    target: ActorId,
    payload: T,
    params: GstdParams,
) -> Result<GtsdFuture, Error> {
    let redirect_on_exit = params.redirect_on_exit;
    // clone params w/o reply_hook
    let params_cloned = params.clone();
    // send message
    let future = send_for_reply_future(target, payload.as_ref(), params)?;
    if redirect_on_exit {
        let created_block = params_cloned.wait_up_to.map(|_| gstd::exec::block_height());
        Ok(GtsdFuture::MessageWithRedirect {
            created_block,
            future,
            params: params_cloned,
            payload: payload.as_ref().to_vec(),
            target: target,
        })
    } else {
        Ok(GtsdFuture::Message { future })
    }
}

#[cfg(target_arch = "wasm32")]
const _: () = {
    impl<T: CallEncodeDecode> PendingCall<GstdEnv, T> {
        pub fn send(mut self) -> Result<MessageId, Error> {
            let route = self
                .route
                .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
            let params = self.params.unwrap_or_default();
            let args = self
                .args
                .take()
                .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
            let payload = T::encode_params_with_prefix(route, &args);

            let value = params.value.unwrap_or(0);

            #[cfg(feature = "ethexe")]
            {
                ::gcore::msg::send(self.destination, payload.as_slice(), value).map_err(Error::Core)
            }

            #[cfg(not(feature = "ethexe"))]
            if let Some(gas_limit) = params.gas_limit {
                ::gcore::msg::send_with_gas(self.destination, payload.as_slice(), gas_limit, value)
                    .map_err(Error::Core)
            } else {
                ::gcore::msg::send(self.destination, payload.as_slice(), value).map_err(Error::Core)
            }
        }
    }

    impl<T: CallEncodeDecode> Future for PendingCall<GstdEnv, T> {
        type Output = Result<T::Reply, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let Some(route) = self.route else {
                return Poll::Ready(Err(Error::Decode("PendingCall route is not set".into())));
            };
            if self.state.is_none() {
                // Send message
                let args = self
                    .args
                    .take()
                    .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
                let payload = T::encode_params_with_prefix(route, &args);
                let params = self.params.take().unwrap_or_default();

                let send_res = send_for_reply(self.destination, payload, params);
                match send_res {
                    Ok(future) => {
                        self.state = Some(future);
                    }
                    Err(err) => {
                        return Poll::Ready(Err(err));
                    }
                }
            }
            let this = self.as_mut().project();
            if let Some(mut state) = this.state.as_pin_mut() {
                // Poll message future
                let output = match state.as_mut().project() {
                    Projection::CreateProgram { .. } => panic!("{PENDING_CALL_INVALID_STATE}"),
                    Projection::Message { future } => ready!(future.poll(cx)),
                    Projection::MessageWithRedirect { future, .. } => ready!(future.poll(cx)),
                    Projection::Dummy => panic!("{PENDING_CALL_INVALID_STATE}"),
                };
                match output {
                    // ok reply
                    Ok(payload) => match T::decode_reply_with_prefix(route, payload) {
                        Ok(reply) => Poll::Ready(Ok(reply)),
                        Err(err) => Poll::Ready(Err(Error::Decode(err))),
                    },
                    // reply with ProgramExited
                    Err(gstd::errors::Error::ErrorReply(
                        error_payload,
                        ErrorReplyReason::UnavailableActor(
                            SimpleUnavailableActorError::ProgramExited,
                        ),
                    )) => {
                        if let Replace::MessageWithRedirect {
                            target: _target,
                            payload,
                            mut params,
                            created_block,
                            ..
                        } = state.as_mut().project_replace(GtsdFuture::Dummy)
                            && params.redirect_on_exit
                            && let Ok(new_target) = ActorId::try_from(error_payload.0.as_ref())
                        {
                            gstd::debug!("Redirecting message from {_target} to {new_target}");

                            // Calculate updated `wait_up_to` if provided
                            // wait_up_to = wait_up_to - (current_block - created_block)
                            params.wait_up_to = params.wait_up_to.and_then(|wait_up_to| {
                                created_block.map(|created_block| {
                                    let current_block = gstd::exec::block_height();
                                    wait_up_to
                                        .saturating_sub(current_block.saturating_sub(created_block))
                                })
                            });

                            // send message to new target
                            let future_res = send_for_reply(new_target, payload, params);
                            match future_res {
                                Ok(future) => {
                                    // Replace the future with a new one
                                    _ = state.as_mut().project_replace(future);
                                    // Return Pending to allow the new future to be polled
                                    Poll::Pending
                                }
                                Err(err) => Poll::Ready(Err(err)),
                            }
                        } else {
                            Poll::Ready(Err(gstd::errors::Error::ErrorReply(
                                error_payload,
                                ErrorReplyReason::UnavailableActor(
                                    SimpleUnavailableActorError::ProgramExited,
                                ),
                            )
                            .into()))
                        }
                    }
                    // error reply
                    Err(err) => Poll::Ready(Err(err)),
                }
            } else {
                panic!("{PENDING_CALL_INVALID_STATE}");
            }
        }
    }

    impl<A, T: CallEncodeDecode> Future for PendingCtor<GstdEnv, A, T> {
        type Output = Result<Actor<GstdEnv, A>, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.state.is_none() {
                // Send message
                let payload = self.encode_ctor();
                let params = self.params.take().unwrap_or_default();
                let value = params.value.unwrap_or(0);
                let salt = self.salt.take().unwrap();

                #[cfg(not(feature = "ethexe"))]
                let program_future = if let Some(gas_limit) = params.gas_limit {
                    gstd::prog::create_program_bytes_with_gas_for_reply(
                        self.code_id,
                        salt,
                        payload,
                        gas_limit,
                        value,
                        params.reply_deposit.unwrap_or_default(),
                    )?
                } else {
                    gstd::prog::create_program_bytes_for_reply(
                        self.code_id,
                        salt,
                        payload,
                        value,
                        params.reply_deposit.unwrap_or_default(),
                    )?
                };
                #[cfg(feature = "ethexe")]
                let mut program_future =
                    gstd::prog::create_program_bytes_for_reply(self.code_id, salt, payload, value)?;

                // self.program_id = Some(program_future.program_id);
                self.state = Some(GtsdFuture::CreateProgram {
                    future: program_future,
                });
            }
            let this = self.as_mut().project();
            if let Some(state) = this.state.as_pin_mut()
                && let Projection::CreateProgram { future } = state.project()
            {
                // Poll create program future
                match future.poll(cx) {
                    Poll::Ready(Ok((program_id, _payload))) => {
                        // Do not decode payload here
                        Poll::Ready(Ok(Actor::new(this.env.clone(), program_id)))
                    }
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            } else {
                panic!("{PENDING_CTOR_INVALID_STATE}");
            }
        }
    }
};

pin_project_lite::pin_project! {
    #[project = Projection]
    #[project_replace = Replace]
    pub enum GtsdFuture {
        CreateProgram { #[pin] future: CreateProgramFuture },
        Message { #[pin] future: MessageFuture },
        MessageWithRedirect {
            #[pin]
            future: MessageFuture,
            target: ActorId,
            payload: Vec<u8>,
            params: GstdParams,
            created_block: Option<BlockCount>,
        },
        Dummy,
    }
}

#[cfg(not(target_arch = "wasm32"))]
const _: () = {
    impl<T: CallEncodeDecode> PendingCall<GstdEnv, T>
    where
        T::Reply: Encode + Decode,
    {
        pub fn from_output(output: T::Reply) -> Self {
            Self::from_result(Ok(output))
        }

        pub fn from_error(err: <GstdEnv as GearEnv>::Error) -> Self {
            Self::from_result(Err(err))
        }

        pub fn from_result(res: Result<T::Reply, <GstdEnv as GearEnv>::Error>) -> Self {
            PendingCall {
                env: GstdEnv,
                destination: ActorId::zero(),
                route: None,
                params: None,
                args: None,
                state: Some(future::ready(res.map(|v| v.encode()))),
            }
        }
    }

    impl<T: CallEncodeDecode<Reply = O>, O> From<O> for PendingCall<GstdEnv, T>
    where
        O: Encode + Decode,
    {
        fn from(value: O) -> Self {
            PendingCall::from_output(value)
        }
    }

    impl<T: CallEncodeDecode> Future for PendingCall<GstdEnv, T> {
        type Output = Result<T::Reply, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            match self.state.take() {
                Some(ready) => {
                    let res = ready.into_inner();
                    Poll::Ready(res.map(|v| T::Reply::decode(&mut v.as_slice()).unwrap()))
                }
                None => panic!("{PENDING_CALL_INVALID_STATE}"),
            }
        }
    }

    impl<A, T: CallEncodeDecode> Future for PendingCtor<GstdEnv, A, T> {
        type Output = Result<Actor<GstdEnv, A>, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            match self.state.take() {
                Some(_ready) => {
                    let program_id = self
                        .program_id
                        .take()
                        .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
                    let env = self.env.clone();
                    Poll::Ready(Ok(Actor::new(env, program_id)))
                }
                None => panic!("{PENDING_CTOR_INVALID_STATE}"),
            }
        }
    }
};
