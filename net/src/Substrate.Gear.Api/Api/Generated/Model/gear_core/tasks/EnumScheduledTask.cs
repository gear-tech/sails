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


namespace Substrate.Gear.Api.Generated.Model.gear_core.tasks
{
    
    
    /// <summary>
    /// >> ScheduledTask
    /// </summary>
    public enum ScheduledTask
    {
        
        /// <summary>
        /// >> PauseProgram
        /// </summary>
        PauseProgram = 0,
        
        /// <summary>
        /// >> RemoveCode
        /// </summary>
        RemoveCode = 1,
        
        /// <summary>
        /// >> RemoveFromMailbox
        /// </summary>
        RemoveFromMailbox = 2,
        
        /// <summary>
        /// >> RemoveFromWaitlist
        /// </summary>
        RemoveFromWaitlist = 3,
        
        /// <summary>
        /// >> RemovePausedProgram
        /// </summary>
        RemovePausedProgram = 4,
        
        /// <summary>
        /// >> WakeMessage
        /// </summary>
        WakeMessage = 5,
        
        /// <summary>
        /// >> SendDispatch
        /// </summary>
        SendDispatch = 6,
        
        /// <summary>
        /// >> SendUserMessage
        /// </summary>
        SendUserMessage = 7,
        
        /// <summary>
        /// >> RemoveGasReservation
        /// </summary>
        RemoveGasReservation = 8,
        
        /// <summary>
        /// >> RemoveResumeSession
        /// </summary>
        RemoveResumeSession = 9,
    }
    
    /// <summary>
    /// >> 609 - Variant[gear_core.tasks.ScheduledTask]
    /// </summary>
    public sealed class EnumScheduledTask : BaseEnumRust<ScheduledTask>
    {
        
        /// <summary>
        /// Initializes a new instance of the class.
        /// </summary>
        public EnumScheduledTask()
        {
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gprimitives.ActorId>(ScheduledTask.PauseProgram);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gprimitives.CodeId>(ScheduledTask.RemoveCode);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32, Substrate.Gear.Api.Generated.Model.gprimitives.MessageId>>(ScheduledTask.RemoveFromMailbox);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.gprimitives.ActorId, Substrate.Gear.Api.Generated.Model.gprimitives.MessageId>>(ScheduledTask.RemoveFromWaitlist);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gprimitives.ActorId>(ScheduledTask.RemovePausedProgram);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.gprimitives.ActorId, Substrate.Gear.Api.Generated.Model.gprimitives.MessageId>>(ScheduledTask.WakeMessage);
				AddTypeDecoder<Substrate.Gear.Api.Generated.Model.gprimitives.MessageId>(ScheduledTask.SendDispatch);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.gprimitives.MessageId, Substrate.NetApi.Model.Types.Primitive.Bool>>(ScheduledTask.SendUserMessage);
				AddTypeDecoder<BaseTuple<Substrate.Gear.Api.Generated.Model.gprimitives.ActorId, Substrate.Gear.Api.Generated.Model.gprimitives.ReservationId>>(ScheduledTask.RemoveGasReservation);
				AddTypeDecoder<Substrate.NetApi.Model.Types.Primitive.U32>(ScheduledTask.RemoveResumeSession);
        }
    }
}