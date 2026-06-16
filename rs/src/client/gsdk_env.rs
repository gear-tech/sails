use super::*;
use core::task::ready;
use futures::{Stream, StreamExt};
pub use gear_core_errors::{ErrorReplyReason, SimpleExecutionError};
use gsdk::{
    SignedApi,
    gear::runtime_types::pallet_gear_voucher::internal::VoucherId,
    subscription::{UserMessageSentFilter, UserMessageSentSubscription},
};

#[derive(Debug, thiserror::Error)]
pub enum GsdkError {
    #[error(transparent)]
    Env(#[from] gsdk::Error),
    #[error("reply error: {0}")]
    ReplyHasError(ErrorReplyReason, crate::Vec<u8>),
    #[error("reply is missing")]
    ReplyIsMissing,
}

impl ReplyError for GsdkError {
    fn from_codec_error(err: parity_scale_codec::Error) -> Self {
        gsdk::Error::Codec(err).into()
    }

    fn userspace_panic_payload(&self) -> Option<&[u8]> {
        match self {
            GsdkError::ReplyHasError(
                ErrorReplyReason::Execution(SimpleExecutionError::UserspacePanic),
                payload,
            ) => Some(payload),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GsdkEnv {
    api: SignedApi,
}

crate::params_struct_impl!(
    GsdkEnv,
    GsdkParams {
        #[cfg(not(feature = "ethexe"))]
        gas_limit: GasUnit,
        value: ValueUnit,
        at_block: H256,
        voucher: (VoucherId, bool),
    }
);

impl GsdkEnv {
    pub fn new(api: SignedApi) -> Self {
        Self { api }
    }

    pub fn with_suri(self, suri: impl AsRef<str>) -> Self {
        let api = self
            .api
            .unsigned()
            .clone()
            .signed(suri.as_ref(), None)
            .unwrap();
        Self { api }
    }

    pub async fn create_program(
        &self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        params: GsdkParams,
    ) -> Result<(ActorId, Vec<u8>), GsdkError> {
        create_program(self.api.clone(), code_id, salt, payload, params).await
    }

    pub async fn send_for_reply(
        &self,
        program_id: ActorId,
        payload: Vec<u8>,
        params: GsdkParams,
    ) -> Result<Vec<u8>, GsdkError> {
        send_for_reply(self.api.clone(), program_id, payload, params)
            .await
            .map(|(_program_id, payload)| payload)
    }

    pub async fn send_one_way(
        &self,
        program_id: ActorId,
        payload: Vec<u8>,
        params: GsdkParams,
    ) -> Result<MessageId, GsdkError> {
        send_one_way(&self.api, program_id, payload, params).await
    }

    pub async fn query(
        &self,
        destination: ActorId,
        payload: impl AsRef<[u8]>,
        params: GsdkParams,
    ) -> Result<Vec<u8>, GsdkError> {
        query_calculate_reply(&self.api, destination, payload, params).await
    }
}

impl GearEnv for GsdkEnv {
    type Params = GsdkParams;
    type Error = GsdkError;
    type MessageState = Pin<Box<dyn Future<Output = Result<(ActorId, Vec<u8>), GsdkError>>>>;
}

impl EnvWithCtor for GsdkEnv {}

impl<T: ServiceCall> PendingCall<T, GsdkEnv> {
    pub async fn send_one_way(&mut self) -> Result<MessageId, GsdkError> {
        let (payload, params) = self.take_encoded_args_and_params();
        self.env
            .send_one_way(self.destination, payload, params)
            .await
    }

    pub async fn send_for_reply(mut self) -> Result<Self, GsdkError> {
        let (payload, params) = self.take_encoded_args_and_params();
        // Subscribe and dispatch eagerly, store only the reply-waiting future.
        let (subscription, message_id) =
            send_for_reply_and_listen(&self.env.api, self.destination, payload, params).await?;
        self.state = Some(Box::pin(wait_for_reply_owned(
            subscription,
            message_id,
            self.destination,
        )));
        Ok(self)
    }

    pub async fn query(mut self) -> Result<T::Output, GsdkError> {
        let (payload, params) = self.take_encoded_args_and_params();
        let reply = query_calculate_reply(&self.env.api, self.destination, payload, params).await;
        decode_reply_or_throw::<T, _>(&self.route, reply)
    }
}

impl<T: ServiceCall> Future for PendingCall<T, GsdkEnv> {
    type Output = Result<T::Output, <GsdkEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            let (payload, params) = self.take_encoded_args_and_params();
            // Send message
            let send_future =
                send_for_reply(self.env.api.clone(), self.destination, payload, params);
            self.state = Some(Box::pin(send_future));
        }

        let this = self.as_mut().project();
        let message_future = this
            .state
            .as_pin_mut()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        let reply = ready!(message_future.poll(cx)).map(|(_, payload)| payload);
        Poll::Ready(decode_reply_or_throw::<T, _>(this.route, reply))
    }
}

impl<A, T> Future for PendingCtor<A, T, GsdkEnv>
where
    T: ServiceCall,
    T::Output: PendingCtorOutput<A, GsdkEnv>,
{
    type Output =
        Result<<T::Output as PendingCtorOutput<A, GsdkEnv>>::Output, <GsdkEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_none() {
            // Send message
            let params = self.params.take().unwrap_or_default();
            let salt = self.salt.take().unwrap();
            let args = self
                .args
                .take()
                .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
            let payload = T::encode_call(&self.route, &args);

            let create_program_future =
                create_program(self.env.api.clone(), self.code_id, salt, payload, params);
            self.state = Some(Box::pin(create_program_future));
        }

        let this = self.as_mut().project();
        let message_future = this
            .state
            .as_pin_mut()
            .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
        let (reply, program_id) = match ready!(message_future.poll(cx)) {
            Ok((program_id, payload)) => (Ok(payload), program_id),
            Err(err) => (Err(err), ActorId::zero()),
        };
        match decode_reply_or_throw::<T, _>(this.route, reply) {
            Ok(output) => Poll::Ready(Ok(output.map_result(this.env.clone(), program_id))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl Listener for GsdkEnv {
    type Error = GsdkError;

    async fn listen<E, F: FnMut((ActorId, Vec<u8>)) -> Option<(ActorId, E)>>(
        &self,
        f: F,
    ) -> Result<impl Stream<Item = (ActorId, E)> + Unpin + use<E, F>, Self::Error> {
        // Events are user messages sent to the zero address.
        let subscription = self
            .api
            .subscribe_user_message_sent(
                UserMessageSentFilter::new().with_destination(ActorId::zero()),
            )
            .await?;
        // End the stream on the first subscription error so consumers observe
        // termination (`next()` returns `None`) instead of silently missing events.
        let stream = tokio_stream::StreamExt::map_while(subscription, |res| {
            res.ok().map(|m| (m.source, m.payload))
        });
        let stream = tokio_stream::StreamExt::filter_map(stream, f);
        Ok(Box::pin(stream))
    }
}

async fn create_program(
    api: SignedApi,
    code_id: CodeId,
    salt: impl AsRef<[u8]>,
    payload: impl AsRef<[u8]>,
    params: GsdkParams,
) -> Result<(ActorId, Vec<u8>), GsdkError> {
    let value = params.value.unwrap_or(0);
    let payload = Vec::from(payload.as_ref());
    // Calculate gas amount if it is not explicitly set
    #[cfg(not(feature = "ethexe"))]
    let gas_limit = if let Some(gas_limit) = params.gas_limit {
        gas_limit
    } else {
        // Calculate gas amount needed for initialization
        api.calculate_create_gas(code_id, payload.clone(), value, true)
            .await?
            .min_limit
    };
    #[cfg(feature = "ethexe")]
    let gas_limit = 0;

    // Subscribe before dispatching so the reply event can't fire before the listener exists.
    let user_id = ActorId::from(api.account_id().0);
    let mut subscription = api
        .subscribe_user_message_sent(UserMessageSentFilter::new().with_destination(user_id))
        .await?;
    let tx = api
        .create_program_bytes(code_id, Vec::from(salt.as_ref()), payload, gas_limit, value)
        .await?;
    let (message_id, program_id) = tx.value;
    let payload = wait_for_reply(&mut subscription, message_id).await?;
    Ok((program_id, payload))
}

async fn send_for_reply(
    api: SignedApi,
    program_id: ActorId,
    payload: Vec<u8>,
    params: GsdkParams,
) -> Result<(ActorId, Vec<u8>), GsdkError> {
    let (subscription, message_id) =
        send_for_reply_and_listen(&api, program_id, payload, params).await?;
    wait_for_reply_owned(subscription, message_id, program_id).await
}

// Subscribes before dispatching so the reply event can't fire before the listener exists.
async fn send_for_reply_and_listen(
    api: &SignedApi,
    program_id: ActorId,
    payload: Vec<u8>,
    params: GsdkParams,
) -> Result<(UserMessageSentSubscription, MessageId), GsdkError> {
    #[cfg(not(feature = "ethexe"))]
    let gas_limit =
        calculate_gas_limit(api, program_id, &payload, params.gas_limit, params.value).await?;
    #[cfg(feature = "ethexe")]
    let gas_limit = 0;
    let value = params.value.unwrap_or(0);

    let user_id = ActorId::from(api.account_id().0);
    let subscription = api
        .subscribe_user_message_sent(UserMessageSentFilter::new().with_destination(user_id))
        .await?;
    let message_id = send_message_with_voucher_if_some(
        api,
        program_id,
        payload,
        gas_limit,
        value,
        params.voucher,
    )
    .await?;
    Ok((subscription, message_id))
}

async fn wait_for_reply_owned(
    mut subscription: UserMessageSentSubscription,
    message_id: MessageId,
    program_id: ActorId,
) -> Result<(ActorId, Vec<u8>), GsdkError> {
    let payload = wait_for_reply(&mut subscription, message_id).await?;
    Ok((program_id, payload))
}

async fn wait_for_reply(
    subscription: &mut UserMessageSentSubscription,
    message_id: MessageId,
) -> Result<Vec<u8>, GsdkError> {
    while let Some(item) = subscription.next().await {
        let message = item?;
        let Some(reply) = message.reply else {
            continue;
        };
        if reply.to != message_id {
            continue;
        }
        return match reply.code {
            ReplyCode::Success(_) => Ok(message.payload),
            ReplyCode::Error(reason) => Err(GsdkError::ReplyHasError(reason, message.payload)),
            ReplyCode::Unsupported => Err(GsdkError::ReplyIsMissing),
        };
    }
    Err(GsdkError::ReplyIsMissing)
}

async fn send_one_way(
    api: &SignedApi,
    program_id: ActorId,
    payload: Vec<u8>,
    params: GsdkParams,
) -> Result<MessageId, GsdkError> {
    #[cfg(not(feature = "ethexe"))]
    let gas_limit =
        calculate_gas_limit(api, program_id, &payload, params.gas_limit, params.value).await?;
    #[cfg(feature = "ethexe")]
    let gas_limit = 0;
    let value = params.value.unwrap_or(0);

    send_message_with_voucher_if_some(api, program_id, payload, gas_limit, value, params.voucher)
        .await
}

async fn send_message_with_voucher_if_some(
    api: &SignedApi,
    program_id: ActorId,
    payload: impl Into<Vec<u8>>,
    gas_limit: GasUnit,
    value: ValueUnit,
    voucher: Option<(VoucherId, bool)>,
) -> Result<MessageId, GsdkError> {
    let tx = if let Some((voucher_id, keep_alive)) = voucher {
        api.send_message_bytes_with_voucher(
            voucher_id, program_id, payload, gas_limit, value, keep_alive,
        )
        .await?
    } else {
        api.send_message_bytes(program_id, payload, gas_limit, value)
            .await?
    };
    Ok(tx.value)
}

#[cfg(not(feature = "ethexe"))]
async fn calculate_gas_limit(
    api: &SignedApi,
    program_id: ActorId,
    payload: impl AsRef<[u8]>,
    gas_limit: Option<GasUnit>,
    value: Option<ValueUnit>,
) -> Result<u64, GsdkError> {
    let gas_limit = if let Some(gas_limit) = gas_limit {
        gas_limit
    } else {
        // Calculate gas amount needed for handling the message
        api.calculate_handle_gas(
            program_id,
            payload.as_ref().to_vec(),
            value.unwrap_or(0),
            true,
        )
        .await?
        .min_limit
    };
    Ok(gas_limit)
}

async fn query_calculate_reply(
    api: &SignedApi,
    destination: ActorId,
    payload: impl AsRef<[u8]>,
    params: GsdkParams,
) -> Result<Vec<u8>, GsdkError> {
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
    let payload = payload.as_ref().to_vec();
    let at_block = params
        .at_block
        .map(|at| gsdk::ext::subxt::utils::H256::from(at.0));

    let reply_info = api
        .calculate_reply_for_handle_at(destination, payload, gas_limit, value, at_block)
        .await?;

    match reply_info.code {
        ReplyCode::Success(_) => Ok(reply_info.payload),
        ReplyCode::Error(reason) => Err(GsdkError::ReplyHasError(reason, reply_info.payload)),
        ReplyCode::Unsupported => Err(GsdkError::ReplyIsMissing),
    }
}
