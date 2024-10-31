using System;
using Substrate.NetApi.Attributes;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Metadata.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types;

/// <summary>
/// NonZeroU128
/// </summary>
[SubstrateNodeType(TypeDefEnum.Composite)]
public sealed class NonZeroU128 : BaseType
{

    /// <summary>
    /// >> value
    /// </summary>
    public required U128 Value { get; set; }

    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU128);

    /// <inheritdoc/>
    public override byte[] Encode() => this.Value.Encode();

    /// <inheritdoc/>
    public override void Decode(byte[] byteArray, ref int p)
    {
        var start = p;
        this.Value = new();
        this.Value.Decode(byteArray, ref p);
        var bytesLength = p - start;
        this.TypeSize = bytesLength;
        this.Bytes = new byte[bytesLength];
        Array.Copy(byteArray, start, this.Bytes, 0, bytesLength);
    }
}
