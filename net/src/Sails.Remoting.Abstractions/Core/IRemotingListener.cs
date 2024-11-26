using System.Collections.Generic;
using System.Threading;
using Substrate.Gear.Api.Generated.Model.gprimitives;

namespace Sails.Remoting.Abstractions.Core;

public interface IRemotingListener
{
    /// <summary>
    /// Listen to Gear events
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    IAsyncEnumerable<(ActorId, byte[])> ListenAsync(CancellationToken cancellationToken);
}
