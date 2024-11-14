using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Xunit.Abstractions;
using Xunit.Sdk;

namespace Sails.Tests.Shared.XUnit;

public sealed class TestCollectionRunner : XunitTestCollectionRunner
{
    public TestCollectionRunner(
        IReadOnlyDictionary<Type, object> assemblyFixtureMappings,
        ITestCollection testCollection,
        IEnumerable<IXunitTestCase> testCases,
        IMessageSink diagnosticMessageSink,
        IMessageBus messageBus,
        ITestCaseOrderer testCaseOrderer,
        ExceptionAggregator aggregator,
        CancellationTokenSource cancellationTokenSource)
        : base(
            testCollection,
            testCases,
            diagnosticMessageSink,
            messageBus,
            testCaseOrderer,
            aggregator,
            cancellationTokenSource)
    {
        EnsureArg.IsNotNull(assemblyFixtureMappings);
        EnsureArg.IsNotNull(diagnosticMessageSink);

        this.assemblyFixtureMappings = assemblyFixtureMappings;
        this.diagnosticMessageSink = diagnosticMessageSink;
    }

    private readonly IReadOnlyDictionary<Type, object> assemblyFixtureMappings;
    private readonly IMessageSink diagnosticMessageSink;

    protected override void CreateCollectionFixture(Type fixtureType)
    {
        var constructors = fixtureType.GetTypeInfo()
            .DeclaredConstructors
            .Where(ci => ci is { IsStatic: false, IsPublic: true })
            .ToList();

        if (constructors.Count != 1)
        {
            this.Aggregator.Add(
                new TestClassException(
                    $"Collection fixture type '{fixtureType.FullName}' may only define a single public constructor."));
            return;
        }

        var constructor = constructors[0];
        var missingParameters = new List<ParameterInfo>();
        var constructorArgs = constructor.GetParameters()
            .Select(
                parameterInfo =>
                {
                    switch (parameterInfo.ParameterType)
                    {
                        case var type when type == typeof(IMessageSink):
                            return this.DiagnosticMessageSink;
                        case var type when this.assemblyFixtureMappings.ContainsKey(type):
                            return this.assemblyFixtureMappings[type];
                        default:
                            missingParameters.Add(parameterInfo);
                            return null;
                    }
                })
            .ToArray();

        if (missingParameters.Count > 0)
        {
            this.Aggregator.Add(
                new TestClassException(
                    $"Collection fixture type '{fixtureType.FullName}' had one or more unresolved constructor arguments: {string.Join(", ", missingParameters.Select(parameter => $"{parameter.ParameterType.Name} {parameter.Name}"))}"));
        }
        else
        {
            this.Aggregator.Run(() => this.CollectionFixtureMappings[fixtureType] = constructor.Invoke(constructorArgs));
        }
    }

    protected override Task<RunSummary> RunTestClassAsync(
        ITestClass testClass,
        IReflectionTypeInfo @class,
        IEnumerable<IXunitTestCase> testCases)
    {
        var combinedFixtures = this.assemblyFixtureMappings.ToDictionary();

        foreach (var kvp in this.CollectionFixtureMappings)
        {
            combinedFixtures[kvp.Key] = kvp.Value;
        }

        return new XunitTestClassRunner(
            testClass,
            @class,
            testCases,
            this.diagnosticMessageSink,
            this.MessageBus,
            this.TestCaseOrderer,
            new ExceptionAggregator(this.Aggregator),
            this.CancellationTokenSource,
            combinedFixtures).RunAsync();
    }
}
