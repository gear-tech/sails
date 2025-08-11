use super::*;
use crate::events::Listener;
use ::gclient::{Error, EventListener, EventProcessor as _, GearApi};
use futures::{Stream, StreamExt as _, stream};

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
        ReplyCode::Error(_error_reply_reason) => todo!(),
        ReplyCode::Unsupported => todo!(),
    }
}

impl<T: CallEncodeDecode> PendingCall<GclientEnv, T> {
    pub async fn send(mut self) -> Result<MessageId, Error> {
        let api = &self.env.api;
        let params = self.params.unwrap_or_default();
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("PendingCtor polled after completion or invalid state"));
        let payload = T::encode_params_with_prefix(self.route.unwrap(), &args);
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

    pub async fn query(mut self) -> Result<T::Reply, Error> {
        let params = self.params.unwrap_or_default();
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("PendingCtor polled after completion or invalid state"));
        let payload = T::encode_params(&args);

        // Calculate reply
        let reply_bytes = self
            .env
            .query_calculate_reply(self.destination, payload, params)
            .await?;

        // Decode reply
        match T::Reply::decode(&mut reply_bytes.as_slice()) {
            Ok(decoded) => Ok(decoded),
            Err(err) => Err(Error::Codec(err)),
        }
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
                .unwrap_or_else(|| panic!("PendingCtor polled after completion or invalid state"));
            let payload = T::encode_params(&args);

            let send_future = send_message(self.env.api.clone(), self.destination, payload, params);
            self.state = Some(Box::pin(send_future));
        }
        if let Some(message_future) = self.project().state.as_pin_mut() {
            // Poll message future
            match message_future.poll(cx) {
                Poll::Ready(Ok(bytes)) => match T::Reply::decode(&mut bytes.as_slice()) {
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

impl Listener<Vec<u8>> for GclientEnv {
    type Error = Error;

    async fn listen(
        &mut self,
    ) -> Result<impl Stream<Item = (ActorId, Vec<u8>)> + Unpin, Self::Error> {
        let listener = self.api.subscribe().await?;
        let stream = stream::unfold(listener, |mut l| async move {
            let vec = get_events_from_block(&mut l).await.ok();
            vec.map(|v| (v, l))
        })
        .flat_map(stream::iter);
        Ok(Box::pin(stream))
    }
}

async fn wait_for_reply(
    listener: &mut EventListener,
    message_id: MessageId,
) -> Result<(MessageId, ReplyCode, Vec<u8>, ValueUnit), Error> {
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
    .await
}

async fn get_events_from_block(
    listener: &mut gclient::EventListener,
) -> Result<Vec<(ActorId, Vec<u8>)>, Error> {
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
