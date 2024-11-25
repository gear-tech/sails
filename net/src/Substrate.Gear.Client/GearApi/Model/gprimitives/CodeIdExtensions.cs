using System.Linq;
using EnsureThat;
using Substrate.NetApi;

namespace Substrate.Gear.Client.GearApi.Model.gprimitives;

public static class CodeIdExtensions
{
    public static string ToHexString(this Api.Generated.Model.gprimitives.CodeId codeId)
    {
        EnsureArg.IsNotNull(codeId, nameof(codeId));
        EnsureArg.IsNotNull(codeId.Value, "codeId.Value");
        EnsureArg.IsNotNull(codeId.Value.Value, "codeId.Value.Value");

        return Utils.Bytes2HexString(codeId.Value.Value.Select(u8 => u8.Value).ToArray());
    }
}
