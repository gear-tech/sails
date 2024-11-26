using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Threading;
using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using StreamJsonRpc;
using Substrate.Gear.Api.Generated.Model.gear_core.message.user;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Client;
using Substrate.Gear.Client.GearApi.Model.gprimitives;
using Substrate.Gear.Client.NetApi.Model.Rpc;
using Substrate.Gear.Client.NetApi.Model.Types.Base;

namespace Sails.Remoting.Core;

internal sealed class RemotingListenerViaNodeClient : IRemotingListener
{
    public RemotingListenerViaNodeClient(INodeClientProvider nodeClientProvider)
    {
        EnsureArg.IsNotNull(nodeClientProvider, nameof(nodeClientProvider));

        this.nodeClientProvider = nodeClientProvider;
    }

    private readonly INodeClientProvider nodeClientProvider;

    public async IAsyncEnumerable<(ActorId, byte[])> ListenAsync([EnumeratorCancellation] CancellationToken cancellationToken)
    {
        var nodeClient = await this.nodeClientProvider.GetNodeClientAsync(cancellationToken).ConfigureAwait(false);
        await using var blocksStream = await nodeClient.GetNewBlocksStreamAsync(cancellationToken).ConfigureAwait(false);

        var eventStream = blocksStream.ReadAllHeadersAsync(cancellationToken)
            .SelectAwait(
                async blockHeader =>
                    await nodeClient.ListBlockEventsAsync(blockHeader.GetBlockHash(), cancellationToken)
                        .ConfigureAwait(false))
            .SelectMany(eventRecords => eventRecords.AsAsyncEnumerable())
            .Select(eventRecord => eventRecord.Event.ToBaseEnumRust())
            .SelectIfMatches(
                RuntimeEvent.Gear,
                (EnumGearEvent gearEvent) => gearEvent.ToBaseEnumRust())
            .SelectIfMatches(
                GearEvent.UserMessageSent,
                (UserMessageSentEventData data) => (UserMessage)data.Value[0])
            .Where(userMessage => userMessage.Destination.IsEqualTo(ActorIdExtensions.Zero))
            .Select(userMessage => (userMessage.Source, userMessage.Payload.Bytes));

        await foreach (var message in eventStream)
        {
            yield return message;
        }
    }
}
