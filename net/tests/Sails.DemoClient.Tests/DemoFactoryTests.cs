using System.Threading;
using FluentAssertions;
using Sails.DemoClient.Tests._Infra.XUnit.Fixtures;
using Sails.Remoting.Exceptions;
using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;

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
            .ActivateAsync(this.codeId!, RandomSalt(), CancellationToken.None);
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
            .ActivateAsync(this.codeId!, RandomSalt(), CancellationToken.None);
        // throws on ReceiveAsync
        var ex = await Assert.ThrowsAsync<ExecutionReplyException>(() => activate.ReceiveAsync(CancellationToken.None));

        // assert
        ex.Should().BeEquivalentTo(new
        {
            Message = "Not enough gas to handle program data",
            Reason = ErrorReplyReason.Execution,
            ExecutionError = SimpleExecutionError.RanOutOfGas,
        });
    }
}
