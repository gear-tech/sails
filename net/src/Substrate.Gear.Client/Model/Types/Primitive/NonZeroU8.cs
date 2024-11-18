using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// NonZeroU8
/// </summary>
public sealed class NonZeroU8 : BaseNonZero<U8>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU8);
}
