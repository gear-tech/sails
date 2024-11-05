using System.Collections.Generic;
using System.Threading;

namespace Sails.Remoting.Abstractions.Core;

public interface IRemotingListener
{
    /// <summary>
    /// Listen to Gear events
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    IAsyncEnumerable<byte[]> ListenAsync(CancellationToken cancellationToken);
}
