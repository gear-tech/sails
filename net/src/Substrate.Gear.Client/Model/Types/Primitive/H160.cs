using System;
using Substrate.Gear.Api.Generated.Types.Base;
using Substrate.NetApi.Attributes;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Metadata.Base;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// H160
/// </summary>
[SubstrateNodeType(TypeDefEnum.Composite)]
public sealed class H160 : BaseType
{
    /// <summary>
    /// >> value
    /// </summary>
    [System.Diagnostics.CodeAnalysis.AllowNull]
    public Arr20U8 Value { get; set; }

    /// <inheritdoc/>
    public override string TypeName() => nameof(H160);

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
