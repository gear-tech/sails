using System;
using System.Threading.Tasks;
using DotNet.Testcontainers.Builders;
using DotNet.Testcontainers.Containers;
using EnsureThat;

namespace Sails.Tests.Shared.Containers;

// TODO: Consider introducing ContainerBuilder similar to how it is done
//       for modules shipped along with Testcontainers library.
public sealed class GearNodeContainer : IAsyncDisposable
{
    // TODO: Consider making 'Version' as an optional parameter.
    //       By default the latest version should be taken which can be determined
    //       from the downloaded 'Cargo.toml' file.
    public GearNodeContainer(string gearNodeVersion, bool reuse)
    {
        EnsureArg.IsNotNullOrWhiteSpace(gearNodeVersion, nameof(gearNodeVersion));

        this.container = new ContainerBuilder()
            .WithName("gear-node-for-tests")
            .WithImage($"ghcr.io/gear-tech/node:v{gearNodeVersion}")
            .WithPortBinding(RpcPort, RpcPort) // Use WithPortBinding(RpcPort, true) if random host port is required
            .WithEntrypoint("gear")
            .WithCommand(
                "--rpc-external", // --rpc-external is required for listening on all interfaces
                "--dev",
                "--tmp")
            .WithEnvironment("RUST_LOG", "gear=debug,pallet_gear=debug,gwasm=debug")
            .WithReuse(reuse)
            .Build();
        this.reuse = reuse;
    }

    private const ushort RpcPort = 9944;

    private readonly IContainer container;
    private readonly bool reuse;

    public Uri WsUrl => new($"ws://localhost:{this.container.GetMappedPublicPort(9944)}");

    public ValueTask DisposeAsync()
        // Do not dispose container if it is reused otherwise it will be stopped
        // which we don't want for this particular container. For another one we might
        // choose the opposite behavior though.
        // https://dotnet.testcontainers.org/api/resource_reuse/
        => this.reuse
            ? ValueTask.CompletedTask
            : this.container.DisposeAsync();

    public Task StartAsync()
        => this.container.StartAsync();
}
