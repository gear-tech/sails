#nullable disable
//------------------------------------------------------------------------------
// <auto-generated>
//     This code was generated by a tool.
//
//     Changes to this file may cause incorrect behavior and will be lost if
//     the code is regenerated.
// </auto-generated>
//------------------------------------------------------------------------------

using Substrate.NetApi.Attributes;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Metadata.Base;
using System.Collections.Generic;


namespace Substrate.Gear.Api.Generated.Model.gear_core.message.stored
{
    
    
    /// <summary>
    /// >> 611 - Composite[gear_core.message.stored.StoredDelayedDispatch]
    /// </summary>
    [SubstrateNodeType(TypeDefEnum.Composite)]
    public sealed class StoredDelayedDispatch : BaseType
    {
        
        /// <summary>
        /// >> kind
        /// </summary>
        public Substrate.Gear.Api.Generated.Model.gear_core.message.EnumDispatchKind Kind { get; set; }
        /// <summary>
        /// >> message
        /// </summary>
        public Substrate.Gear.Api.Generated.Model.gear_core.message.stored.StoredMessage Message { get; set; }
        
        /// <inheritdoc/>
        public override string TypeName()
        {
            return "StoredDelayedDispatch";
        }
        
        /// <inheritdoc/>
        public override byte[] Encode()
        {
            var result = new List<byte>();
            result.AddRange(Kind.Encode());
            result.AddRange(Message.Encode());
            return result.ToArray();
        }
        
        /// <inheritdoc/>
        public override void Decode(byte[] byteArray, ref int p)
        {
            var start = p;
            Kind = new Substrate.Gear.Api.Generated.Model.gear_core.message.EnumDispatchKind();
            Kind.Decode(byteArray, ref p);
            Message = new Substrate.Gear.Api.Generated.Model.gear_core.message.stored.StoredMessage();
            Message.Decode(byteArray, ref p);
            var bytesLength = p - start;
            TypeSize = bytesLength;
            Bytes = new byte[bytesLength];
            global::System.Array.Copy(byteArray, start, Bytes, 0, bytesLength);
        }
    }
}
