using System;
using System.Threading;
using Sails.DemoClient.Tests._Infra.XUnit.Fixtures;
using Sails.Remoting.Abstractions;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.DemoClient.Tests;

public class CounterTests(SailsFixture sailsFixture) : RemotingTestsBase(sailsFixture)
{
    [Fact]
    public async Task Counter_Add_Works()
    {
        // arrange
        var codeId = await this.SailsFixture.GetDemoContractCodeIdAsync();

        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);
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
        var codeId = await this.SailsFixture.GetDemoContractCodeIdAsync();

        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);
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
        var codeId = await this.SailsFixture.GetDemoContractCodeIdAsync();

        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);

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
        var codeId = await this.SailsFixture.GetDemoContractCodeIdAsync();

        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(codeId, BitConverter.GetBytes(Random.NextInt64()), CancellationToken.None);

        //var ex = await Assert.ThrowsAsync<Exception>(() => counterClient.Value()
        //    .WithGasLimit(new GasUnit(0))
        //    .QueryAsync(programId, CancellationToken.None)
        //);

        // assert
        // TODO Assert ReplyException
    }
}
