using System.Threading.Tasks;
using Sails.Remoting.Tests._Infra.XUnit.Fixtures;
using Sails.Tests.Shared.XUnit;
using Substrate.Gear.Api.Generated;
using Substrate.NetApi.Model.Extrinsics;
using Xunit;

namespace Sails.Remoting.Tests.Core;

public sealed class RemotingViaNodeClientTests : IAssemblyFixture<SailsFixture>
{
    public RemotingViaNodeClientTests(SailsFixture sailsFixture)
    {
        this.sailsFixture = sailsFixture;
    }

    private readonly SailsFixture sailsFixture;

    [Fact]
    public async Task Test()
    {
        var gearNodeWsUrl = this.sailsFixture.GearNodeWsUrl;

        using (var nodeClient = new SubstrateClientExt(gearNodeWsUrl, ChargeTransactionPayment.Default()))
        {
            await nodeClient.ConnectAsync();
        }
    }
}
