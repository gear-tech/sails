using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated.Model.gear_core.message.user;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Client;
using Substrate.Gear.Client.GearApi.Model.gprimitives;
using Substrate.Gear.Client.NetApi.Model.Types.Base;

namespace Sails.Remoting.Core;

internal sealed class BlockStreamEventListener : EventListener<(ActorId Source, byte[] Bytes)>
{
    private readonly BlocksStream blocksStream;

    internal BlockStreamEventListener(BlocksStream blocksStream)
    {
        EnsureArg.IsNotNull(blocksStream, nameof(blocksStream));

        this.blocksStream = blocksStream;
    }

    public override IAsyncEnumerable<(ActorId Source, byte[] Bytes)> ReadAllAsync(CancellationToken cancellationToken)
        => this.blocksStream.ReadAllGearRuntimeEventsAsync(cancellationToken)
            .SelectIfMatches(
                GearEvent.UserMessageSent,
                (UserMessageSentEventData data) => (UserMessage)data.Value[0])
            .Where(userMessage => userMessage.Destination
                .IsEqualTo(ActorIdExtensions.Zero))
            .Select(userMessage => (userMessage.Source, userMessage.Payload.Value.Value.Select(@byte => @byte.Value).ToArray()));

    protected override ValueTask DisposeCoreAsync()
        => this.blocksStream.DisposeAsync();
}
