use super::*;
use crate::gstd::{MessageFuture, async_runtime, locks};
use ::gstd::errors::Error;

#[derive(Default)]
pub struct GstdParams {
    #[cfg(not(feature = "ethexe"))]
    pub gas_limit: Option<GasUnit>,
    pub value: Option<ValueUnit>,
    pub wait: Option<locks::Lock>,
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
    pub wait: locks::Lock,
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
    type MessageState = GstdFuture;
    #[cfg(not(target_arch = "wasm32"))]
    type MessageState = core::future::Ready<Result<Vec<u8>, Self::Error>>;
}

impl GstdEnv {
    pub fn send_one_way(
        &self,
        destination: ActorId,
        payload: impl AsRef<[u8]>,
        mut params: GstdParams,
    ) -> Result<MessageId, Error> {
        let message = crate::ok!(async_runtime::send_bytes_for_reply(
            destination,
            payload.as_ref(),
            params.value.unwrap_or_default(),
            params.wait.unwrap_or_default(),
            params.gas_limit,
            params.reply_deposit,
            // params.reply_hook.take(),
        ));

        Ok(message.waiting_reply_to)
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
    use futures::future::FusedFuture;

    #[inline]
    fn send_for_reply(
        destination: ActorId,
        payload: Vec<u8>,
        params: &mut GstdParams,
    ) -> Result<GstdFuture, Error> {
        // send message
        // let future = send_for_reply_future(destination, payload.as_ref(), params)?;
        let future = crate::ok!(crate::gstd::send_bytes_for_reply(
            destination,
            payload.as_ref(),
            params.value.unwrap_or_default(),
            params.wait.unwrap_or_default(),
            params.gas_limit,
            params.reply_deposit,
            // params.reply_hook.take(),
        ));
        if params.redirect_on_exit {
            Ok(GstdFuture::MessageWithRedirect {
                future,
                destination,
                payload,
            })
        } else {
            Ok(GstdFuture::Message { future })
        }
    }

    fn create_program(
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        params: &mut GstdParams,
    ) -> Result<(GstdFuture, ActorId), Error> {
        let (future, program_id) = crate::ok!(crate::gstd::create_program_for_reply(
            code_id,
            salt.as_ref(),
            payload.as_ref(),
            params.value.unwrap_or_default(),
            params.wait.unwrap_or_default(),
            params.gas_limit,
            params.reply_deposit,
            // params.reply_hook.take(),
        ));
        Ok((GstdFuture::CreateProgram { future }, program_id))
    }

    impl<T: CallEncodeDecode> Future for PendingCall<GstdEnv, T> {
        type Output = Result<T::Reply, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.state.is_none() {
                let args = self
                    .args
                    .as_ref()
                    .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
                let payload = T::encode_params_with_prefix(self.route, &args);
                let destination = self.destination;
                let params = self.params.get_or_insert_default();
                // Send message
                let future = match send_for_reply(destination, payload, params) {
                    Ok(future) => future,
                    Err(err) => return Poll::Ready(Err(err)),
                };
                self.state = Some(future);
                // No need to poll the future
                return Poll::Pending;
            }
            let this = self.as_mut().project();
            // SAFETY: checked in the code above.
            let mut state = unsafe { this.state.as_pin_mut().unwrap_unchecked() };
            // Poll message future
            let output = match state.as_mut().project() {
                Projection::Message { future } => ready!(future.poll(cx)),
                Projection::MessageWithRedirect { future, .. } => ready!(future.poll(cx)),
                _ => panic!("{PENDING_CALL_INVALID_STATE}"),
            };
            match output {
                // ok reply
                Ok(payload) => {
                    let res =
                        T::decode_reply_with_prefix(self.route, payload).map_err(Error::Decode)?;
                    Poll::Ready(Ok(res))
                }
                // reply with ProgramExited
                Err(gstd::errors::Error::ErrorReply(
                    error_payload,
                    ErrorReplyReason::UnavailableActor(SimpleUnavailableActorError::ProgramExited),
                )) => {
                    let params = this.params.get_or_insert_default();
                    if let Replace::MessageWithRedirect {
                        destination: _destination,
                        payload,
                        ..
                    } = state.as_mut().project_replace(GstdFuture::Dummy)
                        && params.redirect_on_exit
                        && let Ok(new_target) = ActorId::try_from(error_payload.0.as_ref())
                    {
                        gstd::debug!("Redirecting message from {_destination} to {new_target}");

                        // send message to new target
                        let future = send_for_reply(new_target, payload, params)?;
                        // Replace the future with a new one
                        _ = state.as_mut().project_replace(future);
                        // Return Pending to allow the new future to be polled
                        Poll::Pending
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

    impl<T: CallEncodeDecode> FusedFuture for PendingCall<GstdEnv, T> {
        fn is_terminated(&self) -> bool {
            self.state
                .as_ref()
                .map(|future| match future {
                    GstdFuture::CreateProgram { future } => future.is_terminated(),
                    GstdFuture::Message { future } => future.is_terminated(),
                    GstdFuture::MessageWithRedirect { future, .. } => future.is_terminated(),
                    GstdFuture::Dummy => false,
                })
                .unwrap_or_default()
        }
    }

    impl<A, T: CallEncodeDecode> Future for PendingCtor<GstdEnv, A, T> {
        type Output = Result<Actor<GstdEnv, A>, <GstdEnv as GearEnv>::Error>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.state.is_none() {
                let mut params = self.params.take().unwrap_or_default();
                let value = params.value.unwrap_or_default();
                let salt = self.salt.take().unwrap();

                let args = self
                    .args
                    .as_ref()
                    .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
                let payload = T::encode_params(args);
                // Send message
                let (future, program_id) =
                    match create_program(self.code_id, salt, payload, &mut params) {
                        Ok(res) => res,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                self.program_id = Some(program_id);
                self.state = Some(future);
                // No need to poll the future
                return Poll::Pending;
            }
            let this = self.as_mut().project();
            // SAFETY: checked in the code above.
            let state = unsafe { this.state.as_pin_mut().unwrap_unchecked() };
            if let Projection::CreateProgram { future } = state.project() {
                // Poll create program future
                match ready!(future.poll(cx)) {
                    Ok(_payload) => {
                        let program_id = unsafe { this.program_id.unwrap_unchecked() };
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
    pub enum GstdFuture {
        CreateProgram { #[pin] future: MessageFuture },
        Message { #[pin] future: MessageFuture },
        MessageWithRedirect {
            #[pin]
            future: MessageFuture,
            destination: ActorId,
            payload: Vec<u8>, // reuse encoded payload when redirecting
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
                    Poll::Ready(res.map(|v| T::Reply::decode(&mut v.as_ref()).unwrap()))
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
