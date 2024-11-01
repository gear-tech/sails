using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Attributes;
using Substrate.NetApi.Model.Types.Metadata.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// NonZeroU64
/// </summary>
[SubstrateNodeType(TypeDefEnum.Composite)]
public sealed class NonZeroU64 : BaseNonZero<U64>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU64);
}
