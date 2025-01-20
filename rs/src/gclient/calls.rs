use crate::{
    calls::{Query, Remoting},
    errors::{Result, RtlError},
    events::Listener,
    prelude::*,
};
use core::future::Future;
use futures::{stream, Stream, StreamExt};
use gclient::metadata::runtime_types::{
    gear_core::message::user::UserMessage as GenUserMessage,
    pallet_gear_voucher::internal::VoucherId,
};
use gclient::{ext::sp_core::ByteArray, EventProcessor, GearApi};
use gear_core_errors::ReplyCode;

#[derive(Debug, Default)]
pub struct GClientArgs {
    voucher: Option<(VoucherId, bool)>,
    at_block: Option<H256>,
}

impl GClientArgs {
    pub fn with_voucher(mut self, voucher_id: VoucherId, keep_alive: bool) -> Self {
        self.voucher = Some((voucher_id, keep_alive));
        self
    }

    fn at_block(mut self, hash: H256) -> Self {
        self.at_block = Some(hash);
        self
    }
}

#[derive(Clone)]
pub struct GClientRemoting {
    api: GearApi,
}

impl GClientRemoting {
    pub fn new(api: GearApi) -> Self {
        Self { api }
    }

    pub fn with_suri(self, suri: impl AsRef<str>) -> Self {
        let api = self.api.with(suri).unwrap();
        Self { api }
    }
}

impl Remoting for GClientRemoting {
    type Args = GClientArgs;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        _args: GClientArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        let api = self.api;
        // Calculate gas amount if it is not explicitly set
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = if let Some(gas_limit) = gas_limit {
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
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GClientArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let api = self.api;
        // Calculate gas amount if it is not explicitly set
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = if let Some(gas_limit) = gas_limit {
            gas_limit
        } else {
            // Calculate gas amount needed for handling the message
            let gas_info = api
                .calculate_handle_gas(None, target, Vec::from(payload.as_ref()), value, true)
                .await?;
            gas_info.min_limit
        };
        #[cfg(feature = "ethexe")]
        let gas_limit = 0;

        let mut listener = api.subscribe().await?;
        let (message_id, ..) = if let Some((voucher_id, keep_alive)) = args.voucher {
            api.send_message_bytes_with_voucher(
                voucher_id, target, payload, gas_limit, value, keep_alive,
            )
            .await?
        } else {
            api.send_message_bytes(target, payload, gas_limit, value)
                .await?
        };

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
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GClientArgs,
    ) -> Result<Vec<u8>> {
        let api = self.api;
        // Get Max gas amount if it is not explicitly set
        #[cfg(not(feature = "ethexe"))]
        let gas_limit = if let Some(gas_limit) = gas_limit {
            gas_limit
        } else {
            api.block_gas_limit()?
        };
        #[cfg(feature = "ethexe")]
        let gas_limit = 0;
        let origin = H256::from_slice(api.account_id().as_slice());
        let payload = payload.as_ref().to_vec();

        let reply_info = api
            .calculate_reply_for_handle_at(
                Some(origin),
                target,
                payload,
                gas_limit,
                value,
                args.at_block,
            )
            .await?;

        match reply_info.code {
            ReplyCode::Success(_) => Ok(reply_info.payload),
            ReplyCode::Error(reason) => {
                let message = String::from_utf8_lossy(&reply_info.payload).to_string();
                Err(RtlError::ReplyHasError(reason, message))?
            }
            ReplyCode::Unsupported => Err(RtlError::ReplyIsMissing)?,
        }
    }
}

impl Listener<Vec<u8>> for GClientRemoting {
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

pub trait QueryAtBlock {
    /// Query at a specific block.
    fn at_block(self, hash: H256) -> Self;
}

impl<T> QueryAtBlock for T
where
    T: Query<Args = GClientArgs>,
{
    fn at_block(self, hash: H256) -> Self {
        self.with_args(GClientArgs::default().at_block(hash))
    }
}
