using System;
using System.Threading;
using Sails.DemoClient.Tests._Infra.XUnit.Fixtures;

namespace Sails.DemoClient.Tests;

public sealed class DemoFactoryTests(SailsFixture fixture) : RemotingTestsBase(fixture)
{
    [Fact]
    public async Task Demo_DefaultConstructor_Works()
    {
        // arrange

        // act
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var activate = await demoFactory
            .Default()
            .ActivateAsync(this.codeId!, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);
        var programId = await activate.ReceiveAsync(CancellationToken.None);

        // assert
        Assert.NotNull(programId);
    }

    [Fact]
    public async Task Demo_Activation_Throws_NotEnoughGas()
    {
        // arrange

        // act
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var activate = await demoFactory
            .Default()
            .WithGasLimit(new GasUnit(0))
            .ActivateAsync(this.codeId!, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);
        // throws on ReceiveAsync
        var ex = await Assert.ThrowsAsync<ArgumentException>(() => activate.ReceiveAsync(CancellationToken.None));

        // assert
        // TODO assert custom exception
    }
}
