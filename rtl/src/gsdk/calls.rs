use crate::{calls::Remoting, errors::Result, prelude::*};
use core::future::Future;
use gclient::{EventProcessor, GearApi};
use gear_core::ids::ProgramId;

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
        let mut listener = self.api.subscribe().await.unwrap();
        let code_id: gear_core::ids::CodeId = gear_core::ids::CodeId::from(*code_id.as_ref());
        let (message_id, program_id, ..) = &self
            .api
            .create_program_bytes(
                code_id,
                salt,
                payload,
                args.gas_limit.unwrap_or_default(),
                value,
            )
            .await
            .unwrap();

        let actor_id: ActorId = program_id.as_ref().into();
        let (_, result, _) = listener.reply_bytes_on(*message_id).await.unwrap();
        Ok(async move {
            let reply: Vec<u8> = result.unwrap();
            Ok((actor_id, reply))
        })
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: GSdkArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let mut listener = self.api.subscribe().await.unwrap();
        let destination: ProgramId = ProgramId::from(*target.as_ref());
        let (message_id, ..) = &self
            .api
            .send_message_bytes(
                destination,
                payload,
                args.gas_limit.unwrap_or_default(),
                value,
            )
            .await
            .unwrap();

        let (_, result, _) = listener.reply_bytes_on(*message_id).await.unwrap();
        Ok(async move {
            let reply = result.unwrap();
            Ok(reply)
        })
    }
}
