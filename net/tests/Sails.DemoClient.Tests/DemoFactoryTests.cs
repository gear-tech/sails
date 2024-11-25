using System.Threading.Tasks;
using Sails.Tests.Shared.XUnit;
using Sails.Tests.Shared.XUnit.Fixtures;
using Xunit;

namespace Sails.DemoClient.Tests;

public sealed class DemoFactoryTests : IAssemblyFixture<SailsFixture>
{
    public DemoFactoryTests(SailsFixture fixture)
    {
        this.sailsFixture = fixture;
        // Assert that IDL file from the Sails.DemoClient project is the same as the one
        // from the SailsFixture
    }

    private readonly SailsFixture sailsFixture;

    [Fact]
    public async Task Test1()
    {
        var demoContractCodeId = await this.sailsFixture.GetDemoContractCodeIdAsync();
    }
}
