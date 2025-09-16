use super::*;
use ::gstd::{
    errors::Error,
    msg::{CreateProgramFuture, MessageFuture},
};

#[derive(Default)]
pub struct GstdParams {
    #[cfg(not(feature = "ethexe"))]
    pub gas_limit: Option<GasUnit>,
    pub value: Option<ValueUnit>,
    pub wait_up_to: Option<BlockCount>,
    #[cfg(not(feature = "ethexe"))]
    pub reply_deposit: Option<GasUnit>,
    #[cfg(not(feature = "ethexe"))]
    pub reply_hook: Option<Box<dyn FnOnce() + Send + 'static>>,
    pub redirect_on_exit: bool,
}

crate::params_for_pending_impl!(GstdEnv, GstdParams {
    #[cfg(not(feature = "ethexe"))]
    pub gas_limit: GasUnit,
    pub value: ValueUnit,
    pub wait_up_to: BlockCount,
    #[cfg(not(feature = "ethexe"))]
    pub reply_deposit: GasUnit,
});

impl GstdParams {
    pub fn with_redirect_on_exit(self, redirect_on_exit: bool) -> Self {
        Self {
            redirect_on_exit,
            ..self
        }
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self {
        Self {
            reply_hook: Some(Box::new(f)),
            ..self
        }
    }
}

impl<T: CallEncodeDecode> PendingCall<GstdEnv, T> {
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

