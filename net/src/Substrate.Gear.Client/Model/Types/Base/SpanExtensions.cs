using System;

namespace Substrate.Gear.Client.Model.Types.Base;

internal static class SpanExtensions
{

    /// <summary>
    /// Retunr true if all bytes is zero
    /// </summary>
    /// <param name="bytes"></param>
    /// <returns></returns>
    public static bool IsZero(this Span<byte> bytes)
    {
        byte sum = 0;
        foreach (var b in bytes)
        {
            sum |= b;
        }
        return sum == 0;
    }
}
