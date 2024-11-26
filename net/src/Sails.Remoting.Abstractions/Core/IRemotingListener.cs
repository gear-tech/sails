using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;

namespace Sails.Remoting.Abstractions.Core;

public interface IRemotingListener
{
    /// <summary>
    /// Asynchronously subscribe to Gear events
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    Task<IAsyncEnumerable<(ActorId Source, byte[] Payload)>> ListenAsync(CancellationToken cancellationToken);
}
