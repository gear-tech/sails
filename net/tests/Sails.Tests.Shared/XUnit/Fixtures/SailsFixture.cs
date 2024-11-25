using System;
using System.Collections.Generic;
using System.IO;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Nito.AsyncEx;
using Polly;
using Polly.Retry;
using Sails.Tests.Shared.Containers;
using Sails.Tests.Shared.Git;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Client;
using Substrate.NET.Schnorrkel.Keys;
using Substrate.NetApi;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Types;
using Xunit;

namespace Sails.Tests.Shared.XUnit.Fixtures;

public partial class SailsFixture : IAsyncLifetime
{
    public SailsFixture()
        : this(sailsRsVersion: "0.6.3")
    {
    }

    public SailsFixture(string sailsRsVersion)
    {
        EnsureArg.IsNotNullOrWhiteSpace(sailsRsVersion, nameof(sailsRsVersion));

        this.sailsRsReleaseTag = $"rs/v{sailsRsVersion}";
        this.demoContractIdl = new AsyncLazy<string>(
            () => this.DownloadStringAssetAsync("demo.idl"),
            AsyncLazyFlags.RetryOnFailure);
        this.demoContractWasm = new AsyncLazy<MemoryStream>(
            () => this.DownloadOctetAssetAsync("demo.wasm"),
            AsyncLazyFlags.RetryOnFailure);
        this.demoContractCodeId = new AsyncLazy<CodeId>(
            async () =>
            {
                var codeBytes = await this.GetDemoContractWasmAsync().ConfigureAwait(false);
                return await UploadCodeRetryPolicy.ExecuteAsync(
                        () => this.UploadCodeAsync(codeBytes.ToArray()))
                    .ConfigureAwait(false);
            },
            AsyncLazyFlags.RetryOnFailure);
        this.noSvcsProgContractIdl = new AsyncLazy<string>(
            () => this.DownloadStringAssetAsync("no-svcs-prog.idl"),
            AsyncLazyFlags.RetryOnFailure);
        this.noSvcsProgContractWasm = new AsyncLazy<MemoryStream>(
            () => this.DownloadOctetAssetAsync("no_svcs_prog.wasm"),
            AsyncLazyFlags.RetryOnFailure);
        this.noSvcsProgContractCodeId = new AsyncLazy<CodeId>(
            async () =>
            {
                var codeBytes = await this.GetNoSvcsProgContractWasmAsync().ConfigureAwait(false);
                return await UploadCodeRetryPolicy.ExecuteAsync(
                        () => this.UploadCodeAsync(codeBytes.ToArray()))
                    .ConfigureAwait(false);
            },
            AsyncLazyFlags.RetryOnFailure);
        this.gearNodeContainer = null;
    }

    private static readonly GithubDownloader GithubDownloader = new("gear-tech", "sails");
    private static readonly AsyncRetryPolicy UploadCodeRetryPolicy = Policy.Handle<StreamJsonRpc.RemoteInvocationException>(
            exception =>
                exception.ErrorCode == 1014 && exception.Message.StartsWith("Priority is too low"))
        .WaitAndRetryAsync(
            10,
            retry => retry * TimeSpan.FromSeconds(1));

    private readonly string sailsRsReleaseTag;
    private readonly AsyncLazy<string> demoContractIdl;
    private readonly AsyncLazy<MemoryStream> demoContractWasm;
    private readonly AsyncLazy<CodeId> demoContractCodeId;
    private readonly AsyncLazy<string> noSvcsProgContractIdl;
    private readonly AsyncLazy<MemoryStream> noSvcsProgContractWasm;
    private readonly AsyncLazy<CodeId> noSvcsProgContractCodeId;
    private GearNodeContainer? gearNodeContainer;

    public static MiniSecret AliceMiniSecret { get; }
        = new(
            // Taken from 'gear key inspect //Alice' output
            Utils.HexToByteArray("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"),
            ExpandMode.Ed25519);
    public static Account AliceAccount { get; }
        = Account.Build(
            KeyType.Sr25519,
            AliceMiniSecret.ExpandToSecret().ToEd25519Bytes(),
            AliceMiniSecret.GetPair().Public.Key);
    public static MiniSecret BobMiniSecret { get; }
        = new(
            // Taken from 'gear key inspect //Bob' output
            Utils.HexToByteArray("0x398f0c28f98885e046333d4a41c19cee4c37368a9832c6502f6cfd182e2aef89"),
            ExpandMode.Ed25519);
    public static Account BobAccount { get; }
        = Account.Build(
            KeyType.Sr25519,
            BobMiniSecret.ExpandToSecret().ToEd25519Bytes(),
            BobMiniSecret.GetPair().Public.Key);

