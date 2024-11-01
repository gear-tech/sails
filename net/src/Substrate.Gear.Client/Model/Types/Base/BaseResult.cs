using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.Model.Types.Base;

/// <summary>
/// Result
/// </summary>
public enum BaseResult
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
public sealed class EnumBaseResult<T1, T2> : BaseEnumRust<BaseResult>
    where T1 : IType, new()
    where T2 : IType, new()
{

    /// <summary>
    /// Initializes a new instance of the class.
    /// </summary>
    public EnumBaseResult()
    {
        this.AddTypeDecoder<T1>(BaseResult.Ok);
        this.AddTypeDecoder<T2>(BaseResult.Err);
    }
}
