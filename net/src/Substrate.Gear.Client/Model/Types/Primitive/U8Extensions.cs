using System;
using EnsureThat;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Primitive;

public static class U8Extensions
{
    public static U8[] ToArrayOfU8(this byte[] bytes)
    {
        EnsureArg.IsNotNull(bytes, nameof(bytes));
        var u8Array = new U8[bytes.Length];
        for (var i = 0; i < bytes.Length; i++)
        {
            u8Array[i] = new U8 { Value = bytes[i] };
        }
        return u8Array;
    }

    public static U8[] ToArrayOfU8(this ReadOnlyMemory<byte> bytes)
    {
        var u8s = new U8[bytes.Length];
        for (var i = 0; i < bytes.Length; i++)
        {
            u8s[i] = new U8(bytes.Span[i]);
        }
        return u8s;
    }
}
