using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// NonZeroU128
/// </summary>
public sealed class NonZeroU128 : BaseNonZero<U128>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU128);
}
