using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.NetApi.Model.Types.Primitive;

/// <summary>
/// NonZeroU256
/// </summary>
public sealed class NonZeroU256 : BaseNonZero<U256>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU256);
}
