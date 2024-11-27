using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using EnsureThat;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting;

public static class RemotingListenerExtensions
{
    /// <summary>
    /// Projects Gear event to Typed Service Event
    /// </summary>
    [SuppressMessage(
        "Style",
        "VSTHRD200:Use \"Async\" suffix for async methods",
        Justification = "To be consistent with system provided extensions")]
    public static IAsyncEnumerable<(ActorId Source, T Event)> SelectEvent<T>(
        this IAsyncEnumerable<(ActorId Source, byte[] Payload)> source,
        string serviceRoute,
        string[] eventRoutes)
        where T : IType, new()
    {
        EnsureArg.IsNotNull(source, nameof(source));
        EnsureArg.IsNotNull(serviceRoute, nameof(serviceRoute));
        EnsureArg.IsNotNull(eventRoutes, nameof(eventRoutes));

        return new EventAsyncIterator<T>(source, serviceRoute, eventRoutes);
    }
}
