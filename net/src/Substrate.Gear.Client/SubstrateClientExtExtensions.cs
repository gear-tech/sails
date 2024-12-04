﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.frame_system;
using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Model.sp_runtime;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Api.Generated.Storage;
using Substrate.Gear.Client.GearApi.Model.gprimitives;
using Substrate.Gear.Client.NetApi.Model.Extrinsics;
using Substrate.Gear.Client.NetApi.Model.Rpc;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NET.Schnorrkel;
using Substrate.NetApi;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Rpc;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;

namespace Substrate.Gear.Client;

public static class SubstrateClientExtExtensions
{
    public const uint DefaultExtrinsicTtlInBlocks = (uint)Constants.ExtrinsicEraPeriodDefault;

    /// <summary>
    /// Executes specified extrinsic on blockchain. In case of success, returns hashes of a block
    /// in which the extrinsic was executed and its hash.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="signingAccount"></param>
    /// <param name="method">Extrinsic to execute</param>
    /// <param name="lifeTimeInBlocks">
    ///     Number of blocks the extrinsic is valid for execution.
    ///     0 means infinity (immortal)
    /// </param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    /// <exception cref="TimeoutException"></exception>
    public static async Task<ExtrinsicInfo> ExecuteExtrinsicAsync(
        this SubstrateClient nodeClient,
        Account signingAccount,
        Method method,
        uint lifeTimeInBlocks,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(signingAccount, nameof(signingAccount));
        EnsureArg.IsNotNull(method, nameof(method));

        var extrinsic = await nodeClient.GetExtrinsicParametersAsync(
                method,
                signingAccount,
                ChargeTransactionPayment.Default(),
                lifeTimeInBlocks,
                signed: true,
                cancellationToken)
            .ConfigureAwait(false);

        var (extrinsicBytes, extrinsicHash) = extrinsic.EncodeAndHash();
        var taskCompletionSource = new TaskCompletionSource<Hash>();
        var subscriptionId = default(string);
        try
        {
            subscriptionId = await nodeClient.Author.SubmitAndWatchExtrinsicAsync(
                    (string _, ExtrinsicStatus extrinsicStatus) =>
                    {
                        switch (extrinsicStatus.ExtrinsicState)
                        {
                            case ExtrinsicState.Ready:
                            case ExtrinsicState.InBlock:
                                break;
                            case ExtrinsicState.Finalized:
                                taskCompletionSource.SetResult(extrinsicStatus.Hash);
                                break;
                            case ExtrinsicState.Future:
                            case ExtrinsicState.Broadcast:
                            case ExtrinsicState.Retracted:
                            case ExtrinsicState.FinalityTimeout:
                            case ExtrinsicState.Usurped:
                            case ExtrinsicState.Dropped:
                            case ExtrinsicState.Invalid:
                            default:
                                taskCompletionSource.SetException(new Exception("TODO: Custom exception."));
                                break;
                        }
                    },
                    Utils.Bytes2HexString(extrinsicBytes),
                    cancellationToken)
                .ConfigureAwait(false);

            var cancellationTask = lifeTimeInBlocks > 0
                ? Task.Delay(TimeSpan.FromTicks(ExpectedBlockTime.Ticks * lifeTimeInBlocks), cancellationToken)
                : Task.Delay(Timeout.Infinite, cancellationToken);

            var completedTask = await Task.WhenAny(taskCompletionSource.Task, cancellationTask)
                .ConfigureAwait(false);
            if (completedTask != taskCompletionSource.Task)
            {
                throw new TimeoutException("TODO: Custom exception.");
            }
            var blockHash = await taskCompletionSource.Task.ConfigureAwait(false);

            var blockData = await nodeClient.Chain.GetBlockAsync(blockHash, cancellationToken)
                .ConfigureAwait(false);

            var extrinsicIdx = blockData.Block.GetExtrinsicIdxByHash(extrinsicHash);

            return new ExtrinsicInfo
            {
                BlockHash = blockHash,
                IndexInBlock = extrinsicIdx,
                Hash = extrinsicHash
            };
        }
        finally
        {
            if (subscriptionId is not null)
            {
                await nodeClient.Author.UnwatchExtrinsicAsync(subscriptionId).ConfigureAwait(false);
            }
        }
    }

