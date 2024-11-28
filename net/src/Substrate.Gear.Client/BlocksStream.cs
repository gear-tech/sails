using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Channels;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.frame_system;
using Substrate.Gear.Client.NetApi.Model.Rpc;
using Substrate.NetApi.Model.Rpc;

namespace Substrate.Gear.Client;

public sealed class BlocksStream : IAsyncDisposable
{
    internal BlocksStream(
        SubstrateClientExt nodeClient,
        string subscriptionId,
        Channel<Header> channel,
        Func<SubstrateClientExt, string, Task> unsubscribe)
    {
        this.nodeClient = nodeClient;
        this.subscriptionId = subscriptionId;
        this.channel = channel;
        this.unsubscribe = unsubscribe;
        this.isReadInProgress = 0;
    }

    private readonly SubstrateClientExt nodeClient;
    private readonly string subscriptionId;
    private readonly Channel<Header> channel;
    private readonly Func<SubstrateClientExt, string, Task> unsubscribe;
    private int isReadInProgress;

    public async ValueTask DisposeAsync()
    {
        await this.unsubscribe(this.nodeClient, this.subscriptionId).ConfigureAwait(false);
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
                await foreach (var blockHeader in this.channel.Reader.ReadAllAsync(cancellationToken).ConfigureAwait(false))
                {
                    yield return blockHeader;
                }
            }
            finally
            {
                Interlocked.Exchange(ref this.isReadInProgress, 0);
            }
        }
    }

    /// <summary>
    /// Returns all events from blocks since the stream was created or the last call to this method.
    /// Only one read operation is allowed at a time.
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public IAsyncEnumerable<EventRecord> ReadAllEventsAsync(CancellationToken cancellationToken)
    {
        return Interlocked.CompareExchange(ref this.isReadInProgress, 1, 0) == 0
            ? ReadAllImpl(cancellationToken)
            : throw new InvalidOperationException("TODO: Custom exception. Only one read operation is allowed at a time.");

        async IAsyncEnumerable<EventRecord> ReadAllImpl([EnumeratorCancellation] CancellationToken cancellationToken)
        {
            try
            {
                await foreach (var blockHeader in this.channel.Reader.ReadAllAsync(cancellationToken).ConfigureAwait(false))
                {
                    var blockEvents = await this.nodeClient
                        .ListBlockEventsAsync(blockHeader.GetBlockHash(), cancellationToken)
                        .ConfigureAwait(false);
                    foreach (var blockEvent in blockEvents)
                    {
                        yield return blockEvent;
                    }
                }
            }
            finally
            {
                Interlocked.Exchange(ref this.isReadInProgress, 0);
            }
        }
    }
}
