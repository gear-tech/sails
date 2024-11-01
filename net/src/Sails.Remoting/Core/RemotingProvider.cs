using System;
using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Core;

internal sealed class RemotingProvider : IRemotingProvider
{
    public RemotingProvider(Func<Account, IRemoting> remotingFactory)
    {
        EnsureArg.IsNotNull(remotingFactory, nameof(remotingFactory));

        this.remotingFactory = remotingFactory;
    }

    private readonly Func<Account, IRemoting> remotingFactory;

    /// <inheritdoc/>
    public IRemoting CreateRemoting(Account signingAccount)
    {
        EnsureArg.IsNotNull(signingAccount, nameof(signingAccount));

        return this.remotingFactory(signingAccount);
    }
}
