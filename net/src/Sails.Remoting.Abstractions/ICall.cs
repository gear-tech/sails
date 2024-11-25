using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public interface ICall<T> : IActionBuilder<ICall<T>> where T : IType, new()
{
    /// <summary>
    /// Sends a message to a program for execution.
    /// </summary>
    /// <param name="programId">Program identifier.</param>
    /// <param name="cancellationToken"></param>
    /// <returns>Reply <see cref="IReply{T}"/></returns>
    Task<IReply<T>> MessageAsync(
        ActorId programId,
        CancellationToken cancellationToken);
}
