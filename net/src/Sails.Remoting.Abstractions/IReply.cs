using System;
using System.Threading;
using System.Threading.Tasks;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public interface IReply<T> : IAsyncDisposable
    where T : IType, new()
{
    /// <summary>
    /// Receive reply for a message from a program
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    Task<T> ReceiveAsync(CancellationToken cancellationToken);
}
