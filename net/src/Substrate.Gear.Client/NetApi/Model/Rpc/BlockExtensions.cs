using System;
using EnsureThat;
using Substrate.Gear.Client.NetApi.Model.Extrinsics;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Rpc;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.NetApi.Model.Rpc;

public static class BlockExtensions
{
    /// <summary>
    /// Returns the index of the extrinsic in the block.
    /// </summary>
    /// <param name="block"></param>
    /// <param name="extrinsicHash"></param>
    /// <returns></returns>
    /// <exception cref="Exception"></exception>
    public static uint GetExtrinsicIdxByHash(this Block block, Hash extrinsicHash)
    {
        EnsureArg.IsNotNull(block, nameof(block));
        EnsureArg.IsNotNull(extrinsicHash, nameof(extrinsicHash));

        return block.FindExtrinsicIdxByHash(extrinsicHash)
            ?? throw new Exception("TODO: Custom exception.");
    }

    /// <summary>
    /// Returns the index of the extrinsic in the block if found, otherwise null.
    /// </summary>
    /// <param name="block"></param>
    /// <param name="extrinsicHash"></param>
    /// <returns></returns>
    public static uint? FindExtrinsicIdxByHash(this Block block, Hash extrinsicHash)
    {
        for (var i = 0u; i < block.Extrinsics.Length; i++)
        {
            var extrinsic = block.Extrinsics[i];
            var (_, extrinsicHashCalculated) = extrinsic.EncodeAndHash();
            if (extrinsicHashCalculated.IsEqualTo(extrinsicHash))
            {
                return i;
            }
        }
        return null;
    }
}
