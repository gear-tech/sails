﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using Sails.Remoting.Exceptions;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Api.Generated.Storage;
using Substrate.Gear.Client;
using Substrate.Gear.Client.NetApi.Model.Types;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.Remoting.Core;

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

    private const uint DefaultExtrinsicTtlInBlocks = SubstrateClientExtExtensions.DefaultExtrinsicTtlInBlocks; // TODO: Think of making it configurable.

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
            salt.ToBaseVecOfU8(),
            encodedPayload.ToBaseVecOfU8(),
            gasLimit,
            value,
            keep_alive: new Bool(true));

        return await RemotingReplyViaNodeClient<(ActorId, byte[])>.FromExecutionAsync(
                nodeClient,
                executeExtrinsic: nodeClient => nodeClient.ExecuteExtrinsicAsync(
                    this.signingAccount,
                    createProgram,
                    DefaultExtrinsicTtlInBlocks,
                    selectResultOnSuccess: SelectMessageQueuedEventData,
                    selectResultOnError: static (error) =>
                        throw new ExtrinsicDispatchException("Unable to create program", error),
                    cancellationToken),
                extractResult: static (queuedMessageData, replyMessage) =>
                {
                    var payload = replyMessage.Payload.Value.Value.Select(@byte => @byte.Value).ToArray();
                    EnsureSuccessOrThrowReplyException(replyMessage.Details.Value.Code, payload);
                    return ((ActorId)queuedMessageData.Value[2], payload);
                },
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
            encodedPayload.ToBaseVecOfU8(),
            gasLimit,
            value,
            keep_alive: new Bool(true));

        return await RemotingReplyViaNodeClient<byte[]>.FromExecutionAsync(
                nodeClient,
                executeExtrinsic: nodeClient => nodeClient.ExecuteExtrinsicAsync(
                    this.signingAccount,
                    sendMessage,
                    DefaultExtrinsicTtlInBlocks,
                    selectResultOnSuccess: SelectMessageQueuedEventData,
                    selectResultOnError: static (error) =>
                        throw new ExtrinsicDispatchException("Unable to send message", error),
                    cancellationToken),
                extractResult: static (_, replyMessage) =>
                {
                    var payload = replyMessage.Payload.Value.Value.Select(@byte => @byte.Value).ToArray();
                    EnsureSuccessOrThrowReplyException(replyMessage.Details.Value.Code, payload);
                    return payload;
                },
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

        EnsureSuccessOrThrowReplyException(replyInfo.Code, replyInfo.EncodedPayload);

        return replyInfo.EncodedPayload;
    }

    public async Task<EventListener<(ActorId Source, byte[] Payload)>> ListenAsync(CancellationToken cancellationToken)
    {
        var nodeClient = await this.nodeClientProvider.GetNodeClientAsync(cancellationToken).ConfigureAwait(false);
        var blocksStream = await nodeClient.GetNewBlocksStreamAsync(cancellationToken).ConfigureAwait(false);

        return new BlockStreamEventListener(blocksStream);
    }

    private static MessageQueuedEventData SelectMessageQueuedEventData(IEnumerable<BaseEnumRust<RuntimeEvent>> runtimeEvents)
        => runtimeEvents
            .SelectIfMatches(
                RuntimeEvent.Gear,
                (EnumGearEvent gearEvent) => gearEvent)
            .SelectIfMatches(
                GearEvent.MessageQueued,
                (MessageQueuedEventData data) => data)
            .SingleOrDefault()
            ?? throw new ExtrinsicDispatchException("Something terrible happened - MessageQueued event not found");

    private static void EnsureSuccessOrThrowReplyException(EnumReplyCode replyCode, byte[] payload)
    {
        if (replyCode.Value == ReplyCode.Success)
        {
            return;
        }
        var errorString = ParseErrorString(payload);
        ThrowReplyException(replyCode, errorString);
    }

    private static string ParseErrorString(byte[] payload)
    {
        string errorString;
        try
        {
            errorString = Encoding.UTF8.GetString(payload);
        }
        catch
        {
            errorString = "Unexpected reply error";
        }
        return errorString;
    }

    private static void ThrowReplyException(EnumReplyCode replyCode, string message)
    {
        var reason = ErrorReplyReason.Unsupported;
        if (replyCode.Value == ReplyCode.Error)
        {
            var enumReason = (EnumErrorReplyReason)replyCode.Value2;
            reason = enumReason.Value;

            if (reason == ErrorReplyReason.Execution)
            {
                var error = (EnumSimpleExecutionError)enumReason.Value2;
                throw new ExecutionReplyException(message, reason, error);
            }
            if (reason == ErrorReplyReason.FailedToCreateProgram)
            {
                var error = (EnumSimpleProgramCreationError)enumReason.Value2;
                throw new ProgramCreationReplyException(message, reason, error);
            }
        }
        throw new ReplyException(message, reason);
    }
}
