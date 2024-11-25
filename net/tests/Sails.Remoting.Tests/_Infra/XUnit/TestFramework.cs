[assembly: TestFramework(
    "Sails.Remoting.Tests._Infra.XUnit.TestFramework",
    "Sails.Remoting.Tests")]

namespace Sails.Remoting.Tests._Infra.XUnit;

internal sealed class TestFramework : Sails.Tests.Shared.XUnit.TestFramework
{
    public TestFramework(IMessageSink messageSink)
        : base(messageSink)
    {
    }
}
