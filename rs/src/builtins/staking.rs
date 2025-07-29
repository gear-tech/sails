use crate::{
    ActorId,
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::Result,
    prelude::{Decode, Encode, TypeInfo, Vec},
};
use gbuiltin_staking::{Request as GearStakingRequest, RewardAccount as GearRewardAccount};

builtin_action!(
    StakingRequest,
    StakingBuiltin,
    Bond {
        value: u128,
        payee: RewardAccount
    }
);

builtin_action!(StakingRequest, StakingBuiltin, BondExtra { value: u128 });

builtin_action!(StakingRequest, StakingBuiltin, Unbond { value: u128 });

builtin_action!(
    StakingRequest,
    StakingBuiltin,
    WithdrawUnbonded {
        num_slashing_spans: u32
    }
);

builtin_action!(
    StakingRequest,
    StakingBuiltin,
    Nominate { targets: Vec<ActorId> }
);

builtin_action!(StakingRequest, StakingBuiltin, Chill);

builtin_action!(
    StakingRequest,
    StakingBuiltin,
    PayoutStakers {
        validator_stash: ActorId,
        era: u32
    }
);

builtin_action!(StakingRequest, StakingBuiltin, Rebond { value: u128 });

builtin_action!(
    StakingRequest,
    StakingBuiltin,
    SetPayee {
        payee: RewardAccount
    }
);

pub struct StakingBuiltin<R> {
    remoting: R,
}

impl<R: BuiltinsRemoting + Clone> StakingBuiltin<R> {
    pub fn new(remoting: R) -> Self {
        Self { remoting }
    }
}

pub trait StakingBuiltinTrait {
    type Args;

    /// Bond up to the `value` from the sender to self as the controller.
    fn bond(&self, value: u128, payee: RewardAccount) -> impl Call<Output = (), Args = Self::Args>;

    /// Add up to the `value` to the sender's bonded amount.
    fn bond_extra(&self, value: u128) -> impl Call<Output = (), Args = Self::Args>;

    /// Unbond up to the `value` to allow withdrawal after undonding period.
    fn unbond(&self, value: u128) -> impl Call<Output = (), Args = Self::Args>;

    /// Withdraw unbonded chunks for which undonding period has elapsed.
    fn withdraw_unbonded(
        &self,
        num_slashing_spans: u32,
    ) -> impl Call<Output = (), Args = Self::Args>;

    /// Add sender as a nominator of `targets` or update the existing targets.
    fn nominate(&self, targets: Vec<ActorId>) -> impl Call<Output = (), Args = Self::Args>;

    /// Declare intention to [temporarily] stop nominating while still having funds bonded.
    fn chill(&self) -> impl Call<Output = (), Args = Self::Args>;

    /// Request stakers payout for the given era.
    fn payout_stakers(
        &self,
        validator_stash: ActorId,
        era: u32,
    ) -> impl Call<Output = (), Args = Self::Args>;

    /// Rebond a portion of the sender's stash scheduled to be unlocked.
    fn rebond(&self, value: u128) -> impl Call<Output = (), Args = Self::Args>;

    /// Set the reward destination.
    fn set_payee(&self, payee: RewardAccount) -> impl Call<Output = (), Args = Self::Args>;
}

impl<R: BuiltinsRemoting + Clone> StakingBuiltinTrait for StakingBuiltin<R> {
    type Args = R::Args;

    fn bond(&self, value: u128, payee: RewardAccount) -> impl Call<Output = (), Args = Self::Args> {
        self.bond(value, payee)
    }

    fn bond_extra(&self, value: u128) -> impl Call<Output = (), Args = Self::Args> {
        self.bond_extra(value)
    }

    fn unbond(&self, value: u128) -> impl Call<Output = (), Args = Self::Args> {
        self.unbond(value)
    }

    fn withdraw_unbonded(
        &self,
        num_slashing_spans: u32,
    ) -> impl Call<Output = (), Args = Self::Args> {
        self.withdraw_unbonded(num_slashing_spans)
    }

    fn nominate(&self, targets: Vec<ActorId>) -> impl Call<Output = (), Args = Self::Args> {
        self.nominate(targets)
    }

    fn chill(&self) -> impl Call<Output = (), Args = Self::Args> {
        self.chill()
    }

    fn payout_stakers(
        &self,
        validator_stash: ActorId,
        era: u32,
    ) -> impl Call<Output = (), Args = Self::Args> {
        self.payout_stakers(validator_stash, era)
    }

