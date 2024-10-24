using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;

namespace Substrate.Gear.Client;

public sealed record GasInfo
{
    /// <summary>
    /// Represents minimum gas limit required for execution.
    /// </summary>
    public required GasUnit MinLimit { get; init; }
    /// <summary>
    /// Gas amount that we reserve for some other on-chain interactions.
    /// </summary>
    public required GasUnit Reserved { get; init; }
    /// <summary>
    /// Contains number of gas burned during message processing.
    /// </summary>
    public required GasUnit Burned { get; init; }
    /// <summary>
    /// The value may be returned if a program happens to be executed
    /// the second or next time in a block.
    /// </summary>
    public required GasUnit MayBeReturned { get; init; }
    /// <summary>
    /// Was the message placed into waitlist at the end of calculating.
    /// This flag shows, that `min_limit` makes sense and have some guarantees
    /// only before insertion into waitlist.
    /// </summary>
    public bool IsInWaitList { get; init; }
}
