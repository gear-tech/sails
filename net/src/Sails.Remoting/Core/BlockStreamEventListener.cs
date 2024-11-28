using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Client;

namespace Sails.Remoting.Core;

internal class BlockStreamEventListener : EventListener<(ActorId Source, byte[] Bytes)>
{
    private readonly SubstrateClientExt nodeClient;
    private readonly BlocksStream blocksStream;

    internal BlockStreamEventListener(SubstrateClientExt nodeClient, BlocksStream blocksStream)
    {
        this.nodeClient = nodeClient;
        this.blocksStream = blocksStream;
    }

    public override IAsyncEnumerator<(ActorId Source, byte[] Bytes)> GetAsyncEnumerator(
        CancellationToken cancellationToken = default)
        => this.blocksStream.ReadAllHeadersAsync(cancellationToken)
            .SelectGearEvents(this.nodeClient, cancellationToken)
            .SelectServiceEvents()
            .GetAsyncEnumerator(cancellationToken);

    protected override ValueTask DisposeCoreAsync() => this.blocksStream.DisposeAsync();
}
