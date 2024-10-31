using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public interface IQuery<T> : IActionBuilder<IQuery<T>> where T : IType
{
    /// <summary>
    /// Queries a program for information.
    /// </summary>
    /// <param name="programId">Program identifier.</param>
    /// <param name="cancellationToken">Propagates notification that operations should be canceled. <see cref="CancellationToken"/> </param>
    /// <returns></returns>
    Task<T> QueryAsync(
        ActorId programId,
        CancellationToken cancellationToken);
}
