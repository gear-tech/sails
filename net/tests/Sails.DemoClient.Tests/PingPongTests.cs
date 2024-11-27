using System;
using System.Threading;
using Sails.DemoClient.Tests._Infra.XUnit.Fixtures;
using Sails.Remoting.Abstractions;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.DemoClient.Tests;

public class PingPongTests(SailsFixture sailsFixture) : RemotingTestsBase(sailsFixture)
{
    [Fact]
    public async Task PingPong_Works()
    {
        // arrange
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var pingPongClient = new Demo.PingPong(this.Remoting);

        // act
        var programId = await demoFactory
            .Default()
            .SendReceiveAsync(this.codeId!, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var result = await pingPongClient.Ping(new Str("ping")).SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.True(result.Matches<BaseResultEnum, Str>(BaseResultEnum.Ok, s => s == "pong"));
    }
}