    fn rebond(&self, value: u128) -> impl Call<Output = (), Args = Self::Args> {
        self.rebond(value)
    }

    fn set_payee(&self, payee: RewardAccount) -> impl Call<Output = (), Args = Self::Args> {
        self.set_payee(payee)
    }
}

/// `TypeInfo` implementor copy of `gbuiltin_staking::Request`.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum StakingRequest {
    /// Bond up to the `value` from the sender to self as the controller.
    Bond { value: u128, payee: RewardAccount },
    /// Add up to the `value` to the sender's bonded amount.
    BondExtra { value: u128 },
    /// Unbond up to the `value` to allow withdrawal after undonding period.
    Unbond { value: u128 },
    /// Withdraw unbonded chunks for which undonding period has elapsed.
    WithdrawUnbonded { num_slashing_spans: u32 },
    /// Add sender as a nominator of `targets` or update the existing targets.
    Nominate { targets: Vec<ActorId> },
    /// Declare intention to [temporarily] stop nominating while still having funds bonded.
    Chill,
    /// Request stakers payout for the given era.
    PayoutStakers { validator_stash: ActorId, era: u32 },
    /// Rebond a portion of the sender's stash scheduled to be unlocked.
    Rebond { value: u128 },
    /// Set the reward destination.
    SetPayee { payee: RewardAccount },
}

impl From<GearStakingRequest> for StakingRequest {
    fn from(value: GearStakingRequest) -> Self {
        match value {
            GearStakingRequest::Bond { value, payee } => {
                let payee = payee.into();
                StakingRequest::Bond { value, payee }
            }
            GearStakingRequest::BondExtra { value } => StakingRequest::BondExtra { value },
            GearStakingRequest::Unbond { value } => StakingRequest::Unbond { value },
            GearStakingRequest::WithdrawUnbonded { num_slashing_spans } => {
                StakingRequest::WithdrawUnbonded { num_slashing_spans }
            }
            GearStakingRequest::Nominate { targets } => StakingRequest::Nominate { targets },
            GearStakingRequest::Chill => StakingRequest::Chill,
            GearStakingRequest::PayoutStakers {
                validator_stash,
                era,
            } => StakingRequest::PayoutStakers {
                validator_stash,
                era,
            },
            GearStakingRequest::Rebond { value } => StakingRequest::Rebond { value },
            GearStakingRequest::SetPayee { payee } => {
                let payee = payee.into();
                StakingRequest::SetPayee { payee }
            }
        }
    }
}

/// `TypeInfo` implementor copy of `gbuiltin_staking::RewardAccount`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum RewardAccount {
    /// Pay rewards to the sender's account and increase the amount at stake.
    Staked,
    /// Pay rewards to the sender's account (usually, the one derived from `program_id`)
    /// without increasing the amount at stake.
    Program,
    /// Pay rewards to a custom account.
    Custom(ActorId),
    /// Opt for not receiving any rewards at all.
    None,
}

impl From<GearRewardAccount> for RewardAccount {
    fn from(value: GearRewardAccount) -> Self {
        match value {
            GearRewardAccount::Staked => RewardAccount::Staked,
            GearRewardAccount::Program => RewardAccount::Program,
            GearRewardAccount::Custom(account) => RewardAccount::Custom(account),
            GearRewardAccount::None => RewardAccount::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::test_utils;
    use crate::{builtins::test_utils::assert_action_codec, prelude::vec};

    #[test]
    fn test_codec() {
        assert_action_codec!(
            StakingRequest,
            Bond {
                value: 1000,
                payee: RewardAccount::Staked
            }
        );
        assert_action_codec!(StakingRequest, BondExtra { value: 500 });
        assert_action_codec!(StakingRequest, Unbond { value: 200 });
        assert_action_codec!(
            StakingRequest,
            WithdrawUnbonded {
                num_slashing_spans: 3
            }
        );
        assert_action_codec!(
            StakingRequest,
            Nominate {
                targets: vec![ActorId::from([1; 32]), ActorId::from([2; 32])]
            }
        );
        assert_action_codec!(StakingRequest, Chill);
        assert_action_codec!(
            StakingRequest,
            PayoutStakers {
                validator_stash: ActorId::from([3; 32]),
                era: 42
            }
        );
        assert_action_codec!(StakingRequest, Rebond { value: 300 });
        assert_action_codec!(
            StakingRequest,
            SetPayee {
                payee: RewardAccount::Custom(ActorId::from([4; 32]))
            }
        );
    }
}
