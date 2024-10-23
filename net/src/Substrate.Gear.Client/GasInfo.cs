using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;

namespace Substrate.Gear.Client;

public sealed record GasInfo
{
    /// Represents minimum gas limit required for execution.
    public required GasUnit MinLimit { get; init; }
    /// Gas amount that we reserve for some other on-chain interactions.
    public required GasUnit Reserved { get; init; }
    /// Contains number of gas burned during message processing.
    public required GasUnit Burned { get; init; }
    /// The value may be returned if a program happens to be executed
    /// the second or next time in a block.
    public required GasUnit MayBeReturned { get; init; }
    /// Was the message placed into waitlist at the end of calculating.
    ///
    /// This flag shows, that `min_limit` makes sense and have some guarantees
    /// only before insertion into waitlist.
    public bool IsInWaitList { get; init; }
}
