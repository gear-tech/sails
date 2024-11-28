using System.Collections.Generic;
using Sails.Remoting.Tests._Infra.XUnit.Fixtures;
using Substrate.Gear.Client.GearApi.Model.gprimitives;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.Remoting.Tests.Core;

public sealed class RemotingViaNodeClientTests : IAssemblyFixture<SailsFixture>
{
    public RemotingViaNodeClientTests(SailsFixture sailsFixture)
    {
        this.sailsFixture = sailsFixture;
        var serviceCollection = new ServiceCollection();
        serviceCollection.AddRemotingViaNodeClient(
            new NodeClientOptions
            {
                GearNodeUri = this.sailsFixture.GearNodeWsUrl,
            });
        var serviceProvider = serviceCollection.BuildServiceProvider();
        this.remotingProvider = serviceProvider.GetRequiredService<IRemotingProvider>();
        this.remoting = this.remotingProvider.CreateRemoting(SailsFixture.AliceAccount);
    }

    private static readonly Random Random = new((int)DateTime.UtcNow.Ticks);

    private readonly SailsFixture sailsFixture;
    private readonly IRemotingProvider remotingProvider;
    private readonly IRemoting remoting;

    [Fact]
    public void Service_Provider_Resolves_Expected_Implementation()
        => this.remoting.Should().BeOfType<RemotingViaNodeClient>();

    [Fact]
    public async Task Program_Activation_Works()
    {
        // Arrange
        var codeId = await this.sailsFixture.GetDemoContractCodeIdAsync();

        // Act
        var encodedPayload = new Str("Default").Encode();
        var activationReply = await this.remoting.ActivateAsync(
            codeId,
            salt: BitConverter.GetBytes(Random.NextInt64()),
            encodedPayload,
            CancellationToken.None);

        // Assert
        var (programId, payload) = await activationReply.ReadAsync(CancellationToken.None);

        var programIdStr = programId.ToHexString(); // Should be asserted against logs produced by node

        payload.Should().BeEquivalentTo(encodedPayload, options => options.WithStrictOrdering());
    }

    [Fact]
    public async Task Sending_Message_To_Program_Works()
    {
        // Arrange
        var codeId = await this.sailsFixture.GetDemoContractCodeIdAsync();
        var activationReply = await this.remoting.ActivateAsync(
            codeId,
            salt: BitConverter.GetBytes(Random.NextInt64()),
            new Str("Default").Encode(),
            CancellationToken.None);
        var (programId, _) = await activationReply.ReadAsync(CancellationToken.None);

        // Act
        var encodedPayload = new Str("Counter").Encode()
            .Concat(new Str("Add").Encode())
            .Concat(new U32(42).Encode())
            .ToArray();
        var messageReply = await this.remoting.MessageAsync(
            programId,
            encodedPayload,
            CancellationToken.None);

        // Assert
        var messageResult = await messageReply.ReadAsync(CancellationToken.None);

        messageResult.Should().BeEquivalentTo(encodedPayload, options => options.WithStrictOrdering());

        // Some assertion of programId and payload against logs produced by node
    }

    [Fact]
    public async Task Querying_Program_State_Works()
    {
        // Arrange
        var codeId = await this.sailsFixture.GetDemoContractCodeIdAsync();
        var activationReply = await this.remoting.ActivateAsync(
            codeId,
            salt: BitConverter.GetBytes(Random.NextInt64()),
            new Str("Default").Encode(),
            CancellationToken.None);
        var (programId, _) = await activationReply.ReadAsync(CancellationToken.None);
        var messageReply = await this.remoting.MessageAsync(
            programId,
            encodedPayload: new Str("Counter").Encode()
                .Concat(new Str("Add").Encode())
                .Concat(new U32(42).Encode())
                .ToArray(),
            CancellationToken.None);
        await messageReply.ReadAsync(CancellationToken.None);

        // Act
        var encodedPayload = new Str("Counter").Encode()
            .Concat(new Str("Value").Encode())
            .ToArray();
        var queryResult = await this.remoting.QueryAsync(
            programId,
            encodedPayload,
            CancellationToken.None);

        // Assert
        queryResult.Should().BeEquivalentTo(
            [.. encodedPayload, .. new U32(42).Encode()],
            options => options.WithStrictOrdering());
    }

    [Fact]
    public async Task EventListener_Works()
    {
        // Arrange
        var codeId = await this.sailsFixture.GetDemoContractCodeIdAsync();
        var activationReply = await this.remoting.ActivateAsync(
            codeId,
            salt: BitConverter.GetBytes(Random.NextInt64()),
            new Str("Default").Encode(),
            CancellationToken.None);
        var (programId, _) = await activationReply.ReadAsync(CancellationToken.None);

        var encodedPayload = new Str("Counter").Encode()
            .Concat(new Str("Add").Encode())
            .Concat(new U32(42).Encode())
            .ToArray();

        var expectedEventPayload = new List<byte>();
        expectedEventPayload.AddRange(new Str("Counter").Encode());
        expectedEventPayload.AddRange(new Str("Added").Encode());
        expectedEventPayload.AddRange(new U32(42).Encode());

        await using var listener = await this.remoting.ListenAsync(CancellationToken.None);

        // Act
        var messageReply = await this.remoting.MessageAsync(
            programId,
            encodedPayload,
            CancellationToken.None);

        var (source, payload) = await listener.ReadAllAsync(CancellationToken.None).FirstAsync(CancellationToken.None);

        // Assert
        source.Should().BeEquivalentTo(programId);
        payload.Should().BeEquivalentTo(expectedEventPayload, options => options.WithStrictOrdering());
    }
}
