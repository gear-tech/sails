using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Storage;
using Substrate.Gear.Client;
using Substrate.Gear.Client.Model.Types;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;
using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Sails.Remoting;

internal sealed class RemotingViaNodeClient : IRemoting
{
    /// <summary>
    /// Creates an instance implementing the <see cref="IRemoting"/> interface via <see cref="SubstrateClientExt"/>
    /// with initial account for signing transactions.
    /// </summary>
    /// <param name="nodeClientProvider"></param>
    /// <param name="signingAccount"></param>
    public RemotingViaNodeClient(
        INodeClientProvider nodeClientProvider,
        Account signingAccount)
    {
        EnsureArg.IsNotNull(nodeClientProvider, nameof(nodeClientProvider));
        EnsureArg.IsNotNull(signingAccount, nameof(signingAccount));

        this.nodeClientProvider = nodeClientProvider;
        this.signingAccount = signingAccount;
    }

    private const uint EraLengthInBlocks = 64; // Apparently this is the length of Era in blocks.
    private const uint DefaultExtrinsicTtlInBlocks = EraLengthInBlocks; // TODO: Think of making it configurable.

    private static readonly GasUnit BlockGasLimit = new GearGasConstants().BlockGasLimit();

    private readonly INodeClientProvider nodeClientProvider;
    private readonly Account signingAccount;

    /// <inheritdoc/>
    public async Task<RemotingReply<(ActorId ProgramId, byte[] Payload)>> ActivateAsync(
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

        var nodeClient = await this.nodeClientProvider.GetNodeClientAsync(cancellationToken).ConfigureAwait(false);

        gasLimit ??= (await nodeClient.CalculateGasForCreateProgramAsync(
                    this.signingAccount.GetPublicKey(),
                    codeId,
                    encodedPayload,
                    value,
                    cancellationToken)
                .ConfigureAwait(false))
            .MinLimit;

        var createProgram = GearCalls.CreateProgram(
            codeId,
            new BaseVec<U8>(salt.Select(@byte => new U8(@byte)).ToArray()),
            new BaseVec<U8>(encodedPayload.Select(@byte => new U8(@byte)).ToArray()),
            gasLimit,
            value,
            keep_alive: new Bool(true));

        return await RemotingReplyViaNodeClient<(ActorId, byte[])>.FromExecutionAsync(
                nodeClient,
                nodeClient => nodeClient.ExecuteExtrinsicAsync(
                    this.signingAccount,
                    createProgram,
                    DefaultExtrinsicTtlInBlocks,
                    cancellationToken),
                (queuedMessageData, replyMessage) => (
                    (ActorId)queuedMessageData.Value[2],
                    replyMessage.Payload.Value.Value
                        .Select(@byte => @byte.Value)
                        .ToArray()),
                cancellationToken)
            .ConfigureAwait(false);
    }

    /// <inheritdoc/>
    public async Task<RemotingReply<byte[]>> MessageAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        var nodeClient = await this.nodeClientProvider.GetNodeClientAsync(cancellationToken).ConfigureAwait(false);

        gasLimit ??= (await nodeClient.CalculateGasForHandleAsync(
                    this.signingAccount.GetPublicKey(),
                    programId,
                    encodedPayload,
                    value,
                    cancellationToken)
                .ConfigureAwait(false))
            .MinLimit;

        var sendMessage = GearCalls.SendMessage(
            programId,
            new BaseVec<U8>(encodedPayload.Select(@byte => new U8(@byte)).ToArray()),
            gasLimit,
            value,
            keep_alive: new Bool(true));

        return await RemotingReplyViaNodeClient<byte[]>.FromExecutionAsync(
                nodeClient,
                nodeClient => nodeClient.ExecuteExtrinsicAsync(
                    this.signingAccount,
                    sendMessage,
                    DefaultExtrinsicTtlInBlocks,
                    cancellationToken),
                (_, replyMessage) => replyMessage.Payload.Value.Value
                    .Select(@byte => @byte.Value)
                    .ToArray(),
                cancellationToken)
            .ConfigureAwait(false);
    }

    /// <inheritdoc/>
    public async Task<byte[]> QueryAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        var nodeClient = await this.nodeClientProvider.GetNodeClientAsync(cancellationToken).ConfigureAwait(false);

        gasLimit ??= BlockGasLimit;

        var replyInfo = await nodeClient.CalculateReplyForHandleAsync(
                this.signingAccount.GetPublicKey(),
                programId,
                encodedPayload,
                gasLimit,
                value,
                cancellationToken)
            .ConfigureAwait(false);

        // TODO: Check for reply code

        return replyInfo.EncodedPayload;
    }
}
