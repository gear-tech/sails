use super::*;
use ::gclient::{EventListener, EventProcessor as _, GearApi};
use core::task::ready;
use futures::{Stream, StreamExt, stream};

#[derive(Debug, thiserror::Error)]
pub enum GclientError {
    #[error(transparent)]
    Env(#[from] gclient::Error),
    #[error("reply error: {0}")]
    ReplyHasError(ErrorReplyReason, crate::Vec<u8>),
    #[error("unsupported")]
    Unsupported,
}

#[derive(Clone)]
pub struct GclientEnv {
    api: GearApi,
}

crate::params_struct_impl!(
    GclientEnv,
    GclientParams {
        #[cfg(not(feature = "ethexe"))]
        gas_limit: GasUnit,
        value: ValueUnit,
        at_block: H256,
    }
);

impl GclientEnv {
    pub fn new(api: GearApi) -> Self {
        Self { api }
    }

    pub fn with_suri(self, suri: impl AsRef<str>) -> Self {
        let api = self.api.with(suri).unwrap();
        Self { api }
    }

    pub async fn create_program(
        &self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        params: GclientParams,
    ) -> Result<(ActorId, Vec<u8>), GclientError> {
        let api = self.api.clone();
        create_program(api, code_id, salt, payload, params).await
    }
    pub async fn send_message(
        &self,
        program_id: ActorId,
        payload: Vec<u8>,
        params: GclientParams,
    ) -> Result<Vec<u8>, GclientError> {
        let api = self.api.clone();
        send_message(api, program_id, payload, params)
            .await
            .map(|(_program_id, payload)| payload)
    }

    pub async fn query_calculate_reply(
        &self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        params: GclientParams,
    ) -> Result<Vec<u8>, GclientError> {
        let api = self.api.clone();
        query_calculate_reply(api, target, payload, params).await
    }
}

async fn query_calculate_reply(
    api: GearApi,
    target: ActorId,
    payload: impl AsRef<[u8]>,
    params: GclientParams,
) -> Result<Vec<u8>, GclientError> {
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
        ReplyCode::Error(reason) => Err(GclientError::ReplyHasError(reason, reply_info.payload)),
        // TODO
        ReplyCode::Unsupported => Err(GclientError::Unsupported),
    }
}

impl GearEnv for GclientEnv {
    type Params = GclientParams;
    type Error = GclientError;
    type MessageState = Pin<Box<dyn Future<Output = Result<(ActorId, Vec<u8>), GclientError>>>>;
}

async fn create_program(
    api: GearApi,
    code_id: CodeId,
    salt: impl AsRef<[u8]>,
    payload: impl AsRef<[u8]>,
    params: GclientParams,
) -> Result<(ActorId, Vec<u8>), GclientError> {
    let value = params.value.unwrap_or(0);
    // Calculate gas amount if it is not explicitly set
    #[cfg(not(feature = "ethexe"))]
    let gas_limit = if let Some(gas_limit) = params.gas_limit {
        gas_limit
    } else {
        // Calculate gas amount needed for initialization
        let gas_info = api
            .calculate_create_gas(None, code_id, Vec::from(payload.as_ref()), value, true)
            .await?;
        gas_info.min_limit
    };
    #[cfg(feature = "ethexe")]
    let gas_limit = 0;

    let mut listener = api.subscribe().await?;
    let (message_id, program_id, ..) = api
        .create_program_bytes(code_id, salt, payload, gas_limit, value)
        .await?;
    let (_, reply_code, payload, _) = wait_for_reply(&mut listener, message_id).await?;
    match reply_code {
        ReplyCode::Success(_) => Ok((program_id, payload)),
        ReplyCode::Error(reason) => Err(GclientError::ReplyHasError(reason, payload)),
        ReplyCode::Unsupported => Err(GclientError::Unsupported),
    }
}

async fn send_message(
    api: GearApi,
    program_id: ActorId,
    payload: Vec<u8>,
    params: GclientParams,
) -> Result<(ActorId, Vec<u8>), GclientError> {
    let value = params.value.unwrap_or(0);
    #[cfg(not(feature = "ethexe"))]
    let gas_limit = if let Some(gas_limit) = params.gas_limit {
        gas_limit
    } else {
        // Calculate gas amount needed for handling the message
        let gas_info = api
            .calculate_handle_gas(None, program_id, payload.clone(), value, true)
            .await?;
        gas_info.min_limit
    };
    #[cfg(feature = "ethexe")]
    let gas_limit = 0;

    let mut listener = api.subscribe().await?;
    let (message_id, ..) = api
        .send_message_bytes(program_id, payload, gas_limit, value)
        .await?;
    let (_, reply_code, payload, _) = wait_for_reply(&mut listener, message_id).await?;
    match reply_code {
        ReplyCode::Success(_) => Ok((program_id, payload)),
        ReplyCode::Error(reason) => Err(GclientError::ReplyHasError(reason, payload)),
        ReplyCode::Unsupported => Err(GclientError::Unsupported),
    }
}

