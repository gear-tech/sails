//HintName: demo.g.cs
using global::Sails.Remoting.Abstractions;
using global::System;
using global::System.Collections.Generic;

#pragma warning disable RCS0056 // A line is too long

namespace Demo.Client;
public interface IDemoFactory
{
    /// <summary>
    /// Program constructor (called once at the very beginning of the program lifetime)
    /// </summary>
    IActivation Default();
    /// <summary>
    /// Another program constructor (called once at the very beginning of the program lifetime)
    /// </summary>
    IActivation New(global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U32> counter, global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>> dogPosition);
}

public partial class DemoFactory : IDemoFactory
{
    private readonly IRemoting remoting;
    public DemoFactory(IRemoting remoting)
    {
        this.remoting = remoting;
    }

    /// <inheritdoc/>
    public IActivation Default()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseVoid>(this.remoting, [28, 68, 101, 102, 97, 117, 108, 116], new global::Substrate.NetApi.Model.Types.Base.BaseVoid());
    }

    /// <inheritdoc/>
    public IActivation New(global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U32> counter, global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>> dogPosition)
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U32>, global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>>>>(this.remoting, [12, 78, 101, 119], new global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U32>, global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>>>(counter, dogPosition));
    }
}

public interface ICounter
{
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.U32> Add(global::Substrate.NetApi.Model.Types.Primitive.U32 value);
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.U32> Sub(global::Substrate.NetApi.Model.Types.Primitive.U32 value);
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.U32> Value();
}

public partial class Counter : ICounter
{
    private readonly IRemoting remoting;
    public Counter(IRemoting remoting)
    {
        this.remoting = remoting;
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.U32> Add(global::Substrate.NetApi.Model.Types.Primitive.U32 value)
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.U32>(this.remoting, [28, 67, 111, 117, 110, 116, 101, 114, 12, 65, 100, 100], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(value));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.U32> Sub(global::Substrate.NetApi.Model.Types.Primitive.U32 value)
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.U32>(this.remoting, [28, 67, 111, 117, 110, 116, 101, 114, 12, 83, 117, 98], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(value));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.U32> Value()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.U32>(this.remoting, [28, 67, 111, 117, 110, 116, 101, 114, 20, 86, 97, 108, 117, 101], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }
}

public enum CounterEvents
{
    /// <summary>
    /// Emitted when a new value is added to the counter
    /// </summary>
    Added,
    /// <summary>
    /// Emitted when a value is subtracted from the counter
    /// </summary>
    Subtracted,
}

public sealed partial class EnumCounterEvents : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<CounterEvents>
{
    public EnumCounterEvents()
    {
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Primitive.U32>(CounterEvents.Added);
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Primitive.U32>(CounterEvents.Subtracted);
    }
}

public partial class CounterListener : IRemotingListener<EnumCounterEvents>
{
    private static readonly byte[][] EventRoutes = [[28, 67, 111, 117, 110, 116, 101, 114, 20, 65, 100, 100, 101, 100], [28, 67, 111, 117, 110, 116, 101, 114, 40, 83, 117, 98, 116, 114, 97, 99, 116, 101, 100], ];
    private readonly global::Sails.Remoting.Abstractions.Core.IRemotingListener remoting;
    public CounterListener(global::Sails.Remoting.Abstractions.Core.IRemotingListener remoting)
    {
        this.remoting = remoting;
    }

