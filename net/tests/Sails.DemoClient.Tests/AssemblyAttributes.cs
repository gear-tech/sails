[assembly: TestFramework(
    "Sails.Tests.Shared.XUnit.TestFramework",
    "Sails.Tests.Shared")]

[assembly: AssemblyFixture(
    typeof(Sails.DemoClient.Tests._Infra.XUnit.Fixtures.SailsFixture))]
