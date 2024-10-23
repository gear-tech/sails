using System;

namespace Sails.Remoting.Options;

public sealed record RemotingViaSubstrateClientOptions
{
    public Uri? GearNodeUri { get; init; }
}
