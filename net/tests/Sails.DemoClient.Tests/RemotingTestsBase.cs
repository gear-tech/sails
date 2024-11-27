using System;
using Microsoft.Extensions.DependencyInjection;
using Sails.DemoClient.Tests._Infra.XUnit.Fixtures;
using Sails.Remoting.Abstractions.Core;
using Sails.Remoting.DependencyInjection;
using Sails.Remoting.Options;

namespace Sails.DemoClient.Tests;

public class RemotingTestsBase : IAssemblyFixture<SailsFixture>
{
    public RemotingTestsBase(SailsFixture fixture)
    {
        this.SailsFixture = fixture;
        // Assert that IDL file from the Sails.DemoClient project is the same as the one
        // from the SailsFixture
        var serviceCollection = new ServiceCollection();
        serviceCollection.AddRemotingViaNodeClient(
            new NodeClientOptions
            {
                GearNodeUri = this.SailsFixture.GearNodeWsUrl,
            });
        var serviceProvider = serviceCollection.BuildServiceProvider();
        this.RemotingProvider = serviceProvider.GetRequiredService<IRemotingProvider>();
        this.Remoting = this.RemotingProvider.CreateRemoting(SailsFixture.AliceAccount);
    }

    protected static readonly Random Random = new((int)DateTime.UtcNow.Ticks);

    protected readonly SailsFixture SailsFixture;
    protected readonly IRemotingProvider RemotingProvider;
    protected readonly IRemoting Remoting;
}
