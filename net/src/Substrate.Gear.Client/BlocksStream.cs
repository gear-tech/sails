using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Channels;
using System.Threading.Tasks;
using EnsureThat;
using Substrate.NetApi;
using Substrate.NetApi.Model.Rpc;

namespace Substrate.Gear.Client;

public sealed class BlocksStream : IAsyncDisposable
{
    internal static async Task<BlocksStream> CreateAsync(
        SubstrateClient nodeClient,
        Func<SubstrateClient, Action<string, Header>, Task<string>> subscribe,
        Func<SubstrateClient, string, Task> unsubscribe)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(subscribe, nameof(subscribe));
        EnsureArg.IsNotNull(unsubscribe, nameof(unsubscribe));

        var channel = Channel.CreateUnbounded<Header>(
            new UnboundedChannelOptions
            {
                SingleReader = true
            });

        var subscriptionId = await subscribe(
                nodeClient,
                (_, blockHeader) => channel.Writer.TryWrite(blockHeader))
            .ConfigureAwait(false);

        return new BlocksStream(
            channel,
            () => unsubscribe(nodeClient, subscriptionId));
    }

    private BlocksStream(Channel<Header> channel, Func<Task> unsubscribe)
    {
        this.channel = channel;
        this.unsubscribe = unsubscribe;
        this.isReadInProgress = 0;
    }

    private readonly Channel<Header> channel;
    private readonly Func<Task> unsubscribe;
    private int isReadInProgress;

    public async ValueTask DisposeAsync()
    {
        await this.unsubscribe().ConfigureAwait(false);
        this.channel.Writer.Complete();

        GC.SuppressFinalize(this);
    }

    /// <summary>
    /// Returns all block headers since the stream was created or the last call to this method.
    /// Only one read operation is allowed at a time.
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public IAsyncEnumerable<Header> ReadAllHeadersAsync(CancellationToken cancellationToken)
    {
        return Interlocked.CompareExchange(ref this.isReadInProgress, 1, 0) == 0
            ? ReadAllImpl(cancellationToken)
            : throw new InvalidOperationException("TODO: Custom exception. Only one read operation is allowed at a time.");

        async IAsyncEnumerable<Header> ReadAllImpl([EnumeratorCancellation] CancellationToken cancellationToken)
        {
            try
            {
                while (true)
                {
                    yield return await this.channel.Reader.ReadAsync(cancellationToken).ConfigureAwait(false);
                }
            }
            finally
            {
                Interlocked.Exchange(ref this.isReadInProgress, 0);
            }
        }
    }
}
