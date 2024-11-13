using Sails.Remoting.Tests._Infra.XUnit.Fixtures;
using Sails.TestUtils.XUnit;

[assembly: AssemblyFixture(typeof(SailsFixture))]

namespace Sails.Remoting.Tests._Infra.XUnit.Fixtures;

public sealed class SailsFixture : TestUtils.XUnit.Fixtures.SailsFixture
{
    public SailsFixture()
        : base("0.6.3")
    {
    }
}
