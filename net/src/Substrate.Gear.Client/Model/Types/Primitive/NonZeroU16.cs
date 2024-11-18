using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// NonZeroU16
/// </summary>
public sealed class NonZeroU16 : BaseNonZero<U16>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU16);
}
