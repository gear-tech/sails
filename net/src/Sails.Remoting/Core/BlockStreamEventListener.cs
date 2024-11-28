using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Client;

namespace Sails.Remoting.Core;

internal sealed class BlockStreamEventListener(BlocksStream blocksStream)
    : EventListener<(ActorId Source, byte[] Bytes)>
{
    public override IAsyncEnumerable<(ActorId Source, byte[] Bytes)> ReadAllAsync(CancellationToken cancellationToken)
        => blocksStream
            .ReadAllEventsAsync(cancellationToken)
            .SelectGearEvents()
            .SelectServiceEvents();

    protected override ValueTask DisposeCoreAsync() => blocksStream.DisposeAsync();
}
