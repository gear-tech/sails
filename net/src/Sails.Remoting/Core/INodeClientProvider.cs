using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated;

namespace Sails.Remoting.Core;

internal interface INodeClientProvider
{
    /// <summary>
    /// Returns connected node client.
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    Task<SubstrateClientExt> GetNodeClientAsync(CancellationToken cancellationToken);
}
