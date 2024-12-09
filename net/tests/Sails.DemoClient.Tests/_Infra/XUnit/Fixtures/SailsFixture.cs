namespace Sails.DemoClient.Tests._Infra.XUnit.Fixtures;

public sealed class SailsFixture : Testing.XUnit.Fixtures.SailsFixture
{
    public SailsFixture()
        : base("demo-client-tests", "0.7.0")
    {
    }
}
