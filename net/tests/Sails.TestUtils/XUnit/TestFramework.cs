using System.Reflection;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Sails.TestUtils.XUnit;

public abstract class TestFramework : XunitTestFramework
{
    protected TestFramework(IMessageSink messageSink)
        : base(messageSink)
    {
    }

    protected override ITestFrameworkExecutor CreateExecutor(AssemblyName assemblyName)
        => new TestFrameworkExecutor(
            assemblyName,
            this.SourceInformationProvider,
            this.DiagnosticMessageSink);
}
