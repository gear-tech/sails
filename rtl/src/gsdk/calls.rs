use crate::{
    calls::Remoting,
    errors::{Result, RtlError},
    event_listener::{EventListener, EventSubscriber},
    ActorId, CodeId, GasUnit, MessageId, ValueUnit, Vec,
};
use core::future::Future;
use gclient::metadata::runtime_types::gear_core::message::user::UserMessage as GenUserMessage;
use gclient::{ext::sp_core::ByteArray, EventProcessor, GearApi};
use gprimitives::H256;

#[derive(Debug, Default, Clone)]
pub struct GSdkArgs {
    gas_limit: Option<GasUnit>,
}

impl GSdkArgs {
    pub fn with_gas_limit(mut self, gas_limit: GasUnit) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    pub fn gas_limit(&self) -> Option<GasUnit> {
        self.gas_limit
    }
}

#[derive(Clone)]
pub struct GSdkRemoting {
    api: GearApi,
}

impl GSdkRemoting {
    pub fn new(api: GearApi) -> Self {
        Self { api }
    }

    pub fn api(&self) -> &GearApi {
        &self.api
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
}

pub struct GSdkEventListener {
    listener: gclient::EventListener,
    events: Vec<(ActorId, MessageId, Vec<u8>)>,
}

impl EventSubscriber for GSdkRemoting {
    async fn subscribe(&mut self) -> Result<impl EventListener> {
        let listener = self.api.subscribe().await?;
        Ok(GSdkEventListener {
            listener,
            events: Vec::new(),
        })
    }
}

impl GSdkEventListener {
    async fn get_events_from_block(&mut self) -> Result<Vec<(ActorId, MessageId, Vec<u8>)>> {
        let vec = self
            .listener
            .proc_many(
                |e| {
                    if let gclient::Event::Gear(gclient::GearEvent::UserMessageSent {
                        message:
                            GenUserMessage {
                                id,
                                source,
                                destination,
                                payload,
                                ..
                            },
                        ..
                    }) = e
                    {
                        let source = ActorId::from(source);
                        let message_id = MessageId::from(id);
                        if ActorId::from(destination) == ActorId::zero() {
                            Some((source, message_id, payload.0))
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
}

impl EventListener for GSdkEventListener {
    async fn next_event<T>(
        &mut self,
        predicate: impl Fn((ActorId, MessageId, Vec<u8>)) -> Option<T>,
    ) -> Result<T> {
        loop {
            while let Some(evt) = self.events.pop() {
                if let Some(t) = predicate(evt) {
                    return Ok(t);
                }
            }
            self.events = self.get_events_from_block().await?;
            self.events.reverse();
        }
    }
}
