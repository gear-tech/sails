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


namespace Substrate.Gear.Api.Generated.Model.pallet_bounties.pallet
{
    
    
    /// <summary>
    /// >> Call
    /// Contains a variant per dispatchable extrinsic that this pallet has.
    /// </summary>
    public enum Call
    {
        
        /// <summary>
        /// >> propose_bounty
        /// See [`Pallet::propose_bounty`].
        /// </summary>
        propose_bounty = 0,
        
        /// <summary>
        /// >> approve_bounty
        /// See [`Pallet::approve_bounty`].
        /// </summary>
        approve_bounty = 1,
        
        /// <summary>
        /// >> propose_curator
        /// See [`Pallet::propose_curator`].
        /// </summary>
        propose_curator = 2,
        
        /// <summary>
        /// >> unassign_curator
        /// See [`Pallet::unassign_curator`].
        /// </summary>
        unassign_curator = 3,
        
        /// <summary>
        /// >> accept_curator
        /// See [`Pallet::accept_curator`].
        /// </summary>
        accept_curator = 4,
        
        /// <summary>
        /// >> award_bounty
        /// See [`Pallet::award_bounty`].
        /// </summary>
        award_bounty = 5,
        
        /// <summary>
        /// >> claim_bounty
        /// See [`Pallet::claim_bounty`].
        /// </summary>
        claim_bounty = 6,
        
        /// <summary>
        /// >> close_bounty
        /// See [`Pallet::close_bounty`].
        /// </summary>
        close_bounty = 7,
        
        /// <summary>
        /// >> extend_bounty_expiry
        /// See [`Pallet::extend_bounty_expiry`].
        /// </summary>
        extend_bounty_expiry = 8,
    }
    
    /// <summary>
    /// >> 247 - Variant[pallet_bounties.pallet.Call]
    /// Contains a variant per dispatchable extrinsic that this pallet has.
    /// </summary>
    public sealed class EnumCall : BaseEnumRust<Call>
    {
        
        /// <summary>
        /// Initializes a new instance of the class.
        /// </summary>
        public EnumCall()
        {
				AddTypeDecoder<BaseTuple<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U128>, Substrate.NetApi.Model.Types.Base.BaseVec<Substrate.NetApi.Model.Types.Primitive.U8>>>(Call.propose_bounty);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>>(Call.approve_bounty);
				AddTypeDecoder<BaseTuple<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>, Substrate.Gear.Api.Generated.Model.sp_runtime.multiaddress.EnumMultiAddress, Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U128>>>(Call.propose_curator);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>>(Call.unassign_curator);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>>(Call.accept_curator);
				AddTypeDecoder<BaseTuple<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>, Substrate.Gear.Api.Generated.Model.sp_runtime.multiaddress.EnumMultiAddress>>(Call.award_bounty);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>>(Call.claim_bounty);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>>(Call.close_bounty);
				AddTypeDecoder<BaseTuple<Substrate.NetApi.Model.Types.Base.BaseCom<Substrate.NetApi.Model.Types.Primitive.U32>, Substrate.NetApi.Model.Types.Base.BaseVec<Substrate.NetApi.Model.Types.Primitive.U8>>>(Call.extend_bounty_expiry);
        }
    }
}