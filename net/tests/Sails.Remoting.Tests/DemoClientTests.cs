using Sails.Remoting.Tests._Infra.XUnit.Fixtures;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Client;
using Substrate.Gear.Client.Extensions;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NET.Schnorrkel.Keys;
using Substrate.NetApi.Model.Extrinsics;

namespace Sails.Remoting.Tests;

public class DemoClientTests : IAssemblyFixture<SailsFixture>
{
    public DemoClientTests(SailsFixture sailsFixture)
    {
        this.sailsFixture = sailsFixture;
        var serviceCollection = new ServiceCollection();
        serviceCollection.AddRemotingViaNodeClient(
            new NodeClientOptions
            {
                GearNodeUri = this.sailsFixture.GearNodeWsUrl,
            });
        var serviceProvider = serviceCollection.BuildServiceProvider();
        this.remotingProvider = serviceProvider.GetRequiredService<IRemotingProvider>();
        this.remoting = this.remotingProvider.CreateRemoting(AliceAccount);
    }

    private static readonly MiniSecret AliceMiniSecret
        = new(
            Utils.HexToByteArray("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"),
            ExpandMode.Ed25519);
    private static readonly Account AliceAccount
        = Account.Build(
            KeyType.Sr25519,
            AliceMiniSecret.ExpandToSecret().ToEd25519Bytes(),
            AliceMiniSecret.GetPair().Public.Key);
    private static readonly Random Random = new((int)DateTime.UtcNow.Ticks);

    private readonly SailsFixture sailsFixture;
    private readonly IRemotingProvider remotingProvider;
    private readonly IRemoting remoting;

    [Fact]
    public async Task Demo_DefaultConstructor_Works()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        // act
        var demoFactory = new Demo.DemoFactory(this.remoting);
        var activate = await demoFactory
            .Default()
            .ActivateAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);
        var programId = await activate.ReceiveAsync(CancellationToken.None);

        // assert
        Assert.NotNull(programId);
    }

    [Fact]
    public async Task Demo_Activation_Throws_NotEnoughGas()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        // act
        var demoFactory = new Demo.DemoFactory(this.remoting);
        var activate = await demoFactory
            .Default()
            .WithGasLimit(new GasUnit(0))
            .ActivateAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);
        // throws on ReceiveAsync
        var ex = await Assert.ThrowsAsync<ArgumentException>(() => activate.ReceiveAsync(CancellationToken.None));

        // assert
        // TODO assert custom exception
        Assert.NotNull(ex);
    }

    [Fact]
    public async Task PingPong_Works()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        var demoFactory = new Demo.DemoFactory(this.remoting);
        var pingPongClient = new Demo.PingPong(this.remoting);

        // act
        var programId = await demoFactory
            .Default()
            .SendReceiveAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var result = await pingPongClient.Ping(new Str("ping")).SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.True(result.Matches<BaseResultEnum, Str>(BaseResultEnum.Ok, s => s == "pong"));
    }

    [Fact]
    public async Task Counter_Add_Works()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        var demoFactory = new Demo.DemoFactory(this.remoting);
        var counterClient = new Demo.Counter(this.remoting);
        // TODO add listener
        //var counterClient = new Demo.CounterListener(this.remoting);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var result = await counterClient.Add(new U32(10)).SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.NotNull(result);
        Assert.Equal(52u, result.Value);
        // TODO add event assert
    }

    [Fact]
    public async Task Counter_Sub_Works()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        var demoFactory = new Demo.DemoFactory(this.remoting);
        var counterClient = new Demo.Counter(this.remoting);
        // TODO add listener
        //var counterClient = new Demo.CounterListener(this.remoting);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var result = await counterClient.Sub(new U32(10)).SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.NotNull(result);
        Assert.Equal(32u, result.Value);
        // TODO add event assert
    }

    [Fact]
    public async Task Counter_Query_Works()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        var demoFactory = new Demo.DemoFactory(this.remoting);
        var counterClient = new Demo.Counter(this.remoting);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var result = await counterClient.Value().QueryAsync(programId, CancellationToken.None);

        // assert
        Assert.NotNull(result);
        Assert.Equal(42u, result.Value);
    }

    [Fact]
    public async Task Counter_Query_Throws_NotEnoughGas()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        var demoFactory = new Demo.DemoFactory(this.remoting);
        var counterClient = new Demo.Counter(this.remoting);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var ex = await Assert.ThrowsAsync<ArgumentException>(() => counterClient.Value()
            .WithGasLimit(new GasUnit(0))
            .QueryAsync(programId, CancellationToken.None)
        );

        // assert
        Assert.NotNull(ex);
    }

    [Fact]
    public async Task ValueFee_Works()
    {
        // arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        var demoFactory = new Demo.DemoFactory(this.remoting);
        var valueFeeClient = new Demo.ValueFee(this.remoting);

        // act
        var programId = await demoFactory
            .Default()
            .SendReceiveAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        var result = await valueFeeClient
            .DoSomethingAndTakeFee()
            .WithValue(new ValueUnit(15_000_000_000_000))
            .SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.True(result);
        // TODO assert balances
    }

    private async Task<CodeId> UploadCodeAsync(IReadOnlyCollection<byte> codeBytes)
    {
        using (var nodeClient = new SubstrateClientExt(
            this.sailsFixture.GearNodeWsUrl,
            ChargeTransactionPayment.Default()))
        {
            await nodeClient.ConnectAsync();

            return await nodeClient.UploadCodeAsync(
                AliceAccount,
                codeBytes,
                CancellationToken.None);
        }
    }
}
