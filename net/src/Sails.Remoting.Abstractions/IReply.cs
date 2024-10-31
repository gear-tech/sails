using System.Threading;
using System.Threading.Tasks;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public interface IReply<T> where T : IType, new()
{
    /// <summary>
    /// Receive reply for a message from a program
    /// </summary>
    /// <param name="cancellationToken">Propagates notification that operations should be canceled. <see cref="CancellationToken"/> </param>
    /// <returns></returns>
    Task<T> ReceiveAsync(CancellationToken cancellationToken);
}
