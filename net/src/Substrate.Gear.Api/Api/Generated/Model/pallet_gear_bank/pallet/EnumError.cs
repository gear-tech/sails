#nullable disable
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


namespace Substrate.Gear.Api.Generated.Model.pallet_gear_bank.pallet
{
    
    
    /// <summary>
    /// >> Error
    /// The `Error` enum of this pallet.
    /// </summary>
    public enum Error
    {
        
        /// <summary>
        /// >> InsufficientBalance
        /// Insufficient user balance.
        /// </summary>
        InsufficientBalance = 0,
        
        /// <summary>
        /// >> InsufficientGasBalance
        /// Insufficient user's bank account gas balance.
        /// </summary>
        InsufficientGasBalance = 1,
        
        /// <summary>
        /// >> InsufficientValueBalance
        /// Insufficient user's bank account gas balance.
        /// </summary>
        InsufficientValueBalance = 2,
        
        /// <summary>
        /// >> InsufficientBankBalance
        /// Insufficient bank account balance.
        /// **Must be unreachable in Gear main protocol.**
        /// </summary>
        InsufficientBankBalance = 3,
        
        /// <summary>
        /// >> InsufficientDeposit
        /// Deposit of funds that will not keep bank account alive.
        /// **Must be unreachable in Gear main protocol.**
        /// </summary>
        InsufficientDeposit = 4,
        
        /// <summary>
        /// >> Overflow
        /// Overflow during funds transfer.
        /// **Must be unreachable in Gear main protocol.**
        /// </summary>
        Overflow = 5,
    }
    
    /// <summary>
    /// >> 640 - Variant[pallet_gear_bank.pallet.Error]
    /// The `Error` enum of this pallet.
    /// </summary>
    public sealed class EnumError : BaseEnum<Error>
    {
    }
}
