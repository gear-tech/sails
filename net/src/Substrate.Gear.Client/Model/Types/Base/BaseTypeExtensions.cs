using System;
using System.Linq;
using EnsureThat;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.Model.Types.Base;

public static class BaseTypeExtensions
{
    /// <summary>
    /// Compares two primitive types by their values.
    /// </summary>
    /// <param name="left"></param>
    /// <param name="right"></param>
    /// <returns></returns>
    public static bool IsEqualTo<T>(this T left, T right)
        where T : BaseType
    {
        EnsureArg.IsNotNull(left, nameof(left));
        EnsureArg.IsNotNull(right, nameof(right));
        EnsureArg.IsTrue(left.GetType() == right.GetType(), "left/right");
        EnsureArg.Is(left.TypeSize, right.TypeSize, "typeSize");

        return left.Bytes.AsSpan(0, left.TypeSize)
            .SequenceEqual(right.Bytes.AsSpan(0, right.TypeSize));
    }
}
