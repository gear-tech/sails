using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.NetApi.Model.Types.Primitive;

/// <summary>
/// NonZeroU16
/// </summary>
public sealed class NonZeroU16 : BaseNonZero<U16>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU16);
}
