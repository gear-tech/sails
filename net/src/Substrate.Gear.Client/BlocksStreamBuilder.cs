using System;
using System.Threading;
using System.Threading.Channels;
using System.Threading.Tasks;
using EnsureThat;
using Substrate.Gear.Api.Generated;
using Substrate.NetApi.Model.Rpc;

namespace Substrate.Gear.Client;

internal sealed class BlocksStreamBuilder(SubstrateClientExt nodeClient)
{
    internal static BlocksStreamBuilder FromNode(SubstrateClientExt nodeClient)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));

        return new BlocksStreamBuilder(nodeClient);
    }

    internal async Task<BlocksStream> CreateAsync(
        Func<SubstrateClientExt, Action<string, Header>, CancellationToken, Task<string>> subscribe,
        Func<SubstrateClientExt, string, Task> unsubscribe,
        CancellationToken cancellationToken)
    {
        var channel = Channel.CreateUnbounded<Header>(
            new UnboundedChannelOptions
            {
                SingleReader = true
            });

        void Callback(string _, Header blockHeader) => channel.Writer.TryWrite(blockHeader);
        var subscriptionId = await subscribe(nodeClient, Callback, cancellationToken).ConfigureAwait(false);

        return new BlocksStream(nodeClient, subscriptionId, channel, unsubscribe);
    }
}
