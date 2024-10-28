using System;
using System.Collections.Generic;
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
using Substrate.Gear.Api.Generated.Storage;
using Substrate.Gear.Client.Model.Extrinsics;
using Substrate.Gear.Client.Model.Rpc;
using Substrate.NET.Schnorrkel;
using Substrate.NetApi;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Rpc;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;
using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Substrate.Gear.Client;

public static class SubstrateClientExtExtensions
{
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
    public static async Task<(Hash BlockHash, Hash ExtrinsicHash, uint ExtrinsicIdx)> ExecuteExtrinsicAsync(
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

            var extrinsicId = blockData.Block.GetExtrinsicIdxByHash(extrinsicHash);

            return (blockHash, extrinsicHash, extrinsicId);
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
    /// Lists events that occurred in the specified block.
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

        return await nodeClient.SystemStorage.Events(
                Utils.Bytes2HexString(blockHash),
                cancellationToken)
            .ConfigureAwait(false);
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
        var encodedInitPayloadStr = Utils.Bytes2HexString(
            encodedInitPayload is byte[] encodedInitPayloadBytes
                ? encodedInitPayloadBytes
                : [.. encodedInitPayload]);
        var valueBigInt = value.Value;
        var parameters = new object[]
        {
            accountPublicKeyStr,
            codeId,
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
        var encodedPayloadStr = Utils.Bytes2HexString(
            encodedPayload is byte[] encodedPayloadBytes
                ? encodedPayloadBytes
                : [.. encodedPayload]);
        var parameters = new object[]
        {
            accountPublicKeyStr,
            programId,
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
        var encodedPayloadStr = Utils.Bytes2HexString(
            encodedPayload is byte[] encodedPayloadBytes
                ? encodedPayloadBytes
                : [.. encodedPayload]);
        var parameters = new object[]
        {
            accountPublicKeyStr,
            programId,
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
        public required byte[] EncodedPayload { get; init; }
        // Value sent with the reply.
        public BigInteger Value { get; init; }
        // Reply code of the reply.
        public required JObject Code { get; init; }

        public ReplyInfo ToReplyInfo()
            => new()
            {
                EncodedPayload = this.EncodedPayload,
                Value = (ValueUnit)this.Value,
                // TODO: It is broken. Need to deserialize rust enum.
                Code = new EnumReplyCode()
                {
                    Value = ReplyCode.Success
                }
            };
    }
}