    public async global::System.Collections.Generic.IAsyncEnumerable<EnumCounterEvents> ListenAsync([global::System.Runtime.CompilerServices.EnumeratorCancellation] global::System.Threading.CancellationToken cancellationToken = default)
    {
        await foreach (var bytes in this.remoting.ListenAsync(cancellationToken))
        {
            byte idx = 0;
            foreach (var route in EventRoutes)
            {
                if (route.Length > bytes.Length)
                {
                    continue;
                }

                if (route.AsSpan().SequenceEqual(bytes.AsSpan()[..route.Length]))
                {
                    var bytesLength = bytes.Length - route.Length + 1;
                    var data = new byte[bytesLength];
                    data[0] = idx;
                    Buffer.BlockCopy(bytes, route.Length, data, 1, bytes.Length - route.Length);
                    var p = 0;
                    EnumCounterEvents ev = new();
                    ev.Decode(bytes, ref p);
                    yield return ev;
                }

                idx++;
            }
        }
    }
}

public interface IDog
{
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.Str> MakeSound();
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseVoid> Walk(global::Substrate.NetApi.Model.Types.Primitive.I32 dx, global::Substrate.NetApi.Model.Types.Primitive.I32 dy);
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.U32> AvgWeight();
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>> Position();
}

public partial class Dog : IDog
{
    private readonly IRemoting remoting;
    public Dog(IRemoting remoting)
    {
        this.remoting = remoting;
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.Str> MakeSound()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.Str>(this.remoting, [12, 68, 111, 103, 36, 77, 97, 107, 101, 83, 111, 117, 110, 100], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseVoid> Walk(global::Substrate.NetApi.Model.Types.Primitive.I32 dx, global::Substrate.NetApi.Model.Types.Primitive.I32 dy)
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseVoid>(this.remoting, [12, 68, 111, 103, 16, 87, 97, 108, 107], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(dx, dy));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.U32> AvgWeight()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.U32>(this.remoting, [12, 68, 111, 103, 36, 65, 118, 103, 87, 101, 105, 103, 104, 116], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>> Position()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>>(this.remoting, [12, 68, 111, 103, 32, 80, 111, 115, 105, 116, 105, 111, 110], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }
}

public enum DogEvents
{
    Barked,
    Walked,
}

public sealed partial class EnumDogEvents : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<DogEvents>
{
    public EnumDogEvents()
    {
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Base.BaseVoid>(DogEvents.Barked);
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>, global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.I32, global::Substrate.NetApi.Model.Types.Primitive.I32>>>(DogEvents.Walked);
    }
}

public partial class DogListener : IRemotingListener<EnumDogEvents>
{
    private static readonly byte[][] EventRoutes = [[12, 68, 111, 103, 24, 66, 97, 114, 107, 101, 100], [12, 68, 111, 103, 24, 87, 97, 108, 107, 101, 100], ];
    private readonly global::Sails.Remoting.Abstractions.Core.IRemotingListener remoting;
    public DogListener(global::Sails.Remoting.Abstractions.Core.IRemotingListener remoting)
    {
        this.remoting = remoting;
    }

    public async global::System.Collections.Generic.IAsyncEnumerable<EnumDogEvents> ListenAsync([global::System.Runtime.CompilerServices.EnumeratorCancellation] global::System.Threading.CancellationToken cancellationToken = default)
    {
        await foreach (var bytes in this.remoting.ListenAsync(cancellationToken))
        {
            byte idx = 0;
            foreach (var route in EventRoutes)
            {
                if (route.Length > bytes.Length)
                {
                    continue;
                }

                if (route.AsSpan().SequenceEqual(bytes.AsSpan()[..route.Length]))
                {
                    var bytesLength = bytes.Length - route.Length + 1;
                    var data = new byte[bytesLength];
                    data[0] = idx;
                    Buffer.BlockCopy(bytes, route.Length, data, 1, bytes.Length - route.Length);
                    var p = 0;
                    EnumDogEvents ev = new();
                    ev.Decode(bytes, ref p);
                    yield return ev;
                }

                idx++;
            }
        }
    }
}

public interface IPingPong
{
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>> Ping(global::Substrate.NetApi.Model.Types.Primitive.Str input);
}

public partial class PingPong : IPingPong
{
    private readonly IRemoting remoting;
    public PingPong(IRemoting remoting)
    {
        this.remoting = remoting;
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>> Ping(global::Substrate.NetApi.Model.Types.Primitive.Str input)
    {
        return new RemotingAction<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>>(this.remoting, [32, 80, 105, 110, 103, 80, 111, 110, 103, 16, 80, 105, 110, 103], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(input));
    }
}

public interface IReferences
{
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.U32> Add(global::Substrate.NetApi.Model.Types.Primitive.U32 v);
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseVec<global::Substrate.NetApi.Model.Types.Primitive.U8>> AddByte(global::Substrate.NetApi.Model.Types.Primitive.U8 @byte);
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>> GuessNum(global::Substrate.NetApi.Model.Types.Primitive.U8 number);
    global::Sails.Remoting.Abstractions.ICall<ReferenceCount> Incr();
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Base.BaseVoid, global::Substrate.NetApi.Model.Types.Primitive.Str>> SetNum(global::Substrate.NetApi.Model.Types.Primitive.U8 number);
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.Str> Baked();
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U8>> LastByte();
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.Str>> Message();
}

public partial class References : IReferences
{
    private readonly IRemoting remoting;
    public References(IRemoting remoting)
    {
        this.remoting = remoting;
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.U32> Add(global::Substrate.NetApi.Model.Types.Primitive.U32 v)
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.U32>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 12, 65, 100, 100], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(v));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseVec<global::Substrate.NetApi.Model.Types.Primitive.U8>> AddByte(global::Substrate.NetApi.Model.Types.Primitive.U8 @byte)
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseVec<global::Substrate.NetApi.Model.Types.Primitive.U8>>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 28, 65, 100, 100, 66, 121, 116, 101], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(@byte));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>> GuessNum(global::Substrate.NetApi.Model.Types.Primitive.U8 number)
    {
        return new RemotingAction<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 32, 71, 117, 101, 115, 115, 78, 117, 109], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(number));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<ReferenceCount> Incr()
    {
        return new RemotingAction<ReferenceCount>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 16, 73, 110, 99, 114], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Base.BaseVoid, global::Substrate.NetApi.Model.Types.Primitive.Str>> SetNum(global::Substrate.NetApi.Model.Types.Primitive.U8 number)
    {
        return new RemotingAction<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Base.BaseVoid, global::Substrate.NetApi.Model.Types.Primitive.Str>>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 24, 83, 101, 116, 78, 117, 109], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(number));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.Str> Baked()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.Str>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 20, 66, 97, 107, 101, 100], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U8>> LastByte()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U8>>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 32, 76, 97, 115, 116, 66, 121, 116, 101], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.Str>> Message()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.Str>>(this.remoting, [40, 82, 101, 102, 101, 114, 101, 110, 99, 101, 115, 28, 77, 101, 115, 115, 97, 103, 101], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }
}

