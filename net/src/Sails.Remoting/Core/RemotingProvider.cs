using System;
using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Core;

internal sealed class RemotingProvider : IRemotingProvider
{
    public RemotingProvider(Func<Account, IRemoting> remotingFactory, Func<IRemotingListener> remotingListenerFactory)
    {
        EnsureArg.IsNotNull(remotingFactory, nameof(remotingFactory));
        EnsureArg.IsNotNull(remotingListenerFactory, nameof(remotingListenerFactory));

        this.remotingFactory = remotingFactory;
        this.remotingListenerFactory = remotingListenerFactory;
    }

    private readonly Func<Account, IRemoting> remotingFactory;
    private readonly Func<IRemotingListener> remotingListenerFactory;

    /// <inheritdoc/>
    public IRemoting CreateRemoting(Account signingAccount)
    {
        EnsureArg.IsNotNull(signingAccount, nameof(signingAccount));

        return this.remotingFactory(signingAccount);
    }

    public IRemotingListener CreateRemotingListener() => this.remotingListenerFactory();
}
