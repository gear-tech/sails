using Sails.Tests.Shared.XUnit;
using Xunit;

[assembly: TestFramework(
    "Sails.Tests.Shared.XUnit.TestFramework",
    "Sails.Tests.Shared")]

[assembly: AssemblyFixture(
    typeof(Sails.Tests.Shared.XUnit.Fixtures.SailsFixture))]
