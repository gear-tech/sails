using System;
using System.Linq;
using EnsureThat;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.NetApi.Model.Types.Base;

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

        return left.AsBytesSpan()
            .SequenceEqual(right.AsBytesSpan());
    }

    /// <summary>
    /// Returns the exact span of bytes that represents the primitive type.
    /// It is requied because the BaseType.Bytes property returns the whole array
    /// which can be larger than the actual size of the primitive type.
    /// </summary>
    /// <param name="baseType"></param>
    /// <returns></returns>
    public static Span<byte> AsBytesSpan(this BaseType baseType)
    {
        EnsureArg.IsNotNull(baseType, nameof(baseType));

        return baseType.Bytes.AsSpan(0, baseType.TypeSize);
    }
}
