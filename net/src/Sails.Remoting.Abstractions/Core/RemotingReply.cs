using System;
using System.Threading;
using System.Threading.Tasks;

namespace Sails.Remoting.Abstractions.Core;

/// <summary>
/// Represents reply from a remoting call.
/// </summary>
/// <typeparam name="T"></typeparam>
public abstract class RemotingReply<T> : IAsyncDisposable
{
    public async ValueTask DisposeAsync()
    {
        await this.DisposeCoreAsync().ConfigureAwait(false);

        GC.SuppressFinalize(false);
    }

    /// <summary>
    /// Reads reply data.
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public abstract Task<T> ReadAsync(CancellationToken cancellationToken);

    protected virtual ValueTask DisposeCoreAsync()
        => new();
}
