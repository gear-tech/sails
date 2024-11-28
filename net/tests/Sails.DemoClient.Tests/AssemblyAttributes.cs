[assembly: TestFramework(
    "Sails.Testing.XUnit.TestFramework",
    "Sails.Net.Testing")]

[assembly: AssemblyFixture(
    typeof(Sails.DemoClient.Tests._Infra.XUnit.Fixtures.SailsFixture))]

[assembly: CollectionBehavior(DisableTestParallelization = true)]
