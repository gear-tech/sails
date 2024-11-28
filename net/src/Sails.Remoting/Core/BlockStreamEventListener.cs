using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Client;

namespace Sails.Remoting.Core;

internal sealed class BlockStreamEventListener : EventListener<(ActorId Source, byte[] Bytes)>
{
    private readonly SubstrateClientExt nodeClient;
    private readonly BlocksStream blocksStream;

    internal BlockStreamEventListener(SubstrateClientExt nodeClient, BlocksStream blocksStream)
    {
        this.nodeClient = nodeClient;
        this.blocksStream = blocksStream;
    }

    public override IAsyncEnumerable<(ActorId Source, byte[] Bytes)> ReadAllAsync(CancellationToken cancellationToken)
        => this.blocksStream.ReadAllHeadersAsync(cancellationToken)
            .SelectGearEvents(this.nodeClient, cancellationToken)
            .SelectServiceEvents();

    protected override ValueTask DisposeCoreAsync() => this.blocksStream.DisposeAsync();
}
