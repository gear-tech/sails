﻿using System;
using Substrate.Gear.Client.Model.Types.Primitive;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client.Model.Types.Base;

public static class BaseVecExtensions
{
    public static BaseVec<U8> ToBaseVecOfU8(this ReadOnlyMemory<byte> bytes)
        => new(bytes.ToArrayOfU8());
}