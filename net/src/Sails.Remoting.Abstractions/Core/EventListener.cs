using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Sails.Remoting.Abstractions.Core;

public abstract class EventListener<T> : IAsyncEnumerable<T>, IAsyncDisposable
{
    public async ValueTask DisposeAsync()
    {
        await this.DisposeCoreAsync().ConfigureAwait(false);

        GC.SuppressFinalize(false);
    }

    protected abstract ValueTask DisposeCoreAsync();

    public abstract IAsyncEnumerator<T> GetAsyncEnumerator(CancellationToken cancellationToken = default);
}
