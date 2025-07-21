use super::*;

pub struct StakingBroker;

#[sails_rs::service]
impl StakingBroker {
    #[export]
    pub async fn bond(&mut self, value: u128, payee: RewardAccount) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .bond(value, payee)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn bond_extra(&mut self, value: u128) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .bond_extra(value)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn unbond(&mut self, value: u128) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .unbond(value)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn withdraw_unbonded(&mut self, num_slashing_spans: u32) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .withdraw_unbonded(num_slashing_spans)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn nominate(&mut self, targets: Vec<ActorId>) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .nominate(targets)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn chill(&mut self) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .chill()
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn payout_stakers(
        &mut self,
        validator_stash: ActorId,
        era: u32,
    ) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .payout_stakers(validator_stash, era)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn rebond(&mut self, value: u128) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .rebond(value)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }

    #[export]
    pub async fn set_payee(&mut self, payee: RewardAccount) -> Result<Vec<u8>, String> {
        let staking_builtin_client = StakingBuiltin::new(GStdRemoting::new());

        staking_builtin_client
            .set_payee(payee)
            .send_recv(STAKING_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending staking builtin request: {e}"))
    }
}
