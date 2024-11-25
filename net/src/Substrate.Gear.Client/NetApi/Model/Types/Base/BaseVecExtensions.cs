using System.Collections.Generic;
using Substrate.Gear.Client.NetApi.Model.Types.Primitive;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.NetApi.Model.Types.Base;

public static class BaseVecExtensions
{
    public static BaseVec<U8> ToBaseVecOfU8(this IReadOnlyCollection<byte> bytes)
       => new(bytes.ToArrayOfU8());
}