    #[cfg(not(feature = "ethexe"))]
    pub fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self {
        self.with_params(|params| params.with_reply_hook(f))
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

impl GstdEnv {
    pub fn send_one_way(
        &self,
        destination: ActorId,
        payload: impl AsRef<[u8]>,
        params: GstdParams,
    ) -> Result<MessageId, Error> {
        let value = params.value.unwrap_or(0);

        #[cfg(not(feature = "ethexe"))]
        if let Some(gas_limit) = params.gas_limit {
            return ::gcore::msg::send_with_gas(destination, payload.as_ref(), gas_limit, value)
                .map_err(Error::Core);
        }

        ::gcore::msg::send(destination, payload.as_ref(), value).map_err(Error::Core)
    }
}

impl<T: CallEncodeDecode> PendingCall<GstdEnv, T> {
    pub fn send_one_way(&mut self) -> Result<MessageId, Error> {
        let (payload, params) = self.take_encoded_args_and_params();
        self.env.send_one_way(self.destination, payload, params)
    }
}

#[cfg(target_arch = "wasm32")]
const _: () = {
    use core::task::ready;

    #[cfg(not(feature = "ethexe"))]
    #[inline]
    fn send_for_reply_future(
        destination: ActorId,
        payload: &[u8],
        params: &mut GstdParams,
    ) -> Result<MessageFuture, Error> {
        let value = params.value.unwrap_or(0);
        // here can be a redirect target
        let mut message_future = if let Some(gas_limit) = params.gas_limit {
            ::gstd::msg::send_bytes_with_gas_for_reply(
                destination,
                payload,
                gas_limit,
                value,
                params.reply_deposit.unwrap_or_default(),
            )?
        } else {
            ::gstd::msg::send_bytes_for_reply(
                destination,
                payload,
                value,
                params.reply_deposit.unwrap_or_default(),
            )?
        };

        message_future = message_future.up_to(params.wait_up_to)?;

        if let Some(reply_hook) = params.reply_hook.take() {
            message_future = message_future.handle_reply(reply_hook)?;
        }
        Ok(message_future)
    }

    #[cfg(feature = "ethexe")]
    #[inline]
    fn send_for_reply_future(
        destination: ActorId,
        payload: &[u8],
        params: &mut GstdParams,
    ) -> Result<MessageFuture, Error> {
        let value = params.value.unwrap_or(0);
        // here can be a redirect target
        let mut message_future = ::gstd::msg::send_bytes_for_reply(destination, payload, value)?;

        message_future = message_future.up_to(params.wait_up_to)?;

        Ok(message_future)
    }

    #[inline]
    fn send_for_reply(
        destination: ActorId,
        payload: &[u8],
        params: &mut GstdParams,
    ) -> Result<GtsdFuture, Error> {
        // send message
        let future = send_for_reply_future(destination, payload, params)?;
        if params.redirect_on_exit {
            let created_block = params.wait_up_to.map(|_| gstd::exec::block_height());
            Ok(GtsdFuture::MessageWithRedirect {
                created_block,
                future,
                destination,
            })
        } else {
            Ok(GtsdFuture::Message { future })
        }
    }

    impl<T: CallEncodeDecode> Future for PendingCall<GstdEnv, T> {
        type Output = Result<T::Reply, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.state.is_none() {
                let args = self
                    .args
                    .as_ref()
                    .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
                let payload = T::encode_params_with_prefix(self.route, args);
                let destination = self.destination;
                let params = self.params.get_or_insert_default();
                // Send message
                let send_res = send_for_reply(destination, payload.as_slice(), params);
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
            let Some(mut state) = this.state.as_pin_mut() else {
                panic!("{PENDING_CALL_INVALID_STATE}");
            };
            // Poll message future
            let output = match state.as_mut().project() {
                Projection::Message { future } => ready!(future.poll(cx)),
                Projection::MessageWithRedirect { future, .. } => ready!(future.poll(cx)),
                _ => panic!("{PENDING_CALL_INVALID_STATE}"),
            };
            match output {
                // ok reply
                Ok(payload) => match T::decode_reply_with_prefix(self.route, payload) {
                    Ok(reply) => Poll::Ready(Ok(reply)),
                    Err(err) => Poll::Ready(Err(Error::Decode(err))),
                },
                // reply with ProgramExited
                Err(gstd::errors::Error::ErrorReply(
                    error_payload,
                    ErrorReplyReason::UnavailableActor(SimpleUnavailableActorError::ProgramExited),
                )) => {
                    let params = this.params.get_or_insert_default();
                    if let Replace::MessageWithRedirect {
                        destination: _destination,
                        created_block,
                        ..
                    } = state.as_mut().project_replace(GtsdFuture::Dummy)
                        && params.redirect_on_exit
                        && let Ok(new_target) = ActorId::try_from(error_payload.0.as_ref())
                    {
                        gstd::debug!("Redirecting message from {_destination} to {new_target}");

                        // Calculate updated `wait_up_to` if provided
                        // wait_up_to = wait_up_to - (current_block - created_block)
                        params.wait_up_to = params.wait_up_to.and_then(|wait_up_to| {
                            created_block.map(|created_block| {
                                let current_block = gstd::exec::block_height();
                                wait_up_to
                                    .saturating_sub(current_block.saturating_sub(created_block))
                            })
                        });

                        let args = this
                            .args
                            .as_ref()
                            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
                        let payload = T::encode_params_with_prefix(this.route, args);
                        // send message to new target
                        let send_res = send_for_reply(new_target, payload.as_slice(), params);
                        match send_res {
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
        }
    }

    impl<A, T: CallEncodeDecode> Future for PendingCtor<GstdEnv, A, T> {
        type Output = Result<Actor<GstdEnv, A>, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.state.is_none() {
                let params = self.params.take().unwrap_or_default();
                let value = params.value.unwrap_or(0);
                let salt = self.salt.take().unwrap();

                let args = self
                    .args
                    .take()
                    .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
                let payload = T::encode_params(&args);
                // Send message
                #[cfg(not(feature = "ethexe"))]
                let program_future = if let Some(gas_limit) = params.gas_limit {
                    ::gstd::prog::create_program_bytes_with_gas_for_reply(
                        self.code_id,
                        salt,
                        payload,
                        gas_limit,
                        value,
                        params.reply_deposit.unwrap_or_default(),
                    )?
                } else {
                    ::gstd::prog::create_program_bytes_for_reply(
                        self.code_id,
                        salt,
                        payload,
                        value,
                        params.reply_deposit.unwrap_or_default(),
                    )?
                };
                #[cfg(feature = "ethexe")]
                let program_future = ::gstd::prog::create_program_bytes_for_reply(
                    self.code_id,
                    salt,
                    payload,
                    value,
                )?;

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
                match ready!(future.poll(cx)) {
                    Ok((program_id, _payload)) => {
                        // Do not decode payload here
                        Poll::Ready(Ok(Actor::new(this.env.clone(), program_id)))
                    }
                    Err(err) => Poll::Ready(Err(err)),
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
            destination: ActorId,
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
                route: "",
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
