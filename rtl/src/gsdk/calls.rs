use crate::{
    calls::Remoting,
    errors::{Result, RtlError},
    ActorId, CodeId, GasUnit, ValueUnit, Vec,
};
use core::future::Future;
use gclient::{EventProcessor, GearApi};

#[derive(Debug, Default, Clone)]
pub struct GSdkArgs {
    gas_limit: Option<GasUnit>,
}

impl GSdkArgs {
    pub fn with_gas_limit(mut self, gas_limit: Option<GasUnit>) -> Self {
        self.gas_limit = gas_limit;
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
        let gas_limit = if let Some(gas_limit) = args.gas_limit {
            gas_limit
        } else {
            // api.block_gas_limit()?
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
        value: ValueUnit,
        args: GSdkArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let api = self.api;
        let gas_limit = if let Some(gas_limit) = args.gas_limit {
            gas_limit
        } else {
            //api.block_gas_limit()?
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
}
