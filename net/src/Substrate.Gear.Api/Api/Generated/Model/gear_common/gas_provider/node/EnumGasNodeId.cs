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


namespace Substrate.Gear.Api.Generated.Model.gear_common.gas_provider.node
{
    
    
    /// <summary>
    /// >> GasNodeId
    /// </summary>
    public enum GasNodeId
    {
        
        /// <summary>
        /// >> Node
        /// </summary>
        Node = 0,
        
        /// <summary>
        /// >> Reservation
        /// </summary>
        Reservation = 1,
    }
    
    /// <summary>
    /// >> 331 - Variant[gear_common.gas_provider.node.GasNodeId]
    /// </summary>
    public sealed class EnumGasNodeId : BaseEnumRust<GasNodeId>
    {
        
        /// <summary>
        /// Initializes a new instance of the class.
        /// </summary>
        public EnumGasNodeId()
        {
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gprimitives.MessageId>(GasNodeId.Node);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gprimitives.ReservationId>(GasNodeId.Reservation);
        }
    }
}
