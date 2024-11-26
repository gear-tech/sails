using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions.Core;

public interface IRemotingProvider
{
    /// <summary>
    /// Creates an instance implementing the <see cref="IRemoting"/> interface
    /// with initial account for signing transactions.
    /// </summary>
    /// <param name="signingAccount"></param>
    /// <returns></returns>
    IRemoting CreateRemoting(Account signingAccount);

    /// <summary>
    /// Creates an instance implementing the <see cref="IRemotingListener"/> interface
    /// </summary>
    /// <returns></returns>
    IRemotingListener CreateRemotingListener();
}
