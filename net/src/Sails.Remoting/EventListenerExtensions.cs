using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting;

public static class EventListenerExtensions
{
    /// <summary>
    /// Projects Gear event to Typed Service Event
    /// </summary>
    public static EventListener<(ActorId Source, T Event)> ToServiceEventListener<T>(
        this EventListener<(ActorId Source, byte[] Payload)> source,
        string serviceRoute,
        string[] eventRoutes)
        where T : IType, new()
    {
        EnsureArg.IsNotNull(source, nameof(source));
        EnsureArg.IsNotNull(serviceRoute, nameof(serviceRoute));
        EnsureArg.IsNotNull(eventRoutes, nameof(eventRoutes));

        return new ServiceEventListener<T>(source, serviceRoute, eventRoutes);
    }
}
