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


namespace Substrate.Gear.Api.Generated.Model.pallet_im_online.pallet
{
    
    
    /// <summary>
    /// >> Error
    /// The `Error` enum of this pallet.
    /// </summary>
    public enum Error
    {
        
        /// <summary>
        /// >> InvalidKey
        /// Non existent public key.
        /// </summary>
        InvalidKey = 0,
        
        /// <summary>
        /// >> DuplicatedHeartbeat
        /// Duplicated heartbeat.
        /// </summary>
        DuplicatedHeartbeat = 1,
    }
    
    /// <summary>
    /// >> 419 - Variant[pallet_im_online.pallet.Error]
    /// The `Error` enum of this pallet.
    /// </summary>
    public sealed class EnumError : BaseEnum<Error>
    {
    }
}
