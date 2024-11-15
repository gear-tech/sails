#nullable disable
using System;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.Model.Types.Base;

public class BaseNonZero<T> : BaseType
    where T : BaseType, new()
{
    public static explicit operator BaseNonZero<T>(T value) => new(value);

    /// <summary>
    /// >> value
    /// </summary>
    public T Value { get; set; }

    public BaseNonZero()
    {
    }

    public BaseNonZero(T value)
    {
        var span = value.Bytes.AsSpan();
        if (span.IsZero())
        {
            throw new InvalidOperationException($"Unable to create a {this.TypeName()} instance while value is zero");
        }
        this.TypeSize = value.TypeSize;
        this.Bytes = span.ToArray();
        this.Value = value;
    }

    /// <inheritdoc/>
    public override byte[] Encode() => this.Value.Encode();

    /// <inheritdoc/>
    public override void Decode(byte[] byteArray, ref int p)
    {
        var start = p;
        this.Value = new();
        this.Value.Decode(byteArray, ref p);
        var bytesLength = p - start;
        if (byteArray.AsSpan().Slice(p, bytesLength).IsZero())
        {
            throw new InvalidOperationException($"Unable to create a {this.TypeName()} instance while value is zero");
        }
        this.TypeSize = bytesLength;
        this.Bytes = new byte[bytesLength];
        Array.Copy(byteArray, start, this.Bytes, 0, bytesLength);
    }
}
