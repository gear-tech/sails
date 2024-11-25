using System.Linq;
using EnsureThat;
using Substrate.NetApi;

namespace Substrate.Gear.Client.GearApi.Model.gprimitives;

public static class ActorIdExtensions
{
    public static string ToHexString(this Api.Generated.Model.gprimitives.ActorId actorId)
    {
        EnsureArg.IsNotNull(actorId, nameof(actorId));
        EnsureArg.IsNotNull(actorId.Value, "actorId.Value");
        EnsureArg.IsNotNull(actorId.Value.Value, "actorId.Value.Value");

        return Utils.Bytes2HexString(actorId.Value.Value.Select(u8 => u8.Value).ToArray());
    }
}
