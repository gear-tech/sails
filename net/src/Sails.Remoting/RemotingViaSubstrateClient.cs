using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions;
using Sails.Remoting.Options;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.frame_system;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Api.Generated.Storage;
using Substrate.Gear.Client;
using Substrate.Gear.Client.Model.Types;
using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;
using EnumGearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.EnumEvent;
using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
using GearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.Event;
using MessageQueuedGearEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gprimitives.MessageId,
    Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32,
    Substrate.Gear.Api.Generated.Model.gprimitives.ActorId,
    Substrate.Gear.Api.Generated.Model.gear_common.@event.EnumMessageEntry>;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Sails.Remoting;

internal sealed class RemotingViaSubstrateClient : IDisposable, IRemoting
{
    /// <summary>
    /// Creates an instance implementing the <see cref="IRemoting"/> interface via <see cref="SubstrateClientExt"/>
    /// with initial account for signing transactions.
    /// </summary>
    /// <param name="options"></param>
    /// <param name="signingAccount"></param>
    public RemotingViaSubstrateClient(
        RemotingViaSubstrateClientOptions options,
        Account signingAccount)
    {
        EnsureArg.IsNotNull(options, nameof(options));
        EnsureArg.IsNotNull(options.GearNodeUri, nameof(options.GearNodeUri));
        EnsureArg.IsNotNull(signingAccount, nameof(signingAccount));

        this.nodeClient = new SubstrateClientExt(options.GearNodeUri, ChargeTransactionPayment.Default());
        this.isNodeClientConnected = false;
        this.signingAccount = signingAccount;
    }

    private const uint EraLengthInBlocks = 64; // Apparently this is the length of Era in blocks.
    private const uint DefaultExtrinsicTtlInBlocks = EraLengthInBlocks; // TODO: Think of making it configurable.

    private static readonly GasUnit BlockGasLimit = new GearGasConstants().BlockGasLimit();

    private readonly SubstrateClientExt nodeClient;
    private Account signingAccount;
    private bool isNodeClientConnected;

    public void Dispose()
    {
        this.nodeClient.Dispose();
        GC.SuppressFinalize(this);
    }

    /// <inheritdoc/>
    public void SetSigningAccount(Account signingAccount)
    {
        EnsureArg.IsNotNull(signingAccount, nameof(signingAccount));

        this.signingAccount = signingAccount;
    }

    /// <inheritdoc/>
    public async Task<Task<(ActorId ProgramId, byte[] EncodedReply)>> ActivateAsync(
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

        var nodeClient = await this.GetConnectedNodeClientAsync(cancellationToken).ConfigureAwait(false);

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

        var (blockHash, extrinsicHash, extrinsicIdx) = await this.nodeClient.ExecuteExtrinsicAsync(
                this.signingAccount,
                createProgram,
                DefaultExtrinsicTtlInBlocks,
                cancellationToken)
            .ConfigureAwait(false);

        // It can be moved inside the task to return.
        var blockEvents = await this.nodeClient.ListBlockEventsAsync(
                blockHash,
                cancellationToken)
            .ConfigureAwait(false);

        var messageQueuedGearEventData = blockEvents
            .Where(
                blockEvent =>
                    blockEvent.Phase.Matches(
                        Phase.ApplyExtrinsic,
                        (U32 blockExtrinsicIdx) => blockExtrinsicIdx.Value == extrinsicIdx))
            .Select(
                blockEvents =>
                    blockEvents.Event)
            .SelectIfMatches(
                RuntimeEvent.Gear,
                (EnumGearEvent gearEvent) => gearEvent)
            .SelectIfMatches(
                GearEvent.MessageQueued,
                (MessageQueuedGearEventData data) => data)
            .SingleOrDefault()
            ?? throw new Exception("TODO: Custom exception. Something terrible happened.");

        var programId = (ActorId)messageQueuedGearEventData.Value[2];

        static Task<(ActorId ProgramId, byte[] EncodedPayload)> ReceiveReply(CancellationToken cancellationToken)
        {
            cancellationToken.ThrowIfCancellationRequested();
            throw new NotImplementedException();
        }

        return ReceiveReply(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<Task<byte[]>> MessageAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        var nodeClient = await this.GetConnectedNodeClientAsync(cancellationToken).ConfigureAwait(false);

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

        var (blockHash, extrinsicHash, extrinsicIdx) = await this.nodeClient.ExecuteExtrinsicAsync(
                this.signingAccount,
                sendMessage,
                DefaultExtrinsicTtlInBlocks,
                cancellationToken)
            .ConfigureAwait(false);

        static Task<byte[]> ReceiveReply(CancellationToken cancellationToken)
        {
            cancellationToken.ThrowIfCancellationRequested();
            throw new NotImplementedException();
        }

        return ReceiveReply(cancellationToken);
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

        var nodeClient = await this.GetConnectedNodeClientAsync(cancellationToken).ConfigureAwait(false);

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
