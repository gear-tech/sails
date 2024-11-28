using System.Collections.Generic;
using System.Linq;
using System.Threading;
using EnsureThat;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Client.NetApi.Model.Rpc;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client;

public static class BlocksStreamExtensions
{
    public static IAsyncEnumerable<BaseEnumRust<RuntimeEvent>> ReadAllRuntimeEventsAsync(
        this BlocksStream blocksStream,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(blocksStream, nameof(blocksStream));

        var eventRecords = blocksStream
            .ReadAllAsync(
                selectAsync: async (nodeClient, blockHeader)
                    => await nodeClient.ListBlockEventsAsync(
                            blockHeader.GetBlockHash(),
                            cancellationToken)
                        .ConfigureAwait(false),
                cancellationToken);
        return eventRecords.SelectMany(eventRecords => eventRecords.ToAsyncEnumerable())
            .Select(eventRecord => eventRecord.Event.ToBaseEnumRust());
    }

    public static IAsyncEnumerable<BaseEnumRust<GearEvent>> ReadAllGearRuntimeEventsAsync(
        this BlocksStream blocksStream,
        CancellationToken cancellationToken)
        => blocksStream
            .ReadAllRuntimeEventsAsync(cancellationToken)
            .SelectIfMatches(
                RuntimeEvent.Gear,
                (EnumGearEvent gearEvent) => gearEvent.ToBaseEnumRust());
}
