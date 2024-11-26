using System.Linq;
using EnsureThat;
using Substrate.Gear.Client.NetApi.Model.Types.Primitive;
using Substrate.NetApi;

namespace Substrate.Gear.Client.GearApi.Model.gprimitives;

public static class ActorIdExtensions
{
    public static Api.Generated.Model.gprimitives.ActorId Zero = new()
    {
        Value = new Api.Generated.Types.Base.Arr32U8
        {
            Value = new byte[32].ToArrayOfU8()
        }
    };

    public static string ToHexString(this Api.Generated.Model.gprimitives.ActorId actorId)
    {
        EnsureArg.IsNotNull(actorId, nameof(actorId));
        EnsureArg.IsNotNull(actorId.Value, "actorId.Value");
        EnsureArg.IsNotNull(actorId.Value.Value, "actorId.Value.Value");

        return Utils.Bytes2HexString(actorId.Value.Value.Select(u8 => u8.Value).ToArray());
    }
}
