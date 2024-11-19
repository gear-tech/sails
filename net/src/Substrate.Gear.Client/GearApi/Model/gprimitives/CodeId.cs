using System.Collections.Generic;
using EnsureThat;
using Substrate.Gear.Client.Model.Types.Primitive;
using Substrate.NetApi;

namespace Substrate.Gear.Client.GearApi.Model.gprimitives;

public static class CodeId
{
    /// <summary>
    /// Calculates CodeId from code bytes.
    /// </summary>
    /// <param name="codeBytes"></param>
    /// <returns></returns>
    public static Api.Generated.Model.gprimitives.CodeId FromCodeBytes(IReadOnlyCollection<byte> codeBytes)
    {
        EnsureArg.IsNotNull(codeBytes, nameof(codeBytes));

        var codeBytesArray = codeBytes as byte[] ?? [.. codeBytes];

        var codeHash = HashExtension.Blake2(codeBytesArray, 256);

        return new Api.Generated.Model.gprimitives.CodeId
        {
            Value = new Api.Generated.Types.Base.Arr32U8
            {
                Value = codeHash.ToArrayOfU8()
            }
        };
    }
}
