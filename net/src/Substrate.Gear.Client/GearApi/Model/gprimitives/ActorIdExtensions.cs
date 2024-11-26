using System.Linq;
using EnsureThat;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi;

namespace Substrate.Gear.Client.GearApi.Model.gprimitives;

public static class ActorIdExtensions
{
    public static readonly ActorId Zero = new();

    static ActorIdExtensions()
    {
        var p = 0;
        Zero.Decode(new byte[32], ref p);
    }

    public static string ToHexString(this ActorId actorId)
    {
        EnsureArg.IsNotNull(actorId, nameof(actorId));
        EnsureArg.IsNotNull(actorId.Value, "actorId.Value");
        EnsureArg.IsNotNull(actorId.Value.Value, "actorId.Value.Value");

        return Utils.Bytes2HexString(actorId.Value.Value.Select(u8 => u8.Value).ToArray());
    }
}