    /// <summary>
    /// Executes specified extrinsic on blockchain and calls provided callbacks for
    /// extracting result from events generated by the execution.
    /// </summary>
    /// <typeparam name="TResult"></typeparam>
    /// <param name="nodeClient"></param>
    /// <param name="signingAccount"></param>
    /// <param name="method"></param>
    /// <param name="lifeTimeInBlocks"></param>
    /// <param name="selectResultOnSuccess"></param>
    /// <param name="selectResultOnError">
    /// If result can't be returned based on error information, then exception should be thrown.
    /// </param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<TResult> ExecuteExtrinsicAsync<TResult>(
        this SubstrateClientExt nodeClient,
        Account signingAccount,
        Method method,
        uint lifeTimeInBlocks,
        Func<IEnumerable<BaseEnumRust<RuntimeEvent>>, TResult> selectResultOnSuccess,
        Func<ExtrinsicFailedEventData, TResult> selectResultOnError,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(selectResultOnSuccess, nameof(selectResultOnSuccess));
        EnsureArg.IsNotNull(selectResultOnError, nameof(selectResultOnError));

        var extrinsicInfo = await nodeClient.ExecuteExtrinsicAsync(
                signingAccount,
                method,
                lifeTimeInBlocks,
                cancellationToken)
            .ConfigureAwait(false);

        var extrinsicBlockEvents = await nodeClient.ListBlockEventsAsync(
                extrinsicInfo.BlockHash,
                cancellationToken)
            .ConfigureAwait(false);

        var extrinsicRuntimeEvents = extrinsicBlockEvents
            .Where(
                eventRecord =>
                    eventRecord.Phase.Matches(
                        Phase.ApplyExtrinsic,
                        (U32 extrinsicIdxInBlock) => extrinsicIdxInBlock.Value == extrinsicInfo.IndexInBlock))
            .Select(
                eventRecord => eventRecord.Event);

        var extrinsicDispatchError = extrinsicRuntimeEvents
            .SelectIfMatches(
                RuntimeEvent.System,
                (EnumSystemEvent systemEvent) => systemEvent)
            .SelectIfMatches(
                SystemEvent.ExtrinsicFailed,
                (ExtrinsicFailedEventData data) => data)
            .SingleOrDefault();

