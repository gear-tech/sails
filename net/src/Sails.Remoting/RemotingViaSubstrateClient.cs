using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Nito.AsyncEx;
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

        this.nodeClient = new AsyncLazy<SubstrateClientExt>(
            async () =>
            {
                this.nodeClientToDispose ??= new SubstrateClientExt(options.GearNodeUri, ChargeTransactionPayment.Default());
                await this.nodeClientToDispose.ConnectAsync().ConfigureAwait(false);
                return this.nodeClientToDispose;
            },
            AsyncLazyFlags.RetryOnFailure);
    }

    private readonly AsyncLazy<SubstrateClientExt> nodeClient;
    private SubstrateClientExt? nodeClientToDispose;

    public void Dispose()
    {
        if (this.nodeClientToDispose is not null)
        {
            this.nodeClientToDispose.Dispose();
            this.nodeClientToDispose = null;
        }
    }

    public Task<(ActorId ProgramId, byte[] EncodedReply)> ActivateAsync(
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
}