public interface IThisThat
{
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.Gear.Api.Generated.Model.gprimitives.ActorId, global::Substrate.Gear.Api.Generated.Types.Base.NonZeroU32>, global::Substrate.NetApi.Model.Types.Primitive.Str>> DoThat(DoThatParam param);
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.U32>> DoThis(global::Substrate.NetApi.Model.Types.Primitive.U32 p1, global::Substrate.NetApi.Model.Types.Primitive.Str p2, global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.Gear.Client.Model.Types.Primitive.H160>, global::Substrate.Gear.Client.Model.Types.Primitive.NonZeroU8> p3, TupleStruct p4);
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseVoid> Noop();
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>> That();
    global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.U32> This();
}

public partial class ThisThat : IThisThat
{
    private readonly IRemoting remoting;
    public ThisThat(IRemoting remoting)
    {
        this.remoting = remoting;
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.Gear.Api.Generated.Model.gprimitives.ActorId, global::Substrate.Gear.Api.Generated.Types.Base.NonZeroU32>, global::Substrate.NetApi.Model.Types.Primitive.Str>> DoThat(DoThatParam param)
    {
        return new RemotingAction<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.Gear.Api.Generated.Model.gprimitives.ActorId, global::Substrate.Gear.Api.Generated.Types.Base.NonZeroU32>, global::Substrate.NetApi.Model.Types.Primitive.Str>>(this.remoting, [32, 84, 104, 105, 115, 84, 104, 97, 116, 24, 68, 111, 84, 104, 97, 116], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(param));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.U32>> DoThis(global::Substrate.NetApi.Model.Types.Primitive.U32 p1, global::Substrate.NetApi.Model.Types.Primitive.Str p2, global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.Gear.Client.Model.Types.Primitive.H160>, global::Substrate.Gear.Client.Model.Types.Primitive.NonZeroU8> p3, TupleStruct p4)
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.U32>>(this.remoting, [32, 84, 104, 105, 115, 84, 104, 97, 116, 24, 68, 111, 84, 104, 105, 115], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust(p1, p2, p3, p4));
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Base.BaseVoid> Noop()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Base.BaseVoid>(this.remoting, [32, 84, 104, 105, 115, 84, 104, 97, 116, 16, 78, 111, 111, 112], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>> That()
    {
        return new RemotingAction<global::Substrate.Gear.Client.Model.Types.Base.BaseResult<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.NetApi.Model.Types.Primitive.Str>>(this.remoting, [32, 84, 104, 105, 115, 84, 104, 97, 116, 16, 84, 104, 97, 116], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.IQuery<global::Substrate.NetApi.Model.Types.Primitive.U32> This()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.U32>(this.remoting, [32, 84, 104, 105, 115, 84, 104, 97, 116, 16, 84, 104, 105, 115], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }
}

public interface IValueFee
{
    global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.Bool> DoSomethingAndTakeFee();
}

public partial class ValueFee : IValueFee
{
    private readonly IRemoting remoting;
    public ValueFee(IRemoting remoting)
    {
        this.remoting = remoting;
    }

    /// <inheritdoc/>
    public global::Sails.Remoting.Abstractions.ICall<global::Substrate.NetApi.Model.Types.Primitive.Bool> DoSomethingAndTakeFee()
    {
        return new RemotingAction<global::Substrate.NetApi.Model.Types.Primitive.Bool>(this.remoting, [32, 86, 97, 108, 117, 101, 70, 101, 101, 84, 68, 111, 83, 111, 109, 101, 116, 104, 105, 110, 103, 65, 110, 100, 84, 97, 107, 101, 70, 101, 101], new global::Substrate.NetApi.Model.Types.Base.BaseTupleRust());
    }
}

public enum ValueFeeEvents
{
    Withheld,
}

public sealed partial class EnumValueFeeEvents : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<ValueFeeEvents>
{
    public EnumValueFeeEvents()
    {
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Primitive.U128>(ValueFeeEvents.Withheld);
    }
}

public partial class ValueFeeListener : IRemotingListener<EnumValueFeeEvents>
{
    private static readonly byte[][] EventRoutes = [[32, 86, 97, 108, 117, 101, 70, 101, 101, 32, 87, 105, 116, 104, 104, 101, 108, 100], ];
    private readonly global::Sails.Remoting.Abstractions.Core.IRemotingListener remoting;
    public ValueFeeListener(global::Sails.Remoting.Abstractions.Core.IRemotingListener remoting)
    {
        this.remoting = remoting;
    }

    public async global::System.Collections.Generic.IAsyncEnumerable<EnumValueFeeEvents> ListenAsync([global::System.Runtime.CompilerServices.EnumeratorCancellation] global::System.Threading.CancellationToken cancellationToken = default)
    {
        await foreach (var bytes in this.remoting.ListenAsync(cancellationToken))
        {
            byte idx = 0;
            foreach (var route in EventRoutes)
            {
                if (route.Length > bytes.Length)
                {
                    continue;
                }

                if (route.AsSpan().SequenceEqual(bytes.AsSpan()[..route.Length]))
                {
                    var bytesLength = bytes.Length - route.Length + 1;
                    var data = new byte[bytesLength];
                    data[0] = idx;
                    Buffer.BlockCopy(bytes, route.Length, data, 1, bytes.Length - route.Length);
                    var p = 0;
                    EnumValueFeeEvents ev = new();
                    ev.Decode(bytes, ref p);
                    yield return ev;
                }

                idx++;
            }
        }
    }
}

[global::Substrate.NetApi.Attributes.SubstrateNodeType(global::Substrate.NetApi.Model.Types.Metadata.Base.TypeDefEnum.Composite)]
public sealed partial class ReferenceCount : global::Substrate.NetApi.Model.Types.Base.BaseType
{
    [System.Diagnostics.CodeAnalysis.AllowNull]
    public global::Substrate.NetApi.Model.Types.Primitive.U32 Value { get; set; }

    /// <inheritdoc/>
    public override string TypeName() => "ReferenceCount";
    /// <inheritdoc/>
    public override byte[] Encode()
    {
        var result = new List<byte>();
        result.AddRange(this.Value.Encode());
        return result.ToArray();
    }

    /// <inheritdoc/>
    public override void Decode(byte[] byteArray, ref int p)
    {
        var start = p;
        this.Value = new global::Substrate.NetApi.Model.Types.Primitive.U32();
        this.Value.Decode(byteArray, ref p);
        var bytesLength = p - start;
        this.TypeSize = bytesLength;
        this.Bytes = new byte[bytesLength];
        Array.Copy(byteArray, start, this.Bytes, 0, bytesLength);
    }
}

[global::Substrate.NetApi.Attributes.SubstrateNodeType(global::Substrate.NetApi.Model.Types.Metadata.Base.TypeDefEnum.Composite)]
public sealed partial class DoThatParam : global::Substrate.NetApi.Model.Types.Base.BaseType
{
    [System.Diagnostics.CodeAnalysis.AllowNull]
    public global::Substrate.Gear.Api.Generated.Types.Base.NonZeroU32 P1 { get; set; }

    [System.Diagnostics.CodeAnalysis.AllowNull]
    public global::Substrate.Gear.Api.Generated.Model.gprimitives.ActorId P2 { get; set; }

    [System.Diagnostics.CodeAnalysis.AllowNull]
    public EnumManyVariants P3 { get; set; }

    /// <inheritdoc/>
    public override string TypeName() => "DoThatParam";
    /// <inheritdoc/>
    public override byte[] Encode()
    {
        var result = new List<byte>();
        result.AddRange(this.P1.Encode());
        result.AddRange(this.P2.Encode());
        result.AddRange(this.P3.Encode());
        return result.ToArray();
    }

    /// <inheritdoc/>
    public override void Decode(byte[] byteArray, ref int p)
    {
        var start = p;
        this.P1 = new global::Substrate.Gear.Api.Generated.Types.Base.NonZeroU32();
        this.P1.Decode(byteArray, ref p);
        this.P2 = new global::Substrate.Gear.Api.Generated.Model.gprimitives.ActorId();
        this.P2.Decode(byteArray, ref p);
        this.P3 = new EnumManyVariants();
        this.P3.Decode(byteArray, ref p);
        var bytesLength = p - start;
        this.TypeSize = bytesLength;
        this.Bytes = new byte[bytesLength];
        Array.Copy(byteArray, start, this.Bytes, 0, bytesLength);
    }
}

public enum ManyVariants
{
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

public sealed partial class EnumManyVariants : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<ManyVariants>
{
    public EnumManyVariants()
    {
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Base.BaseVoid>(ManyVariants.One);
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Primitive.U32>(ManyVariants.Two);
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U256>>(ManyVariants.Three);
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.U32, global::Substrate.NetApi.Model.Types.Base.BaseOpt<global::Substrate.NetApi.Model.Types.Primitive.U16>>>(ManyVariants.Four);
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Base.BaseTuple<global::Substrate.NetApi.Model.Types.Primitive.Str, global::Substrate.Gear.Api.Generated.Model.primitive_types.H256>>(ManyVariants.Five);
        this.AddTypeDecoder<global::Substrate.NetApi.Model.Types.Primitive.U32>(ManyVariants.Six);
    }
}

[global::Substrate.NetApi.Attributes.SubstrateNodeType(global::Substrate.NetApi.Model.Types.Metadata.Base.TypeDefEnum.Composite)]
public sealed partial class TupleStruct : global::Substrate.NetApi.Model.Types.Base.BaseType
{
    [System.Diagnostics.CodeAnalysis.AllowNull]
    public global::Substrate.NetApi.Model.Types.Primitive.Bool Value { get; set; }

    /// <inheritdoc/>
    public override string TypeName() => "TupleStruct";
    /// <inheritdoc/>
    public override byte[] Encode()
    {
        var result = new List<byte>();
        result.AddRange(this.Value.Encode());
        return result.ToArray();
    }

    /// <inheritdoc/>
    public override void Decode(byte[] byteArray, ref int p)
    {
        var start = p;
        this.Value = new global::Substrate.NetApi.Model.Types.Primitive.Bool();
        this.Value.Decode(byteArray, ref p);
        var bytesLength = p - start;
        this.TypeSize = bytesLength;
        this.Bytes = new byte[bytesLength];
        Array.Copy(byteArray, start, this.Bytes, 0, bytesLength);
    }
}
