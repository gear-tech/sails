//------------------------------------------------------------------------------
// <auto-generated>
//     This code was generated by a tool.
//
//     Changes to this file may cause incorrect behavior and will be lost if
//     the code is regenerated.
// </auto-generated>
//------------------------------------------------------------------------------

using Substrate.NetApi.Model.Types.Base;
using System.Collections.Generic;


namespace Substrate.Gear.Api.Generated.Model.pallet_staking.pallet.pallet
{
    
    
    /// <summary>
    /// >> Event
    /// The `Event` enum of this pallet
    /// </summary>
    public enum Event
    {
        
        /// <summary>
        /// >> EraPaid
        /// The era payout has been set; the first balance is the validator-payout; the second is
        /// the remainder from the maximum amount of reward.
        /// </summary>
        EraPaid = 0,
        
        /// <summary>
        /// >> Rewarded
        /// The nominator has been rewarded by this amount to this destination.
        /// </summary>
        Rewarded = 1,
        
        /// <summary>
        /// >> Slashed
        /// A staker (validator or nominator) has been slashed by the given amount.
        /// </summary>
        Slashed = 2,
        
        /// <summary>
        /// >> SlashReported
        /// A slash for the given validator, for the given percentage of their stake, at the given
        /// era as been reported.
        /// </summary>
        SlashReported = 3,
        
        /// <summary>
        /// >> OldSlashingReportDiscarded
        /// An old slashing report from a prior era was discarded because it could
        /// not be processed.
        /// </summary>
        OldSlashingReportDiscarded = 4,
        
        /// <summary>
        /// >> StakersElected
        /// A new set of stakers was elected.
        /// </summary>
        StakersElected = 5,
        
        /// <summary>
        /// >> Bonded
        /// An account has bonded this amount. \[stash, amount\]
        /// 
        /// NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
        /// it will not be emitted for staking rewards when they are added to stake.
        /// </summary>
        Bonded = 6,
        
        /// <summary>
        /// >> Unbonded
        /// An account has unbonded this amount.
        /// </summary>
        Unbonded = 7,
        
        /// <summary>
        /// >> Withdrawn
        /// An account has called `withdraw_unbonded` and removed unbonding chunks worth `Balance`
        /// from the unlocking queue.
        /// </summary>
        Withdrawn = 8,
        
        /// <summary>
        /// >> Kicked
        /// A nominator has been kicked from a validator.
        /// </summary>
        Kicked = 9,
        
        /// <summary>
        /// >> StakingElectionFailed
        /// The election failed. No new era is planned.
        /// </summary>
        StakingElectionFailed = 10,
        
        /// <summary>
        /// >> Chilled
        /// An account has stopped participating as either a validator or nominator.
        /// </summary>
        Chilled = 11,
        
        /// <summary>
        /// >> PayoutStarted
        /// The stakers' rewards are getting paid.
        /// </summary>
        PayoutStarted = 12,
        
        /// <summary>
        /// >> ValidatorPrefsSet
        /// A validator has set their preferences.
        /// </summary>
        ValidatorPrefsSet = 13,
        
        /// <summary>
        /// >> SnapshotVotersSizeExceeded
        /// Voters size limit reached.
        /// </summary>
        SnapshotVotersSizeExceeded = 14,
        
        /// <summary>
        /// >> SnapshotTargetsSizeExceeded
        /// Targets size limit reached.
        /// </summary>
        SnapshotTargetsSizeExceeded = 15,
        
        /// <summary>
        /// >> ForceEra
        /// A new force era mode was set.
        /// </summary>
        ForceEra = 16,
    }
    
    /// <summary>
    /// >> 49 - Variant[pallet_staking.pallet.pallet.Event]
    /// The `Event` enum of this pallet
    /// </summary>
    public sealed class EnumEvent : BaseEnumRust<Event>
    {
        
        /// <summary>
        /// Initializes a new instance of the class.
        /// </summary>
        public EnumEvent()
        {
				AddTypeDecoder<BaseTuple<Substrate.NetApi.Model.Types.Primitive.U32, Substrate.NetApi.Model.Types.Primitive.U128, Substrate.NetApi.Model.Types.Primitive.U128>>(Event.EraPaid);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.Gear.Api.Generated.Model.pallet_staking.EnumRewardDestination, Substrate.NetApi.Model.Types.Primitive.U128>>(Event.Rewarded);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.NetApi.Model.Types.Primitive.U128>>(Event.Slashed);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.Gear.Api.Generated.Model.sp_arithmetic.per_things.Perbill, Substrate.NetApi.Model.Types.Primitive.U32>>(Event.SlashReported);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Primitive.U32>(Event.OldSlashingReportDiscarded);
				AddTypeDecoder<BaseVoid>(Event.StakersElected);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.NetApi.Model.Types.Primitive.U128>>(Event.Bonded);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.NetApi.Model.Types.Primitive.U128>>(Event.Unbonded);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.NetApi.Model.Types.Primitive.U128>>(Event.Withdrawn);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32>>(Event.Kicked);
				AddTypeDecoder<BaseVoid>(Event.StakingElectionFailed);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32>(Event.Chilled);
				AddTypeDecoder<BaseTuple<Substrate.NetApi.Model.Types.Primitive.U32, Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32>>(Event.PayoutStarted);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.Gear.Api.Generated.Model.pallet_staking.ValidatorPrefs>>(Event.ValidatorPrefsSet);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Primitive.U32>(Event.SnapshotVotersSizeExceeded);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Primitive.U32>(Event.SnapshotTargetsSizeExceeded);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.pallet_staking.EnumForcing>(Event.ForceEra);
        }
    }
}