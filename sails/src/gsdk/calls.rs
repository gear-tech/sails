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

#[derive(Debug, Default, Clone, Copy)]
pub struct GSdkArgs {
    gas_limit: Option<GasUnit>,
}

impl GSdkArgs {
    pub fn with_gas_limit(gas_limit: GasUnit) -> Self {
        Self {
            gas_limit: Some(gas_limit),
        }
    }

    pub fn gas_limit(&self) -> Option<GasUnit> {
        self.gas_limit
    }
}

#[derive(Clone)]
pub struct GSdkRemoting {
    api: GearApi,
    args: GSdkArgs,
}

impl GSdkRemoting {
    pub fn new(api: GearApi) -> Self {
        Self {
            api,
            args: Default::default(),
        }
    }

    pub fn with_args(self, args: GSdkArgs) -> Self {
        Self { args, ..self }
    }

    pub fn api(&self) -> &GearApi {
        &self.api
    }

    pub async fn upload_code_by_path(&self, path: &str) -> Result<CodeId> {
        let (code_id, ..) = self.api.upload_code_by_path(path).await?;
        Ok(code_id)
    }
}

impl Remoting<GSdkArgs> for GSdkRemoting {
    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GSdkArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let api = self.api;
        // Do not Calculate gas amount needed
        let gas_limit = args.gas_limit.unwrap_or_default();

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
        value: ValueUnit,
        args: GSdkArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let api = self.api;
        // Do not Calculate gas amount needed
        let gas_limit = args.gas_limit.unwrap_or_default();

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
        value: ValueUnit,
        args: GSdkArgs,
    ) -> Result<Vec<u8>> {
        let api = self.api;
        // Do not Calculate gas amount needed
        let gas_limit = args.gas_limit.unwrap_or_default();
        let origin = H256::from_slice(api.account_id().as_slice());
        let payload = payload.as_ref().to_vec();

        let reply_info = api
            .calculate_reply_for_handle(Some(origin), target, payload, gas_limit, value)
            .await?;

        Ok(reply_info.payload)
    }

    fn args(&self) -> GSdkArgs {
        self.args
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
