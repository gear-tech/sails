using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.NetApi.Model.Types.Base;

public static class BaseEnumRustExtensions
{
    /// <summary>
    /// Checks if "Rust" enum matches the specified variant and predicate.
    /// </summary>
    /// <typeparam name="TEnum"></typeparam>
    /// <typeparam name="TData"></typeparam>
    /// <param name="rustEnum"></param>
    /// <param name="variant"></param>
    /// <param name="predicate"></param>
    /// <returns></returns>
    /// <exception cref="InvalidCastException">
    ///   Thrown when TData doesn't match the actual type of the BaseRustEnum<typeparamref name="TEnum"/>.Value2.
    /// </exception>"
    public static bool Matches<TEnum, TData>(
        this BaseEnumRust<TEnum> rustEnum,
        TEnum variant,
        Predicate<TData>? predicate = null)
        where TEnum : Enum
        where TData : IType
    {
        EnsureArg.IsNotNull(rustEnum, nameof(rustEnum));

        predicate ??= _ => true;

        return rustEnum.Value.Equals(variant)
            && predicate((TData)rustEnum.Value2);
    }

    /// <summary>
    /// Projects data associated with the "Rust" enums if they matche the specified variant and predicate.
    /// </summary>
    /// <typeparam name="TEnum"></typeparam>
    /// <typeparam name="TData"></typeparam>
    /// <typeparam name="T"></typeparam>
    /// <param name="rustEnums"></param>
    /// <param name="variant"></param>
    /// <param name="predicate"></param>
    /// <param name="selector"></param>
    /// <returns></returns>
    /// <exception cref="InvalidCastException">
    ///   Thrown when TData doesn't match the actual type of the BaseRustEnum<typeparamref name="TEnum"/>.Value2.
    /// </exception>
    public static IEnumerable<T> SelectIfMatches<TEnum, TData, T>(
        this IEnumerable<BaseEnumRust<TEnum>> rustEnums,
        TEnum variant,
        Predicate<TData> predicate,
        Func<TData, T> selector)
        where TEnum : Enum
        where TData : class, IType
    {
        EnsureArg.IsNotNull(rustEnums, nameof(rustEnums));
        EnsureArg.IsNotNull(predicate, nameof(predicate));
        EnsureArg.IsNotNull(selector, nameof(selector));

        return rustEnums.Where(rustEnum => rustEnum.Matches(variant, predicate))
            .Select(rustEnum => selector((rustEnum.Value2 as TData)!));
    }

    /// <summary>
    /// Projects data associated with the "Rust" enums if they match the specified variant.
    /// </summary>
    /// <typeparam name="TEnum"></typeparam>
    /// <typeparam name="TData"></typeparam>
    /// <typeparam name="T"></typeparam>
    /// <param name="rustEnums"></param>
    /// <param name="variant"></param>
    /// <param name="selector"></param>
    /// <returns></returns>
    /// <exception cref="InvalidCastException">
    ///   Thrown when TData doesn't match the actual type of the BaseRustEnum<typeparamref name="TEnum"/>.Value2.
    /// </exception>
    public static IEnumerable<T> SelectIfMatches<TEnum, TData, T>(
        this IEnumerable<BaseEnumRust<TEnum>> rustEnums,
        TEnum variant,
        Func<TData, T> selector)
        where TEnum : Enum
        where TData : class, IType
    {
        EnsureArg.IsNotNull(rustEnums, nameof(rustEnums));
        EnsureArg.IsNotNull(selector, nameof(selector));

        return rustEnums.Where(rustEnum => rustEnum.Matches<TEnum, TData>(variant))
            .Select(rustEnum => selector((rustEnum.Value2 as TData)!));
    }

    /// <summary>
    /// Projects data associated with the "Rust" enums if they match the specified variant.
    /// </summary>
    /// <typeparam name="TEnum"></typeparam>
    /// <typeparam name="TData"></typeparam>
    /// <typeparam name="T"></typeparam>
    /// <param name="rustEnums"></param>
    /// <param name="variant"></param>
    /// <param name="selector"></param>
    /// <returns></returns>
    [SuppressMessage(
        "Style",
        "VSTHRD200:Use \"Async\" suffix for async methods",
        Justification = "To be consistent with system provided extensions")]
    public static IAsyncEnumerable<T> SelectIfMatches<TEnum, TData, T>(
        this IAsyncEnumerable<BaseEnumRust<TEnum>> rustEnums,
        TEnum variant,
        Func<TData, T> selector)
        where TEnum : Enum
        where TData : class, IType
    {
        EnsureArg.IsNotNull(rustEnums, nameof(rustEnums));
        EnsureArg.IsNotNull(selector, nameof(selector));

        return rustEnums.Where(rustEnum => rustEnum.Matches<TEnum, TData>(variant))
            .Select(rustEnum => selector((rustEnum.Value2 as TData)!));
    }

    /// <summary>
    /// Extracts data associated with the "Rust" enum.
    /// </summary>
    /// <typeparam name="TEnum"></typeparam>
    /// <typeparam name="TData"></typeparam>
    /// <param name="rustEnum"></param>
    /// <returns></returns>
    /// <exception cref="InvalidCastException">
    ///   Thrown when TData doesn't match the actual type of the BaseRustEnum<typeparamref name="TEnum"/>.Value2.
    /// </exception>
    public static TData GetData<TEnum, TData>(this BaseEnumRust<TEnum> rustEnum)
        where TEnum : Enum
        where TData : IType
    {
        EnsureArg.IsNotNull(rustEnum, nameof(rustEnum));

        return (TData)rustEnum.Value2;
    }
}
