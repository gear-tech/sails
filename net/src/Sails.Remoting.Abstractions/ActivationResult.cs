using System;
using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;

namespace Sails.Remoting.Abstractions;

/// <summary>
/// Represents result returned from the <see cref="IRemoting.ActivateAsync" /> method.
/// </summary>
public abstract class ActivationResult : IAsyncDisposable
{
    public async ValueTask DisposeAsync()
    {
        await this.DisposeCoreAsync().ConfigureAwait(false);

        GC.SuppressFinalize(this);
    }

    /// <summary>
    /// Reads ProgramId of the activated program and SCALE-encoded reply
    /// from its activation method.
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public abstract Task<(ActorId ProgramId, byte[] EncodedPayload)> ReadReplyAsync(CancellationToken cancellationToken);

    protected virtual ValueTask DisposeCoreAsync()
        => new();
}
