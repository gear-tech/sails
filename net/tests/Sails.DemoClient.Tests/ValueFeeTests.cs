using System;
using System.Threading;
using Sails.DemoClient.Tests._Infra.XUnit.Fixtures;
using Sails.Remoting.Abstractions;

namespace Sails.DemoClient.Tests;

public class ValueFeeTests(SailsFixture sailsFixture) : RemotingTestsBase(sailsFixture)
{
    [Fact]
    public async Task ValueFee_Works()
    {
        // arrange
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var valueFeeClient = new Demo.ValueFee(this.Remoting);

        // act
        var programId = await demoFactory
            .Default()
            .SendReceiveAsync(this.codeId!, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var result = await valueFeeClient
            .DoSomethingAndTakeFee()
            .WithValue(new ValueUnit(15_000_000_000_000))
            .SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.True(result);
        // TODO assert balances
    }
}
