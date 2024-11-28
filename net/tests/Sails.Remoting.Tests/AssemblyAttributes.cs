[assembly: TestFramework(
    "Sails.Testing.XUnit.TestFramework",
    "Sails.Testing")]

[assembly: AssemblyFixture(
    typeof(Sails.Remoting.Tests._Infra.XUnit.Fixtures.SailsFixture))]
