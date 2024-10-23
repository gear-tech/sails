using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions;
using Sails.Remoting.Options;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Extrinsics;
using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Sails.Remoting;

internal sealed class RemotingViaSubstrateClient : IDisposable, IRemoting
{
    public RemotingViaSubstrateClient(RemotingViaSubstrateClientOptions options)
    {
        EnsureArg.IsNotNull(options, nameof(options));
        EnsureArg.IsNotNull(options.GearNodeUri, nameof(options.GearNodeUri));

        this.nodeClient = new SubstrateClientExt(options.GearNodeUri, ChargeTransactionPayment.Default());
        this.isNodeClientConnected = false;
    }

    private readonly SubstrateClientExt nodeClient;
    private bool isNodeClientConnected;

    public void Dispose()
    {
        this.nodeClient.Dispose();
        GC.SuppressFinalize(this);
    }

    public async Task<(ActorId ProgramId, byte[] EncodedReply)> ActivateAsync(
        CodeId codeId,
        IReadOnlyCollection<byte> salt,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(codeId, nameof(codeId));
        EnsureArg.IsNotNull(salt, nameof(salt));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        await this.GetConnectedNodeClientAsync(cancellationToken).ConfigureAwait(false);

        throw new NotImplementedException();
    }

    public Task<byte[]> MessageAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        throw new NotImplementedException();
    }

    public Task<byte[]> QueryAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        throw new NotImplementedException();
    }

    private async Task<SubstrateClientExt> GetConnectedNodeClientAsync(CancellationToken cancellationToken)
    {
        if (!this.isNodeClientConnected)
        {
            await this.nodeClient.ConnectAsync(cancellationToken).ConfigureAwait(false);
            this.isNodeClientConnected = true;
        }
        return this.nodeClient;
    }
}
