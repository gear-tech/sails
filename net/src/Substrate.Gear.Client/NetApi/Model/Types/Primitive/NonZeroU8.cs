using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.NetApi.Model.Types.Primitive;

/// <summary>
/// NonZeroU8
/// </summary>
public sealed class NonZeroU8 : BaseNonZero<U8>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU8);
}
