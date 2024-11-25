using System;
using System.Linq;
using EnsureThat;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi;
using Substrate.NetApi.Model.Rpc;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.NetApi.Model.Rpc;

public static class HeaderExtensions
{
    public static Hash GetBlockHash(this Header header)
    {
        EnsureArg.IsNotNull(header, nameof(header));

        var parentHashBytes = header.ParentHash.AsBytesSpan();
        var numberBytes = new CompactInteger(header.Number).Encode();
        var stateRootBytes = header.StateRoot.AsBytesSpan();
        var extrinsicsRootBytes = header.ExtrinsicsRoot.AsBytesSpan();
        var logsCountBytes = new CompactInteger(header.Digest.Logs.Count).Encode();
        var logsBytesLength = 0;
        var logsBytes = header.Digest.Logs
            .Select(log =>
                {
                    var logBytes = Utils.HexToByteArray(log);
                    logsBytesLength += logBytes.Length;
                    return logBytes;
                })
            .ToArray();

        var bytesToHash = new byte[
            parentHashBytes.Length
            + numberBytes.Length
            + stateRootBytes.Length
            + extrinsicsRootBytes.Length
            + logsCountBytes.Length
            + logsBytesLength];

        var copyAt = 0;

        parentHashBytes.CopyTo(bytesToHash.AsSpan(copyAt));
        copyAt += parentHashBytes.Length;

        numberBytes.CopyTo(bytesToHash.AsSpan(copyAt));
        copyAt += numberBytes.Length;

        stateRootBytes.CopyTo(bytesToHash.AsSpan(copyAt));
        copyAt += stateRootBytes.Length;

        extrinsicsRootBytes.CopyTo(bytesToHash.AsSpan(copyAt));
        copyAt += extrinsicsRootBytes.Length;

        logsCountBytes.CopyTo(bytesToHash.AsSpan(copyAt));
        copyAt += logsCountBytes.Length;

        foreach (var logBytes in logsBytes)
        {
            logBytes.CopyTo(bytesToHash.AsSpan(copyAt));
            copyAt += logBytes.Length;
        }

        return new Hash(HashExtension.Blake2(bytesToHash, 256));
    }
}
