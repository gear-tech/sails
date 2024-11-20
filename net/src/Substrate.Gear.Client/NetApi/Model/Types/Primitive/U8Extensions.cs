using System.Collections.Generic;
using EnsureThat;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.NetApi.Model.Types.Primitive;

public static class U8Extensions
{
    public static U8[] ToArrayOfU8(this IReadOnlyCollection<byte> bytes)
    {
        EnsureArg.IsNotNull(bytes, nameof(bytes));

        var u8Array = new U8[bytes.Count];
        var i = 0;
        foreach (var @byte in bytes)
        {
            u8Array[i] = new U8(@byte);
            i++;
        }
        return u8Array;
    }
}
