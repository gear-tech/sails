using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// NonZeroU256
/// </summary>
public sealed class NonZeroU256 : BaseNonZero<U256>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU256);
}
