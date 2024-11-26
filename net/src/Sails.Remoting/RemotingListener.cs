using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
using Sails.Remoting.Abstractions;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.Remoting;

public class RemotingListener<T> : IRemotingListener<T>
     where T : IType, new()
{
    private readonly IAsyncEnumerable<(ActorId Source, byte[] Payload)> eventStream;
    private readonly byte[] serviceRoute;
    private readonly byte[][] eventRoutes;

    public RemotingListener(
        IAsyncEnumerable<(ActorId Source, byte[] Payload)> eventStream,
        string serviceRoute,
        string[] eventRoutes)
    {
        this.eventStream = eventStream;
        this.serviceRoute = new Str(serviceRoute).Encode();
        this.eventRoutes = eventRoutes.Select(r => new Str(r).Encode()).ToArray();
    }

    public async IAsyncEnumerable<(ActorId Source, T Event)> ListenAsync(
        [EnumeratorCancellation] CancellationToken cancellationToken)
    {
        var serviceLength = this.serviceRoute.Length;
        await foreach (var (source, bytes) in this.eventStream.WithCancellation(cancellationToken))
        {
            if (bytes.Length < serviceLength || !this.serviceRoute.AsSpan().SequenceEqual(bytes.AsSpan(0, serviceLength)))
            {
                continue;
            }
            var offset = serviceLength;
            byte idx = 0;
            foreach (var route in this.eventRoutes)
            {
                if (bytes.Length < route.Length + offset)
                {
                    continue;
                }
                if (route.AsSpan().SequenceEqual(bytes.AsSpan(offset, route.Length)))
                {
                    offset += route.Length;
                    var bytesLength = bytes.Length - offset + 1;
                    var data = new byte[bytesLength];
                    data[0] = idx;
                    Buffer.BlockCopy(bytes, offset, data, 1, bytesLength - 1);

                    var p = 0;
                    T ev = new();
                    ev.Decode(data, ref p);
                    yield return (source, ev);
                }
                idx++;
            }
        }
    }
}
