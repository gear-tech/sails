using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public interface IRemotingProvider
{
    /// <summary>
    /// Creates an instance implementing the <see cref="IRemoting"/> interface
    /// with initial account for signing transactions.
    /// </summary>
    /// <param name="signingAccount"></param>
    /// <returns></returns>
    IRemoting CreateRemoting(Account signingAccount);
}
