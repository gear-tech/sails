using System;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Microsoft.Extensions.Options;
using Sails.Remoting.Options;
using Substrate.Gear.Api.Generated;
using Substrate.NetApi.Model.Extrinsics;

namespace Sails.Remoting.Core;

internal sealed class NodeClientProvider : IDisposable, INodeClientProvider
{
    public NodeClientProvider(IOptions<NodeClientOptions> options)
    {
        EnsureArg.IsNotNull(options, nameof(options));
        EnsureArg.IsNotNull(options.Value.GearNodeUri, nameof(options.Value.GearNodeUri));

        this.nodeClient = new SubstrateClientExt(options.Value.GearNodeUri, ChargeTransactionPayment.Default());
    }

    private readonly SubstrateClientExt nodeClient;

    /// <inheritdoc/>
    public void Dispose()
    {
        this.nodeClient.Dispose();
        GC.SuppressFinalize(this);
    }

    /// <inheritdoc/>
    public async Task<SubstrateClientExt> GetNodeClientAsync(CancellationToken cancellationToken)
    {
        await this.nodeClient.ConnectAsync(cancellationToken).ConfigureAwait(false);
        return this.nodeClient;
    }
}
