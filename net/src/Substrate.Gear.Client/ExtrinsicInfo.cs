using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client;

public sealed record ExtrinsicInfo
{
    /// <summary>
    /// Hash of block in which extrinsic was included.
    /// </summary>
    public required Hash BlockHash { get; init; }
    /// <summary>
    /// Index of extrinsic in block.
    /// </summary>
    public uint IndexInBlock { get; init; }
    /// <summary>
    /// Hash of extrinsic itself.
    /// </summary>
    public required Hash Hash { get; init; }
}
