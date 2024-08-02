use crate::{
    calls::Remoting,
    errors::{Result, RtlError},
    events::Listener,
    prelude::*,
};
use core::future::Future;
use futures::{stream, Stream, StreamExt};
use gclient::metadata::runtime_types::gear_core::message::user::UserMessage as GenUserMessage;
use gclient::{ext::sp_core::ByteArray, EventProcessor, GearApi};
use gear_core_errors::ReplyCode;

#[derive(Debug, Default, Clone)]
pub struct GSdkArgs;

#[derive(Clone)]
pub struct GSdkRemoting {
    api: GearApi,
}

impl GSdkRemoting {
    pub fn new(api: GearApi) -> Self {
        Self { api }
    }

    pub fn with_suri(self, suri: impl AsRef<str>) -> Self {
        let api = self.api.with(suri).unwrap();
        Self { api }
    }

    pub fn api(&self) -> &GearApi {
        &self.api
    }

    pub async fn upload_code_by_path(&self, path: &str) -> Result<CodeId> {
        let (code_id, ..) = self.api.upload_code_by_path(path).await?;
        Ok(code_id)
    }
}

impl Remoting for GSdkRemoting {
    type Args = GSdkArgs;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        _args: GSdkArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let api = self.api;
        // Calculate gas amount if it is not explicitly set
        let gas_limit = if let Some(gas_limit) = gas_limit {
            gas_limit
        } else {
            // Calculate gas amount needed for initialization
            let gas_info = api
                .calculate_create_gas(None, code_id, Vec::from(payload.as_ref()), value, true)
                .await?;
            gas_info.min_limit
        };

        let mut listener = api.subscribe().await?;
        let (message_id, program_id, ..) = api
            .create_program_bytes(code_id, salt, payload, gas_limit, value)
            .await?;

        Ok(async move {
            let (_, result, _) = listener.reply_bytes_on(message_id).await?;
            let reply = result.map_err(RtlError::ReplyHasErrorString)?;
            Ok((program_id, reply))
        })
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        _args: GSdkArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let api = self.api;
        // Calculate gas amount if it is not explicitly set
        let gas_limit = if let Some(gas_limit) = gas_limit {
            gas_limit
        } else {
            // Calculate gas amount needed for handling the message
            let gas_info = api
                .calculate_handle_gas(None, target, Vec::from(payload.as_ref()), value, true)
                .await?;
            gas_info.min_limit
        };

        let mut listener = api.subscribe().await?;
        let (message_id, ..) = api
            .send_message_bytes(target, payload, gas_limit, value)
            .await?;

        Ok(async move {
            let (_, result, _) = listener.reply_bytes_on(message_id).await?;
            let reply = result.map_err(RtlError::ReplyHasErrorString)?;
            Ok(reply)
        })
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        _args: GSdkArgs,
    ) -> Result<Vec<u8>> {
        let api = self.api;
        // Get Max gas amount if it is not explicitly set
        let gas_limit = if let Some(gas_limit) = gas_limit {
            gas_limit
        } else {
            api.block_gas_limit()?
        };
        let origin = H256::from_slice(api.account_id().as_slice());
        let payload = payload.as_ref().to_vec();

        let reply_info = api
            .calculate_reply_for_handle(Some(origin), target, payload, gas_limit, value)
            .await?;

        match reply_info.code {
            ReplyCode::Success(_) => Ok(reply_info.payload),
            ReplyCode::Error(reason) => Err(RtlError::ReplyHasError(reason))?,
            ReplyCode::Unsupported => Err(RtlError::ReplyIsMissing)?,
        }
    }
}

impl Listener<Vec<u8>> for GSdkRemoting {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, Vec<u8>)> + Unpin> {
        let listener = self.api.subscribe().await?;
        let stream = stream::unfold(listener, |mut l| async move {
            let vec = get_events_from_block(&mut l).await.ok();
            vec.map(|v| (v, l))
        })
        .flat_map(stream::iter);
        Ok(Box::pin(stream))
    }
}

async fn get_events_from_block(
    listener: &mut gclient::EventListener,
) -> Result<Vec<(ActorId, Vec<u8>)>> {
    let vec = listener
        .proc_many(
            |e| {
                if let gclient::Event::Gear(gclient::GearEvent::UserMessageSent {
                    message:
                        GenUserMessage {
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
