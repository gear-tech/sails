using System.Threading.Tasks;
using Sails.Remoting.Tests._Infra.XUnit.Fixtures;
using Sails.TestUtils.XUnit;
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
        var demoIdl = await this.sailsFixture.GetDemoContractIdlAsync();
        var demoContractWasm = await this.sailsFixture.GetDemoContractWasmAsync();
        var noSvcsProgIdl = await this.sailsFixture.GetNoSvcsProgContractIdlAsync();
        var gearNodeWsUrl = this.sailsFixture.GearNodeWsUrl;
    }
}
