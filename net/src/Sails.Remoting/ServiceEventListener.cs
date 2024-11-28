using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.Remoting;

internal class ServiceEventListener<T> : EventListener<(ActorId Source, T Event)>
     where T : IType, new()
{
    private readonly EventListener<(ActorId Source, byte[] Payload)> source;
    private readonly byte[] serviceRoute;
    private readonly byte[][] eventRoutes;

    internal ServiceEventListener(
        EventListener<(ActorId Source, byte[] Payload)> source,
        string serviceRoute,
        string[] eventRoutes)
    {
        this.source = source;
        this.serviceRoute = new Str(serviceRoute).Encode();
        this.eventRoutes = eventRoutes.Select(r => new Str(r).Encode()).ToArray();
    }

    public override IAsyncEnumerable<(ActorId Source, T Event)> ReadAllAsync(CancellationToken cancellationToken = default)
        => this.source
            .ReadAllAsync(cancellationToken)
            .Select(this.Map)
            .Where(x => x != null)
            .Select(x => x!.Value);

    protected override ValueTask DisposeCoreAsync() => this.source.DisposeAsync();

    private (ActorId Source, T Event)? Map((ActorId, byte[]) tuple)
    {
        var (source, bytes) = tuple;
        var serviceLength = this.serviceRoute.Length;
        if (bytes.Length < serviceLength || !this.serviceRoute.AsSpan().SequenceEqual(bytes.AsSpan(0, serviceLength)))
        {
            return null;
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
                return (source, ev);
            }
            idx++;
        }
        return null;
    }
}
