using System;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using DotNet.Testcontainers.Builders;
using DotNet.Testcontainers.Configurations;
using DotNet.Testcontainers.Containers;
using EnsureThat;

namespace Sails.Testing.Containers;

// TODO: Consider introducing ContainerBuilder similar to how it is done
//       for modules shipped along with Testcontainers library.
public sealed class GearNodeContainer : IAsyncDisposable
{
    // TODO: Consider making 'Version' as an optional parameter.
    //       By default the latest version should be taken which can be determined
    //       from the downloaded 'Cargo.toml' file.
    public GearNodeContainer(string consumerName, string gearNodeVersion, bool reuse)
    {
        EnsureArg.IsNotNullOrWhiteSpace(consumerName, nameof(consumerName));
        EnsureArg.IsNotNullOrWhiteSpace(gearNodeVersion, nameof(gearNodeVersion));

        this.nodeInitializationDetector = new NodeInitializationDetector();
        this.container = new ContainerBuilder()
            .WithName($"gear-node-{gearNodeVersion}-for-{consumerName.ToLower()}")
            .WithImage($"ghcr.io/gear-tech/node:v{gearNodeVersion}")
            .WithPortBinding(RpcPort, true)
            .WithEntrypoint("gear")
            .WithCommand(
                "--rpc-external", // --rpc-external is required for listening on all interfaces
                "--dev",
                "--tmp")
            .WithEnvironment("RUST_LOG", "gear=debug,pallet_gear=debug,gwasm=debug")
            .WithReuse(reuse)
            .WithOutputConsumer(this.nodeInitializationDetector)
            .Build();
        this.reuse = reuse;
    }

    private const ushort RpcPort = 9944;
    private static readonly TimeSpan NodeInitializationTimeout = TimeSpan.FromSeconds(30);

    private readonly NodeInitializationDetector nodeInitializationDetector;
    private readonly IContainer container;
    private readonly bool reuse;

    public Uri WsUrl => new($"ws://localhost:{this.container.GetMappedPublicPort(RpcPort)}");

    public ValueTask DisposeAsync()
        // Do not dispose container if it is reused otherwise it will be stopped
        // which we don't want for this particular container. For another one we might
        // choose the opposite behavior though.
        // https://dotnet.testcontainers.org/api/resource_reuse/
        => this.reuse
            ? ValueTask.CompletedTask
            : this.container.DisposeAsync();

    public async Task StartAsync()
    {
        await this.container.StartAsync().ConfigureAwait(false);
        await this.nodeInitializationDetector.IsInitializedAsync(NodeInitializationTimeout).ConfigureAwait(false);
    }

    private sealed class NodeInitializationDetector : IOutputConsumer
    {
        public NodeInitializationDetector()
        {
            this.isNodeInitialized = new TaskCompletionSource();
            this.nodeStdout = new NodeOutput(this.HandleNodeOutput);
            this.nodeStrerr = new NodeOutput(this.HandleNodeOutput);
        }

        private readonly TaskCompletionSource isNodeInitialized;
        private readonly NodeOutput nodeStdout;
        private readonly NodeOutput nodeStrerr;

        public bool Enabled => !this.isNodeInitialized.Task.IsCompleted;
        Stream IOutputConsumer.Stdout => this.nodeStdout;
        Stream IOutputConsumer.Stderr => this.nodeStrerr;

        public async Task IsInitializedAsync(TimeSpan maxWaitTime)
        {
            var timeoutTask = Task.Delay(maxWaitTime);
            var completedTask = await Task.WhenAny(this.isNodeInitialized.Task, timeoutTask).ConfigureAwait(false);
            if (completedTask == timeoutTask)
            {
                this.isNodeInitialized.SetException(
                    new TimeoutException($"Node initialization timed out after {maxWaitTime}."));
                await this.isNodeInitialized.Task.ConfigureAwait(false);
            }
        }

        public void Dispose()
        {
            this.isNodeInitialized.SetCanceled();
            this.nodeStrerr.Dispose();
            this.nodeStdout.Dispose();
            GC.SuppressFinalize(this);
        }

        private void HandleNodeOutput(string output)
        {
            if (this.Enabled && output.Contains("Initialization of block #"))
            {
                this.isNodeInitialized.SetResult();
            }
        }

        private sealed class NodeOutput : Stream
        {
            public NodeOutput(Action<string> output)
            {
                this.output = output;
                this.length = 0;
            }

            private readonly Action<string> output;
            private long length;

            public override bool CanRead => false;
            public override bool CanSeek => false;
            public override bool CanWrite => true;
            public override long Length => this.length;
            public override long Position
            {
                get => this.Length;
                set => throw new NotImplementedException();
            }

            public override void Write(byte[] buffer, int offset, int count)
            {
                var message = Encoding.UTF8.GetString(buffer, offset, count);
                this.output(message);
                this.length += count;
            }

            public override void Flush()
                => throw new NotImplementedException();

            public override int Read(byte[] buffer, int offset, int count)
                => throw new NotImplementedException();

            public override long Seek(long offset, SeekOrigin origin)
                => throw new NotImplementedException();

            public override void SetLength(long value)
                => throw new NotImplementedException();
        }
    }
}
