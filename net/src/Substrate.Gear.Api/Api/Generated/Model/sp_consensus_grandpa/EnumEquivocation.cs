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


namespace Substrate.Gear.Api.Generated.Model.sp_consensus_grandpa
{
    
    
    /// <summary>
    /// >> Equivocation
    /// </summary>
    public enum Equivocation
    {
        
        /// <summary>
        /// >> Prevote
        /// </summary>
        Prevote = 0,
        
        /// <summary>
        /// >> Precommit
        /// </summary>
        Precommit = 1,
    }
    
    /// <summary>
    /// >> 83 - Variant[sp_consensus_grandpa.Equivocation]
    /// </summary>
    public sealed class EnumEquivocation : BaseEnumRust<Equivocation>
    {
        
        /// <summary>
        /// Initializes a new instance of the class.
        /// </summary>
        public EnumEquivocation()
        {
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.finality_grandpa.EquivocationT1>(Equivocation.Prevote);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.finality_grandpa.EquivocationT2>(Equivocation.Precommit);
        }
    }
}