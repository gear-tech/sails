using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Reflection;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Sails.Testing.XUnit;

public sealed class TestFrameworkExecutor : XunitTestFrameworkExecutor
{
    public TestFrameworkExecutor(
        AssemblyName assemblyName,
        ISourceInformationProvider sourceInformationProvider,
        IMessageSink diagnosticMessageSink)
        : base(assemblyName, sourceInformationProvider, diagnosticMessageSink)
    {
    }

    [SuppressMessage(
        "Usage", "VSTHRD100:Avoid async void methods",
        Justification = "All exceptions should be added into the aggregator")]
    protected override async void RunTestCases(
        IEnumerable<IXunitTestCase> testCases,
        IMessageSink executionMessageSink,
        ITestFrameworkExecutionOptions executionOptions)
    {
        using (var assemblyRunner = new TestAssemblyRunner(
                   this.TestAssembly,
                   testCases,
                   this.DiagnosticMessageSink,
                   executionMessageSink,
                   executionOptions))
        {
            await assemblyRunner.RunAsync().ConfigureAwait(false);
        }
    }
}
