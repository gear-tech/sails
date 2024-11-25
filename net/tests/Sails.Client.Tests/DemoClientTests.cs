namespace Sails.Client.Tests;

public class DemoClientTests
{
    [Fact]
    public async Task Demo_DefaultConstructor()
    {
        // arrange
        var route = new Str("Default").Encode();
        var bytes = Utils.HexToByteArray("0xd5c7e252243071f25d8013aa60e87e1650b4f069983eeafcecec10c0a03619ae");
        var expectedProgramId = new ActorId { Value = new Arr32U8 { Value = bytes.ToArrayOfU8() } };

        var remotingReply = Substitute.For<RemotingReply<(ActorId, byte[])>>();
        remotingReply.ReadAsync(Arg.Any<CancellationToken>()).Returns(Task.FromResult((expectedProgramId, route)));

        var remoting = Substitute.For<IRemoting>();
        remoting
            .ActivateAsync(
                codeId: Arg.Any<CodeId>(),
                salt: Arg.Any<IReadOnlyCollection<byte>>(),
                encodedPayload: Arg.Any<IReadOnlyCollection<byte>>(),
                gasLimit: Arg.Any<GasUnit?>(),
                value: Arg.Any<ValueUnit>(),
                cancellationToken: Arg.Any<CancellationToken>())
            .Returns(Task.FromResult(remotingReply));

        var codeId = new CodeId { Value = new Arr32U8 { Value = (new byte[32]).ToArrayOfU8() } };

        // act
        var demoFactory = new Demo.DemoFactory(remoting);
        var activate = await demoFactory.Default().ActivateAsync(codeId, [], CancellationToken.None);
        var programId = await activate.ReceiveAsync(CancellationToken.None);

        // assert
        Assert.True(programId.IsEqualTo(expectedProgramId));
    }

    [Fact]
    public async Task PingPong_Works()
    {
        // arrange
        var res = new BaseResult<Str, Str>();
        res.Create(BaseResultEnum.Ok, new Str("pong"));
        var replyBytes = new BaseTuple<Str, Str, BaseResult<Str, Str>>(new Str("PingPong"), new Str("Ping"), res).Encode();

        var remotingReply = Substitute.For<RemotingReply<byte[]>>();
        remotingReply.ReadAsync(Arg.Any<CancellationToken>()).Returns(Task.FromResult(replyBytes));

        var programId = new ActorId { Value = new Arr32U8 { Value = (new byte[32]).ToArrayOfU8() } };

        var remoting = Substitute.For<IRemoting>();
        remoting
            .MessageAsync(
                programId: programId,
                encodedPayload: Arg.Any<IReadOnlyCollection<byte>>(),
                gasLimit: Arg.Any<GasUnit?>(),
                value: Arg.Any<ValueUnit>(),
                cancellationToken: Arg.Any<CancellationToken>())
            .Returns(Task.FromResult(remotingReply));

        // act
        var pingPong = new Demo.PingPong(remoting);
        var message = await pingPong.Ping((Str)"ping").MessageAsync(programId, CancellationToken.None);
        var pingReply = await message.ReceiveAsync(CancellationToken.None);

        // assert
        Assert.True(pingReply.Matches<BaseResultEnum, Str>(BaseResultEnum.Ok, s => s == "pong"));
    }
}
