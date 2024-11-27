using System;

namespace Sails.Remoting.Options;

public sealed record NodeClientOptions
{
    public Uri? GearNodeUri { get; set; }
}
