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


namespace Substrate.Gear.Api.Generated.Model.gear_core.message.common
{
    
    
    /// <summary>
    /// >> MessageDetails
    /// </summary>
    public enum MessageDetails
    {
        
        /// <summary>
        /// >> Reply
        /// </summary>
        Reply = 0,
        
        /// <summary>
        /// >> Signal
        /// </summary>
        Signal = 1,
    }
    
    /// <summary>
    /// >> 597 - Variant[gear_core.message.common.MessageDetails]
    /// </summary>
    public sealed class EnumMessageDetails : BaseEnumRust<MessageDetails>
    {
        
        /// <summary>
        /// Initializes a new instance of the class.
        /// </summary>
        public EnumMessageDetails()
        {
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gear_core.message.common.ReplyDetails>(MessageDetails.Reply);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gear_core.message.common.SignalDetails>(MessageDetails.Signal);
        }
    }
}