    public Uri GearNodeWsUrl => this.gearNodeContainer?.WsUrl
        ?? throw new InvalidOperationException("Gear node container is not initialized.");

    public async Task DisposeAsync()
    {
        if (this.gearNodeContainer is not null)
        {
            await this.gearNodeContainer.DisposeAsync().ConfigureAwait(false);
            this.gearNodeContainer = null;
        }
        if (this.demoContractWasm.IsStarted)
        {
            await (await this.demoContractWasm).DisposeAsync().ConfigureAwait(false);
        }
        if (this.noSvcsProgContractWasm.IsStarted)
        {
            await (await this.noSvcsProgContractWasm).DisposeAsync().ConfigureAwait(false);
        }
    }

    public async Task InitializeAsync()
    {
        var sailsRsCargoToml = await this.DownloadSailsRsCargoTomlAsync().ConfigureAwait(false);

        var matchResult = GStdDependencyRegex().Match(sailsRsCargoToml);
        if (!matchResult.Success)
        {
            throw new InvalidOperationException(
                $"Failed to find gstd dependency in Cargo.toml by the '{this.sailsRsReleaseTag}' tag.");
        }
        var gearNodeVersion = matchResult.Groups[1].Value;

        // The `reuse` parameter can be made configurable if needed
        this.gearNodeContainer = new GearNodeContainer(gearNodeVersion, reuse: true);
        await this.gearNodeContainer.StartAsync().ConfigureAwait(false);
    }

    public Task<string> GetDemoContractIdlAsync()
        => this.demoContractIdl.Task;

    public async Task<ReadOnlyMemory<byte>> GetDemoContractWasmAsync()
    {
        var byteStream = await this.demoContractWasm;
        Ensure.Comparable.IsLte(byteStream.Length, int.MaxValue);
        return new ReadOnlyMemory<byte>(byteStream.GetBuffer(), start: 0, length: (int)byteStream.Length);
    }

    public Task<CodeId> GetDemoContractCodeIdAsync()
        => this.demoContractCodeId.Task;

    public Task<string> GetNoSvcsProgContractIdlAsync()
        => this.noSvcsProgContractIdl.Task;

    public async Task<ReadOnlyMemory<byte>> GetNoSvcsProgContractWasmAsync()
    {
        var byteStream = await this.noSvcsProgContractWasm;
        Ensure.Comparable.IsLte(byteStream.Length, int.MaxValue);
        return new ReadOnlyMemory<byte>(byteStream.GetBuffer(), start: 0, length: (int)byteStream.Length);
    }

    public Task<CodeId> GetNoSvcsProgContractCodeIdAsync()
        => this.noSvcsProgContractCodeId.Task;

    private async Task<string> DownloadStringAssetAsync(string assetName)
    {
        var downloadStream = await GithubDownloader.DownloadReleaseAssetAsync(
                this.sailsRsReleaseTag,
                assetName,
                CancellationToken.None)
            .ConfigureAwait(false);
        using (var reader = new StreamReader(downloadStream, leaveOpen: false))
        {
            return await reader.ReadToEndAsync(CancellationToken.None).ConfigureAwait(false);
        }
    }

    private async Task<MemoryStream> DownloadOctetAssetAsync(string assetName)
    {
        var downloadStream = await GithubDownloader.DownloadReleaseAssetAsync(
                this.sailsRsReleaseTag,
                assetName,
                CancellationToken.None)
            .ConfigureAwait(false);
        var memoryStream = new MemoryStream();
        await downloadStream.CopyToAsync(memoryStream).ConfigureAwait(false);
        return memoryStream;
    }

    private async Task<string> DownloadSailsRsCargoTomlAsync()
    {
        var downloadStream = await GithubDownloader.DownloadFileFromTagAsync(
                this.sailsRsReleaseTag,
                "Cargo.toml",
                CancellationToken.None)
            .ConfigureAwait(false);
        using (var reader = new StreamReader(downloadStream, leaveOpen: false))
        {
            return await reader.ReadToEndAsync(CancellationToken.None).ConfigureAwait(false);
        }
    }

    private async Task<CodeId> UploadCodeAsync(IReadOnlyCollection<byte> codeBytes)
    {
        using (var nodeClient = new SubstrateClientExt(
            this.GearNodeWsUrl,
            ChargeTransactionPayment.Default()))
        {
            await nodeClient.ConnectAsync().ConfigureAwait(false);

            return await nodeClient.UploadCodeAsync(
                    AliceAccount,
                    codeBytes,
                    CancellationToken.None)
                .ConfigureAwait(false);
        }
    }

    [GeneratedRegex(@"gstd\s*=\s*""=?(\d+\.\d+\.\d+)""")]
    private static partial Regex GStdDependencyRegex();
}
