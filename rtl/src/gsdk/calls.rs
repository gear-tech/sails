use crate::{calls::Remoting, errors::Result, ActorId, CodeId, GasUnit, ValueUnit, Vec};
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
    // pub async fn new(address: WSAddress) -> Result<Self> {
    //     let api = GearApi::init_with(address).await?;
    //     Ok(Self { api })
    // }

    pub async fn dev() -> Result<Self> {
        let api = GearApi::dev().await.unwrap();
        Ok(Self { api })
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
        let mut listener = api.subscribe().await.unwrap();
        let (message_id, program_id, ..) = api
            .create_program_bytes(
                code_id,
                salt,
                payload,
                args.gas_limit.unwrap_or_default(),
                value,
            )
            .await
            .unwrap();

        let (_, result, _) = listener.reply_bytes_on(message_id).await.unwrap();
        Ok(async move {
            let reply: Vec<u8> = result.unwrap();
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
        let mut listener = api.subscribe().await.unwrap();
        let (message_id, ..) = api
            .send_message_bytes(target, payload, args.gas_limit.unwrap_or_default(), value)
            .await
            .unwrap();

        let (_, result, _) = listener.reply_bytes_on(message_id).await.unwrap();
        Ok(async move {
            let reply = result.unwrap();
            Ok(reply)
        })
    }
}
