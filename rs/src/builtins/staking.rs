use crate::{
    ActorId,
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::Result,
    prelude::{Decode, Encode, TypeInfo, Vec},
};
use gbuiltin_staking::{Request as GearStakingRequest, RewardAccount as GearRewardAccount};

// todo [sab] package must provide the address 77f65ef190e11bfecb8fc8970fd3749e94bed66a23ec2f7a3623e785d0816761
pub const STAKING_BUILTIN_ID: ActorId = ActorId::new([
    0x77, 0xf6, 0x5e, 0xf1, 0x90, 0xe1, 0x1b, 0xfe, 0xcb, 0x8f, 0xc8, 0x97, 0x0f, 0xd3, 0x74, 0x9e,
    0x94, 0xbe, 0xd6, 0x6a, 0x23, 0xec, 0x2f, 0x7a, 0x36, 0x23, 0xe7, 0x85, 0xd0, 0x81, 0x67, 0x61,
]);

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
