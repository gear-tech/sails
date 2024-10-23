using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Newtonsoft.Json;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NET.Schnorrkel.Keys;
using Substrate.NetApi;
using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Substrate.Gear.Client;

public static class SubstrateClientExtExtensions
{
    public static async Task<GasInfo> CalculateGasForUploadAsync(
        this SubstrateClientExt nodeClient,
        MiniSecret accountSecret,
        IReadOnlyCollection<byte> wasm,
        IReadOnlyCollection<byte> encodedInitPayload,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(accountSecret, nameof(accountSecret));
        EnsureArg.HasItems(wasm, nameof(wasm));
        EnsureArg.IsNotNull(encodedInitPayload, nameof(encodedInitPayload));

        var accountPublicKeyStr = Utils.Bytes2HexString(accountSecret.GetPair().Public.Key);
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

    public static async Task<GasInfo> CalculateGasGorHandleAsync(
        this SubstrateClientExt nodeClient,
        MiniSecret accountSecret,
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        ValueUnit value,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(accountSecret, nameof(accountSecret));
        EnsureArg.IsNotNull(programId, nameof(programId));
        EnsureArg.IsNotNull(encodedPayload, nameof(encodedPayload));

        var accountPublicKeyStr = Utils.Bytes2HexString(accountSecret.GetPair().Public.Key);
        var encodedPayloadStr = Utils.Bytes2HexString(
            encodedPayload is byte[] encodedPayloadBytes
                ? encodedPayloadBytes
                : [.. encodedPayload]);
        var parameters = new object[] {
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

    private sealed record GasInfoJson
    {
        /// Represents minimum gas limit required for execution.
        [JsonProperty("min_limit")]
        public ulong MinLimit { get; init; }
        /// Gas amount that we reserve for some other on-chain interactions.
        public ulong Reserved { get; init; }
        /// Contains number of gas burned during message processing.
        public ulong Burned { get; init; }
        /// The value may be returned if a program happens to be executed
        /// the second or next time in a block.
        [JsonProperty("may_be_returned")]
        public ulong MayBeReturned { get; init; }
        /// Was the message placed into waitlist at the end of calculating.
        ///
        /// This flag shows, that `min_limit` makes sense and have some guarantees
        /// only before insertion into waitlist.
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
}