        return extrinsicDispatchError is null
            ? selectResultOnSuccess(extrinsicRuntimeEvents)
            : selectResultOnError(extrinsicDispatchError);
    }

    /// <summary>
    /// Lists events occurred in the specified block.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="blockHash"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<EventRecord[]> ListBlockEventsAsync(
        this SubstrateClientExt nodeClient,
        Hash blockHash,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(blockHash, nameof(blockHash));

        return (await nodeClient.SystemStorage.Events(
                Utils.Bytes2HexString(blockHash),
                cancellationToken)
            .ConfigureAwait(false))?.Value // 0-th block doesn't have any events
            ?? [];
    }

    /// <summary>
    /// Lists events occurred in the specified block.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="blockNumber">Block number which should be less or equal <see cref="uint.MaxValue"/>.</param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<EventRecord[]> ListBlockEventsAsync(
        this SubstrateClientExt nodeClient,
        U64 blockNumber,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(blockNumber, nameof(blockNumber));
        EnsureArg.IsLte(blockNumber.Value, uint.MaxValue, nameof(blockNumber));

        // TODO: Needs own implementation of GetBlockHashAsync accepting U64
        var blockHash = await nodeClient.Chain.GetBlockHashAsync(
                new BlockNumber((uint)blockNumber),
                cancellationToken)
            .ConfigureAwait(false);
        return await nodeClient.ListBlockEventsAsync(
                blockHash,
                cancellationToken)
            .ConfigureAwait(false);
    }

    /// <summary>
    /// Subscribes to all blocks and returns them as a stream which can be read as an async enumerable.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static Task<BlocksStream> GetAllBlocksStreamAsync(
        this SubstrateClientExt nodeClient,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));

        return BlocksStream.CreateAsync(
            nodeClient,
            (nodeClient, callback) =>
                nodeClient.Chain.SubscribeAllHeadsAsync(callback, cancellationToken),
            (nodeClient, subscriptionId) =>
                nodeClient.Chain.UnsubscribeAllHeadsAsync(subscriptionId, CancellationToken.None));
    }

    /// <summary>
    /// Subscribes to the best blocks and returns them as a stream which can be read as an async enumerable.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static Task<BlocksStream> GetNewBlocksStreamAsync(
        this SubstrateClientExt nodeClient,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));

        return BlocksStream.CreateAsync(
            nodeClient,
            (nodeClient, callback) =>
                nodeClient.Chain.SubscribeNewHeadsAsync(callback, cancellationToken),
            (nodeClient, subscriptionId) =>
                nodeClient.Chain.UnsubscribeNewHeadsAsync(subscriptionId, CancellationToken.None));
    }

    /// <summary>
    /// Subscribes to the best finalized blocks and returns them as a stream which can be read as an async enumerable.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static Task<BlocksStream> GetFinalizedBlocksStreamAsync(
        this SubstrateClientExt nodeClient,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));

        // TODO: It is noteworthy that some blocks may be skipped in the stream assuming they were finalized without sending a
        //       notification, i.e., if you observe block X and then X + 2, it means that block X + 1 was finalized too.
        //       Probably it should be accounted here and missed blocks should be fetched from the chain.
        return BlocksStream.CreateAsync(
            nodeClient,
            (nodeClient, callback) =>
                nodeClient.Chain.SubscribeFinalizedHeadsAsync(callback, cancellationToken),
            (nodeClient, subscriptionId) =>
                nodeClient.Chain.UnsubscribeFinalizedHeadsAsync(subscriptionId, CancellationToken.None));
    }

    /// <summary>
    /// Calculates amount of gas required for creating a new program from previously uploaded code.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="signingAccountKey"></param>
    /// <param name="codeId"></param>
    /// <param name="encodedInitPayload"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<GasInfo> CalculateGasForCreateProgramAsync(
        this SubstrateClientExt nodeClient,
        PublicKey signingAccountKey,
        CodeId codeId,
        IReadOnlyCollection<byte> encodedInitPayload,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(signingAccountKey, nameof(signingAccountKey));
        EnsureArg.IsNotNull(codeId, nameof(codeId));
        EnsureArg.IsNotNull(encodedInitPayload, nameof(encodedInitPayload));

        var accountPublicKeyStr = Utils.Bytes2HexString(signingAccountKey.Key);
        var codeIdStr = codeId.ToHexString();
        var encodedInitPayloadStr = Utils.Bytes2HexString(
            encodedInitPayload is byte[] encodedInitPayloadBytes
                ? encodedInitPayloadBytes
                : [.. encodedInitPayload]);
        var valueBigInt = value.Value;
        var parameters = new object[]
        {
            accountPublicKeyStr,
            codeIdStr,
            encodedInitPayloadStr,
            valueBigInt,
            true
        };

        var gasInfoJson = await nodeClient.InvokeAsync<GasInfoJson>(
                "gear_calculateInitCreateGas",
                parameters,
                cancellationToken)
            .ConfigureAwait(false);

        return gasInfoJson.ToGasInfo();
    }

    /// <summary>
    /// Calculates amount of gas required for uploading code and creating a new program from it.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="signingAccountKey"></param>
    /// <param name="wasm"></param>
    /// <param name="encodedInitPayload"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<GasInfo> CalculateGasForUploadProgramAsync(
        this SubstrateClientExt nodeClient,
        PublicKey signingAccountKey,
        IReadOnlyCollection<byte> wasm,
        IReadOnlyCollection<byte> encodedInitPayload,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(signingAccountKey, nameof(signingAccountKey));
        EnsureArg.HasItems(wasm, nameof(wasm));
        EnsureArg.IsNotNull(encodedInitPayload, nameof(encodedInitPayload));

        var accountPublicKeyStr = Utils.Bytes2HexString(signingAccountKey.Key);
        var wasmBytesStr = Utils.Bytes2HexString(
            wasm is byte[] wasmBytes
                ? wasmBytes
                : [.. wasm]);
        var encodedInitPayloadStr = Utils.Bytes2HexString(
            encodedInitPayload is byte[] encodedInitPayloadBytes
                ? encodedInitPayloadBytes
                : [.. encodedInitPayload]);
        var valueBigInt = value.Value;
        var parameters = new object[]
        {
            accountPublicKeyStr,
            wasmBytesStr,
            encodedInitPayloadStr,
            valueBigInt,
            true
        };

        var gasInfoJson = await nodeClient.InvokeAsync<GasInfoJson>(
                "gear_calculateGasForUpload",
                parameters,
                cancellationToken)
            .ConfigureAwait(false);

        return gasInfoJson.ToGasInfo();
    }

    /// <summary>
    /// Calculates amount of gas required for executing a message by specified program.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="signingAccountKey"></param>
    /// <param name="programId"></param>
    /// <param name="encodedPayload"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<GasInfo> CalculateGasForHandleAsync(
        this SubstrateClientExt nodeClient,
        PublicKey signingAccountKey,
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(signingAccountKey, nameof(signingAccountKey));
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        var accountPublicKeyStr = Utils.Bytes2HexString(signingAccountKey.Key);
        var programIdStr = programId.ToHexString();
        var encodedPayloadStr = Utils.Bytes2HexString(
            encodedPayload is byte[] encodedPayloadBytes
                ? encodedPayloadBytes
                : [.. encodedPayload]);
        var parameters = new object[]
        {
            accountPublicKeyStr,
            programIdStr,
            encodedPayloadStr,
            value.Value,
            true
        };

        var gasInfoJson = await nodeClient.InvokeAsync<GasInfoJson>(
                "gear_calculateGasForHandle",
                parameters,
                cancellationToken)
            .ConfigureAwait(false);

        return gasInfoJson.ToGasInfo();
    }

    /// <summary>
    /// Calculates reply for a message which program would return
    /// without actual applying to blockchain.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="signingAccountKey"></param>
    /// <param name="programId"></param>
    /// <param name="encodedPayload"></param>
    /// <param name="gasLimit"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<ReplyInfo> CalculateReplyForHandleAsync(
        this SubstrateClientExt nodeClient,
        PublicKey signingAccountKey,
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(signingAccountKey, nameof(signingAccountKey));
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        var accountPublicKeyStr = Utils.Bytes2HexString(signingAccountKey.Key);
        var programIdStr = programId.ToHexString();
        var encodedPayloadStr = Utils.Bytes2HexString(
            encodedPayload is byte[] encodedPayloadBytes
                ? encodedPayloadBytes
                : [.. encodedPayload]);
        var parameters = new object[]
        {
            accountPublicKeyStr,
            programIdStr,
            encodedPayloadStr,
            gasLimit.Value,
            value.Value,
        }; // Do we need the `at` passed as None?

        var replyInfoJson = await nodeClient.InvokeAsync<ReplyInfoJson>(
                "gear_calculateReplyForHandle",
                parameters,
                cancellationToken)
            .ConfigureAwait(false);

        return replyInfoJson.ToReplyInfo();
    }

    /// <summary>
    /// Uploads specified code to the blockchain and returns its identifier
    /// regardless whether the code was uploaded or already exists.
    /// </summary>
    /// <param name="nodeClient"></param>
    /// <param name="signingAccount"></param>
    /// <param name="wasm"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static Task<CodeId> UploadCodeAsync(
        this SubstrateClientExt nodeClient,
        Account signingAccount,
        IReadOnlyCollection<byte> wasm,
        CancellationToken cancellationToken)
    {
        var uploadCode = GearCalls.UploadCode(wasm.ToBaseVecOfU8());

        return nodeClient.ExecuteExtrinsicAsync(
            signingAccount,
            uploadCode,
            DefaultExtrinsicTtlInBlocks,
            selectResultOnSuccess: (runtimeEvents) =>
                (CodeId)runtimeEvents
                    .SelectIfMatches(
                        RuntimeEvent.Gear,
                        (EnumGearEvent gearEvent) => gearEvent)
                    .SelectIfMatches(
                        GearEvent.CodeChanged,
                        (CodeChangedEventData data) => data)
                    .Single()
                    .Value[0],
            selectResultOnError: (extrinsicFailedEventData) =>
                {
                    var dispatchError = (EnumDispatchError)extrinsicFailedEventData.Value[0];
                    // TODO: Do proper error parsing using node metadata.
                    return dispatchError.Matches(
                        DispatchError.Module,
                        (ModuleError moduleError) =>
                            moduleError.Index == 104 // Gear
                            && moduleError.Error.Value[0] == 6) // CodeAlreadyExists
                        ? GearApi.Model.gprimitives.CodeId.FromCodeBytes(wasm)
                        : throw new Exception("TODO: Custom exception.");
                },
            cancellationToken);
    }

    private static readonly TimeSpan ExpectedBlockTime = TimeSpan.FromMilliseconds(new BabeConstants().ExpectedBlockTime());

    private sealed record GasInfoJson
    {
        // Represents minimum gas limit required for execution.
        [JsonProperty("min_limit")]
        public ulong MinLimit { get; init; }
        // Gas amount that we reserve for some other on-chain interactions.
        public ulong Reserved { get; init; }
        // Contains number of gas burned during message processing.
        public ulong Burned { get; init; }
        // The value may be returned if a program happens to be executed
        // the second or next time in a block.
        [JsonProperty("may_be_returned")]
        public ulong MayBeReturned { get; init; }
        // Was the message placed into waitlist at the end of calculating.
        // This flag shows, that `min_limit` makes sense and have some guarantees
        // only before insertion into waitlist.
        [JsonProperty("waited")]
        public bool IsInWaitList { get; init; }

        public GasInfo ToGasInfo()
            => new()
            {
                MinLimit = (GasUnit)this.MinLimit,
                Reserved = (GasUnit)this.Reserved,
                Burned = (GasUnit)this.Burned,
                MayBeReturned = (GasUnit)this.MayBeReturned,
                IsInWaitList = this.IsInWaitList
            };
    }

    private sealed record ReplyInfoJson
    {
        // Payload of the reply.
        [JsonProperty("payload")]
        public required string EncodedPayload { get; init; }
        // Value sent with the reply.
        public BigInteger Value { get; init; }
        // Reply code of the reply.
        public required JObject Code { get; init; }

        public ReplyInfo ToReplyInfo()
            => new()
            {
                EncodedPayload = Utils.HexToByteArray(this.EncodedPayload),
                Value = (ValueUnit)this.Value,
                Code = this.Code.DeserializeEnumReplyCode(),
            };
    }

    /// <summary>
    /// Convert JToken (JObject) wtih single property to EnumReplyCode
    /// </summary>
    /// <param name="token">JObject with single property</param>
    /// <returns></returns>
    /// <exception cref="InvalidOperationException"></exception>
    /// <exception cref="NotImplementedException"></exception>
    internal static EnumReplyCode DeserializeEnumReplyCode(this JToken? token)
    {
        if (token?.First is not JProperty prop || !Enum.TryParse<ReplyCode>(prop.Name, out var replyCode))
        {
            throw new InvalidOperationException("Failed to convert JToken to EnumReplyCode");
        }
        IType value = replyCode switch
        {
            ReplyCode.Success => DeserializeBaseEnum<EnumSuccessReplyReason, SuccessReplyReason>(prop.Value),
            ReplyCode.Error => DeserializeEnumErrorReplyReason(prop.Value),
            ReplyCode.Unsupported => new BaseVoid(),
            _ => throw new NotImplementedException(),
        };
        var enumValue = new EnumReplyCode();
        enumValue.Create(replyCode, value);
        return enumValue;
    }

    /// <summary>
    /// Convert JToken (JObject) wtih single property to EnumErrorReplyReason
    /// </summary>
    /// <param name="token">JObject with single property</param>
    /// <returns></returns>
    /// <exception cref="InvalidOperationException"></exception>
    /// <exception cref="NotImplementedException"></exception>
    internal static EnumErrorReplyReason DeserializeEnumErrorReplyReason(this JToken? token)
    {
        if (token?.First is not JProperty prop || !Enum.TryParse<ErrorReplyReason>(prop.Name, out var replyReason))
        {
            throw new InvalidOperationException("Failed to convert JToken to EnumErrorReplyReason");
        }
        IType value = replyReason switch
        {
            ErrorReplyReason.Execution
                => DeserializeBaseEnum<EnumSimpleExecutionError, SimpleExecutionError>(prop.Value),
            ErrorReplyReason.FailedToCreateProgram
                => DeserializeBaseEnum<EnumSimpleProgramCreationError, SimpleProgramCreationError>(prop.Value),
            ErrorReplyReason.InactiveActor => new BaseVoid(),
            ErrorReplyReason.RemovedFromWaitlist => new BaseVoid(),
            ErrorReplyReason.ReinstrumentationFailure => new BaseVoid(),
            ErrorReplyReason.Unsupported => new BaseVoid(),
            _ => throw new NotImplementedException(),
        };
        var enumValue = new EnumErrorReplyReason();
        enumValue.Create(replyReason, value);
        return enumValue;
    }

    internal static T DeserializeBaseEnum<T, TEnum>(this JToken? token)
        where T : BaseEnum<TEnum>, new()
        where TEnum : struct, Enum
    {
        if (token is not JValue val || !Enum.TryParse<TEnum>(val.ToString(), out var enumValue))
        {
            throw new InvalidOperationException($"Failed to convert JToken to {typeof(T).FullName}");
        }
        var value = new T();
        value.Create(enumValue);
        return value;
    }
}
