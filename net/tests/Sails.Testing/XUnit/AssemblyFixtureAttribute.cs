﻿using System;
using EnsureThat;

namespace Sails.Testing.XUnit;

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
