using System.Collections.Generic;
using System.Threading;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public interface IRemotingListener<T> where T : IType, new()
{
    /// <summary>
    /// Listen to Service events
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    IAsyncEnumerable<(ActorId, T)> ListenAsync(CancellationToken cancellationToken);
}