impl<T: CallEncodeDecode> PendingCall<GclientEnv, T> {
    pub async fn send(mut self) -> Result<MessageId, GclientError> {
        let api = &self.env.api;
        let params = self.params.unwrap_or_default();
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        let payload = T::encode_params_with_prefix(self.route, &args);
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

    pub async fn query(mut self) -> Result<T::Reply, GclientError> {
        let params = self.params.unwrap_or_default();
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        let payload = T::encode_params_with_prefix(self.route, &args);

        // Calculate reply
        let reply_bytes = self
            .env
            .query_calculate_reply(self.destination, payload, params)
            .await?;

        // Decode reply
        T::decode_reply_with_prefix(self.route, reply_bytes)
            .map_err(|err| gclient::Error::Codec(err).into())
    }
}

impl<T: CallEncodeDecode> Future for PendingCall<GclientEnv, T> {
    type Output = Result<T::Reply, <GclientEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            // Send message
            let params = self.params.take().unwrap_or_default();
            let args = self
                .args
                .take()
                .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
            let payload = T::encode_params_with_prefix(self.route, &args);

            let send_future = send_message(self.env.api.clone(), self.destination, payload, params);
            self.state = Some(Box::pin(send_future));
        }

        let this = self.as_mut().project();
        let message_future = this
            .state
            .as_pin_mut()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        // Poll message future
        match ready!(message_future.poll(cx)) {
            Ok((_, payload)) => match T::decode_reply_with_prefix(self.route, payload) {
                Ok(decoded) => Poll::Ready(Ok(decoded)),
                Err(err) => Poll::Ready(Err(gclient::Error::Codec(err).into())),
            },
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<A, T: CallEncodeDecode> Future for PendingCtor<GclientEnv, A, T> {
    type Output = Result<Actor<GclientEnv, A>, <GclientEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            // Send message
            let params = self.params.take().unwrap_or_default();
            let salt = self.salt.take().unwrap();
            let args = self
                .args
                .take()
                .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
            let payload = T::encode_params(&args);

            let create_program_future =
                create_program(self.env.api.clone(), self.code_id, salt, payload, params);
            self.state = Some(Box::pin(create_program_future));
        }

        let this = self.as_mut().project();
        let message_future = this
            .state
            .as_pin_mut()
            .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
        // Poll message future
        match ready!(message_future.poll(cx)) {
            Ok((program_id, _)) => {
                // Do not decode payload here
                Poll::Ready(Ok(Actor::new(this.env.clone(), program_id)))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl Listener for GclientEnv {
    type Error = GclientError;

    async fn listen<E, F: FnMut((ActorId, Vec<u8>)) -> Option<(ActorId, E)>>(
        &self,
        f: F,
    ) -> Result<impl Stream<Item = (ActorId, E)> + Unpin, Self::Error> {
        let listener = self.api.subscribe().await?;
        let stream = stream::unfold(listener, |mut l| async move {
            let vec = get_events_from_block(&mut l).await.ok();
            vec.map(|v| (v, l))
        })
        .flat_map(stream::iter);
        let stream = tokio_stream::StreamExt::filter_map(stream, f);
        Ok(Box::pin(stream))
    }
}

async fn wait_for_reply(
    listener: &mut EventListener,
    message_id: MessageId,
) -> Result<(MessageId, ReplyCode, Vec<u8>, ValueUnit), GclientError> {
    let message_id: ::gclient::metadata::runtime_types::gprimitives::MessageId = message_id.into();
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
            to.eq(&message_id).then(|| (id.into(), code.into(), payload.0.clone(), value))
        } else {
            None
        }
    })
    .await.map_err(Into::into)
}

async fn get_events_from_block(
    listener: &mut EventListener,
) -> Result<Vec<(ActorId, Vec<u8>)>, GclientError> {
    let vec = listener
        .proc_many(
            |e| {
                if let ::gclient::Event::Gear(::gclient::GearEvent::UserMessageSent {
                    message:
                        ::gclient::metadata::runtime_types::gear_core::message::user::UserMessage {
                            id: _,
                            source,
                            destination,
                            payload,
                            ..
                        },
                    ..
                }) = e
                {
                    let source = ActorId::from(source);
                    if ActorId::from(destination) == ActorId::zero() {
                        Some((source, payload.0))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            |v| (v, true),
        )
        .await?;
    Ok(vec)
}
