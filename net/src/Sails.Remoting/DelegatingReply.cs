using System;
using System.Threading;
using System.Threading.Tasks;
using Sails.Remoting.Abstractions;
using Sails.Remoting.Abstractions.Core;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting;

public sealed class DelegatingReply<TResult, T>(RemotingReply<TResult> innerReply, Func<TResult, T> map) : IReply<T>
    where T : IType, new()
{
    /// <inheritdoc />
    ValueTask IAsyncDisposable.DisposeAsync() => innerReply.DisposeAsync();

    /// <inheritdoc />
    async Task<T> IReply<T>.ReceiveAsync(CancellationToken cancellationToken)
    {
        var result = await innerReply.ReadAsync(cancellationToken).ConfigureAwait(false);
        return map(result);
    }
}
