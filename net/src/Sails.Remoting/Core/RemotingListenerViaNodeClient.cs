using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
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

internal sealed class RemotingListenerViaNodeClient : IRemotingListener, IAsyncDisposable
{
    public RemotingListenerViaNodeClient(INodeClientProvider nodeClientProvider)
    {
        EnsureArg.IsNotNull(nodeClientProvider, nameof(nodeClientProvider));

        this.nodeClientProvider = nodeClientProvider;
    }

    private readonly INodeClientProvider nodeClientProvider;
    private Substrate.Gear.Api.Generated.SubstrateClientExt? nodeClient;
    private BlocksStream? blocksStream;

    public async Task<IAsyncEnumerable<(ActorId Source, byte[] Payload)>> ListenAsync(CancellationToken cancellationToken)
    {
        this.nodeClient ??= await this.nodeClientProvider.GetNodeClientAsync(cancellationToken).ConfigureAwait(false);
        this.blocksStream ??= await this.nodeClient.GetNewBlocksStreamAsync(cancellationToken).ConfigureAwait(false);

        return this.blocksStream.ReadAllHeadersAsync(cancellationToken)
            .SelectAwait(
                async blockHeader =>
                    await this.nodeClient.ListBlockEventsAsync(blockHeader.GetBlockHash(), cancellationToken)
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
            .Select(userMessage => (userMessage.Source, userMessage.Payload.Value.Value.Select(@byte => @byte.Value).ToArray()));
    }

    public async ValueTask DisposeAsync()
    {
        var bs = Interlocked.Exchange(ref this.blocksStream, null);
        if (bs is not null)
        {
            await bs.DisposeAsync().ConfigureAwait(false);
        }
        var nc = Interlocked.Exchange(ref this.nodeClient, null);
        nc?.Dispose();
    }
}
