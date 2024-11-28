using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using System.Threading;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gear_core.message.user;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Client.NetApi.Model.Rpc;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Rpc;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client;

public static class BlocksStreamExtensions
{
    [SuppressMessage(
        "Style",
        "VSTHRD200:Use \"Async\" suffix for async methods",
        Justification = "To be consistent with system provided extensions")]
    public static IAsyncEnumerable<BaseEnumRust<GearEvent>> SelectGearEvents(
        this IAsyncEnumerable<Header> headers,
        SubstrateClientExt nodeClient,
        CancellationToken cancellationToken = default)
        => headers
            .SelectAwait(async blockHeader => await nodeClient
                .ListBlockEventsAsync(blockHeader.GetBlockHash(), cancellationToken).ConfigureAwait(false))
            .SelectMany(eventRecords => eventRecords.ToAsyncEnumerable())
            .Select(eventRecord => eventRecord.Event.ToBaseEnumRust())
            .SelectIfMatches(
                RuntimeEvent.Gear,
                (EnumGearEvent gearEvent) => gearEvent.ToBaseEnumRust()
            );

    [SuppressMessage(
        "Style",
        "VSTHRD200:Use \"Async\" suffix for async methods",
        Justification = "To be consistent with system provided extensions")]
    public static IAsyncEnumerable<(ActorId Source, byte[] Payload)> SelectServiceEvents(
        this IAsyncEnumerable<BaseEnumRust<GearEvent>> gearEvents)
        => gearEvents
            .SelectIfMatches(
                GearEvent.UserMessageSent,
                (UserMessageSentEventData data) => (UserMessage)data.Value[0])
            .Where(userMessage => userMessage.Destination
                .IsEqualTo(GearApi.Model.gprimitives.ActorIdExtensions.Zero))
            .Select(userMessage => (userMessage.Source, userMessage.Payload.Value.Value.Select(@byte => @byte.Value).ToArray()));
}
