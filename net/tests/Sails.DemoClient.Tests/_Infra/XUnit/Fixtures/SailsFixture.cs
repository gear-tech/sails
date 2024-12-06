namespace Sails.DemoClient.Tests._Infra.XUnit.Fixtures;

public sealed class SailsFixture : Testing.XUnit.Fixtures.SailsFixture
{
    public SailsFixture()
#if SAILSRS07
        : base("demo-client-tests", "0.7.0")
#else
        : base("demo-client-tests", "0.6.3")
#endif
    {
    }
}
