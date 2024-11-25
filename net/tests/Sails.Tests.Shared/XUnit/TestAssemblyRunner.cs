using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Xunit;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Sails.Tests.Shared.XUnit;

public sealed class TestAssemblyRunner : XunitTestAssemblyRunner
{
    public TestAssemblyRunner(
        ITestAssembly testAssembly,
        IEnumerable<IXunitTestCase> testCases,
        IMessageSink diagnosticMessageSink,
        IMessageSink executionMessageSink,
        ITestFrameworkExecutionOptions executionOptions)
        : base(
            testAssembly,
            testCases,
            diagnosticMessageSink,
            executionMessageSink,
            executionOptions)
    {
    }

    private readonly Dictionary<Type, object> assemblyFixtureMappings = [];

    protected override async Task AfterTestAssemblyStartingAsync()
    {
        await base.AfterTestAssemblyStartingAsync().ConfigureAwait(false);

        var requiredFixtureTypes = new HashSet<Type>();

        foreach (var testCase in this.TestCases)
        {
            var testClassType = testCase.TestMethod.TestClass.Class.ToRuntimeType();

            foreach (var interfaceType in testClassType.GetInterfaces())
            {
                if (interfaceType.IsGenericType && interfaceType.GetGenericTypeDefinition() == typeof(IAssemblyFixture<>))
                {
                    var fixtureType = interfaceType.GetGenericArguments()[0];
                    requiredFixtureTypes.Add(fixtureType);
                }
            }
        }

        this.Aggregator.Run(
            () =>
            {
                var fixturesAttrs = ((IReflectionAssemblyInfo)this.TestAssembly.Assembly).Assembly
                    .GetCustomAttributes(typeof(AssemblyFixtureAttribute), inherit: false)
                    .Cast<AssemblyFixtureAttribute>();

                foreach (var fixtureAttr in fixturesAttrs)
                {
                    if (requiredFixtureTypes.Contains(fixtureAttr.FixtureType))
                    {
                        var newInstance = Activator.CreateInstance(fixtureAttr.FixtureType);

                        if (newInstance is not null)
                        {
                            this.assemblyFixtureMappings[fixtureAttr.FixtureType] = newInstance;
                        }
                    }
                }
            });

        foreach (var initializable in this.assemblyFixtureMappings.Values.OfType<IAsyncLifetime>())
        {
            await this.Aggregator.RunAsync(initializable.InitializeAsync).ConfigureAwait(false);
        }
    }

    protected override Task BeforeTestAssemblyFinishedAsync()
    {
        foreach (var disposable in this.assemblyFixtureMappings.Values.OfType<IAsyncLifetime>())
        {
            this.Aggregator.RunAsync(disposable.DisposeAsync);
        }

        return base.BeforeTestAssemblyFinishedAsync();
    }

    protected override Task<RunSummary> RunTestCollectionAsync(
        IMessageBus messageBus,
        ITestCollection testCollection,
        IEnumerable<IXunitTestCase> testCases,
        CancellationTokenSource cancellationTokenSource)
        => new TestCollectionRunner(
            this.assemblyFixtureMappings,
            testCollection,
            testCases,
            this.DiagnosticMessageSink,
            messageBus,
            this.TestCaseOrderer,
            new ExceptionAggregator(this.Aggregator),
            cancellationTokenSource).RunAsync();
}
