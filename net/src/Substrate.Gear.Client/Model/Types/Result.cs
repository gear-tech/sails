using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.Model.Types;

/// <summary>
/// Result
/// </summary>
public enum Result
{

    /// <summary>
    /// >> Ok
    /// </summary>
    Ok = 0,

    /// <summary>
    /// >> Err
    /// </summary>
    Err = 1,
}

/// <summary>
/// EnumResult
/// </summary>
public sealed class EnumResult<T1, T2> : BaseEnumRust<Result>
    where T1 : IType, new() where T2 : IType, new()
{

    /// <summary>
    /// Initializes a new instance of the class.
    /// </summary>
    public EnumResult()
    {
        this.AddTypeDecoder<T1>(Result.Ok);
        this.AddTypeDecoder<T2>(Result.Err);
    }
}
