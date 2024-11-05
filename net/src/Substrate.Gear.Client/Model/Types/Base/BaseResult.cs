using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.Model.Types.Base;

/// <summary>
/// Result
/// </summary>
public enum BaseResultEnum
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
public sealed class BaseResult<T1, T2> : BaseEnumRust<BaseResultEnum>
    where T1 : IType, new()
    where T2 : IType, new()
{

    /// <summary>
    /// Initializes a new instance of the class.
    /// </summary>
    public BaseResult()
    {
        this.AddTypeDecoder<T1>(BaseResultEnum.Ok);
        this.AddTypeDecoder<T2>(BaseResultEnum.Err);
    }
}
