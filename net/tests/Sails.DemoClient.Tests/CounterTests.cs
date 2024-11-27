using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using Sails.DemoClient.Tests._Infra.XUnit.Fixtures;
using Sails.Remoting.Abstractions;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.DemoClient.Tests;

public class CounterTests(SailsFixture sailsFixture) : RemotingTestsBase(sailsFixture)
{
    [Fact]
    public async Task Counter_Add_Works()
    {
        // arrange
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);

        var eventStream = await Demo.CounterListener.ListenAsync(this.RemotingListener);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(this.codeId!, RandomSalt(), CancellationToken.None);

        var result = await counterClient.Add(new U32(10)).SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.NotNull(result);
        Assert.Equal(52u, result.Value);

        var (source, ev) = await eventStream.FirstAsync();
        Assert.True(source.IsEqualTo(programId));
        Assert.True(ev.Matches<Demo.CounterEvents, U32>(Demo.CounterEvents.Added, static v => v.Value == 10));
    }

    [Fact]
    public async Task Counter_Sub_Works()
    {
        // arrange
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);

        var eventStream = await Demo.CounterListener.ListenAsync(this.RemotingListener);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(this.codeId!, RandomSalt(), CancellationToken.None);

        var result = await counterClient.Sub(new U32(10)).SendReceiveAsync(programId, CancellationToken.None);

        // assert
        Assert.NotNull(result);
        Assert.Equal(32u, result.Value);

        var (source, ev) = await eventStream.FirstAsync();
        Assert.True(source.IsEqualTo(programId));
        Assert.True(ev.Matches<Demo.CounterEvents, U32>(Demo.CounterEvents.Subtracted, static v => v.Value == 10));
    }

    [Fact]
    public async Task Counter_Query_Works()
    {
        // arrange
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(this.codeId!, RandomSalt(), CancellationToken.None);

        var result = await counterClient.Value().QueryAsync(programId, CancellationToken.None);

        // assert
        Assert.NotNull(result);
        Assert.Equal(42u, result.Value);
    }

    [Fact]
    public async Task Counter_Query_Throws_NotEnoughGas()
    {
        // arrange
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(this.codeId!, RandomSalt(), CancellationToken.None);

        var ex = await Assert.ThrowsAsync<ArgumentException>(() => counterClient.Value()
            .WithGasLimit(new GasUnit(0))
            .QueryAsync(programId, CancellationToken.None)
        );

        // assert
        // TODO assert exception
    }

    [Fact]
    public async Task Counter_Collect_Events_With_Cancellation()
    {
        // arrange
        var demoFactory = new Demo.DemoFactory(this.Remoting);
        var counterClient = new Demo.Counter(this.Remoting);

        var eventStream = await Demo.CounterListener.ListenAsync(this.RemotingListener);

        // act
        var dogPosition = new BaseOpt<BaseTuple<I32, I32>>(new BaseTuple<I32, I32>(new I32(0), new I32(0)));
        var programId = await demoFactory
            .New(counter: new U32(42), dogPosition: dogPosition)
            .SendReceiveAsync(this.codeId!, RandomSalt(), CancellationToken.None);

        var addResult = await counterClient.Add(new U32(10)).SendReceiveAsync(programId, CancellationToken.None);

        var subResult = await counterClient.Sub(new U32(20)).SendReceiveAsync(programId, CancellationToken.None);

        var cts = new CancellationTokenSource();
        cts.CancelAfter(TimeSpan.FromSeconds(5));

        var list = new List<(ActorId, Demo.EnumCounterEvents)>();
        try
        {
            await foreach (var item in eventStream.WithCancellation(cts.Token))
            {
                list.Add(item);
            }
        }
        catch (OperationCanceledException) when (cts.Token.IsCancellationRequested)
        {
            // Expected when the time window elapses
        }

        Assert.Collection(
            list,
            ev => Assert.True(ev.Item2.Matches<Demo.CounterEvents, U32>(Demo.CounterEvents.Added, static v => v.Value == 10)),
            ev => Assert.True(ev.Item2.Matches<Demo.CounterEvents, U32>(Demo.CounterEvents.Subtracted, static v => v.Value == 20))
        );
    }
}
