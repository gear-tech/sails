using System;
using EnsureThat;

namespace Sails.Tests.Shared.XUnit;

[AttributeUsage(AttributeTargets.Assembly)]
public sealed class AssemblyFixtureAttribute : Attribute
{
    public AssemblyFixtureAttribute(Type fixtureType)
    {
        EnsureArg.IsNotNull(fixtureType);

        this.FixtureType = fixtureType;
    }

    public Type FixtureType { get; }
}
