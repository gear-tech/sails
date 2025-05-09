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


namespace Substrate.Gear.Api.Generated.Model.pallet_gear_scheduler.pallet
{
    
    
    /// <summary>
    /// >> Error
    /// The `Error` enum of this pallet.
    /// </summary>
    public enum Error
    {
        
        /// <summary>
        /// >> DuplicateTask
        /// Occurs when given task already exists in task pool.
        /// </summary>
        DuplicateTask = 0,
        
        /// <summary>
        /// >> TaskNotFound
        /// Occurs when task wasn't found in storage.
        /// </summary>
        TaskNotFound = 1,
    }
    
    /// <summary>
    /// >> 615 - Variant[pallet_gear_scheduler.pallet.Error]
    /// The `Error` enum of this pallet.
    /// </summary>
    public sealed class EnumError : BaseEnum<Error>
    {
    }
}
