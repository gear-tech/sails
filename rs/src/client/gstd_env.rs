use super::*;
use ::gstd::{errors::Error, msg, msg::MessageFuture};

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

impl<T: CallEncodeDecode> PendingCall<GstdEnv, T> {
    pub fn send(mut self) -> Result<MessageId, Error> {
        let params = self.params.unwrap_or_default();
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("PendingCtor polled after completion or invalid state"));
        let payload = T::encode_params(&args);

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

impl<T: CallEncodeDecode> Future for PendingCall<GstdEnv, T> {
    type Output = Result<T::Reply, <GstdEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            // Send message
            let payload = self.encode_call();
            let params = self.params.take().unwrap_or_default();

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
                Poll::Ready(Ok(bytes)) => match T::Reply::decode(&mut bytes.as_slice()) {
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

impl<A, T: CallEncodeDecode> Future for PendingCtor<GstdEnv, A, T> {
    type Output = Result<Actor<A, GstdEnv>, <GstdEnv as GearEnv>::Error>;

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
                prog::create_program_bytes_for_reply(code_id, salt, payload, value)?;

            // self.state = Some(program_future);
            self.program_id = Some(program_future.program_id);
        }
        let this = self.project();
        if let Some(message_fut) = this.state.as_pin_mut() {
            // Poll message future
            match message_fut.poll(cx) {
                Poll::Ready(Ok(bytes)) => match T::Reply::decode(&mut bytes.as_slice()) {
                    Ok(_decoded) => {
                        let program_id = this.program_id.take().unwrap_or_else(|| {
                            panic!("PendingCtor polled after completion or invalid state")
                        });
                        let env = this.env.clone();
                        Poll::Ready(Ok(Actor::new(env, program_id)))
                    }
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
