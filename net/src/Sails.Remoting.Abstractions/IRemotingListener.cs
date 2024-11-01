using System.Collections.Generic;
using System.Threading;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public interface IRemotingListener
{
    /// <summary>
    /// Listen to Gear events
    /// </summary>
    /// <param name="cancellationToken">Propagates notification that operations should be canceled. <see cref="CancellationToken"/> </param>
    /// <returns></returns>
    IAsyncEnumerable<byte[]> ListenAsync(CancellationToken cancellationToken);
}

public interface IRemotingListener<T> where T : IType, new()
{
    /// <summary>
    /// Listen to Service events
    /// </summary>
    /// <param name="cancellationToken">Propagates notification that operations should be canceled. <see cref="CancellationToken"/> </param>
    /// <returns></returns>
    IAsyncEnumerable<T> ListenAsync(CancellationToken cancellationToken);
}
