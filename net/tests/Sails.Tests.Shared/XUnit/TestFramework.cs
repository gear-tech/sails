using System.Reflection;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Sails.Tests.Shared.XUnit;

public class TestFramework : XunitTestFramework
{
    public TestFramework(IMessageSink messageSink)
        : base(messageSink)
    {
    }

    protected override ITestFrameworkExecutor CreateExecutor(AssemblyName assemblyName)
        => new TestFrameworkExecutor(
            assemblyName,
            this.SourceInformationProvider,
            this.DiagnosticMessageSink);
}
