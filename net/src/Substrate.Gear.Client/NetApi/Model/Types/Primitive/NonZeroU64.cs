using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.NetApi.Model.Types.Primitive;

/// <summary>
/// NonZeroU64
/// </summary>
public sealed class NonZeroU64 : BaseNonZero<U64>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU64);
}
