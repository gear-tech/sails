using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Attributes;
using Substrate.NetApi.Model.Types.Metadata.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// NonZeroU16
/// </summary>
[SubstrateNodeType(TypeDefEnum.Composite)]
public sealed class NonZeroU16 : BaseNonZero<U16>
{
    /// <inheritdoc/>
    public override string TypeName() => nameof(NonZeroU16);
}
