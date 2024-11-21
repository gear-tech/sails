using Sails.Remoting.Tests._Infra.XUnit.Fixtures;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Client;
using Substrate.Gear.Client.Extensions;
using Substrate.Gear.Client.GearApi.Model.gprimitives;
using Substrate.NET.Schnorrkel.Keys;
using Substrate.NetApi;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Primitive;
using CodeId = Substrate.Gear.Api.Generated.Model.gprimitives.CodeId;

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
        this.remoting = this.remotingProvider.CreateRemoting(AliceAccount);
    }

    private static readonly MiniSecret AliceMiniSecret
        = new(
            Utils.HexToByteArray("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"),
            ExpandMode.Ed25519);
    private static readonly Account AliceAccount
        = Account.Build(
            KeyType.Sr25519,
            AliceMiniSecret.ExpandToSecret().ToEd25519Bytes(),
            AliceMiniSecret.GetPair().Public.Key);
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
        var codeBytes = await this.sailsFixture.GetNoSvcsProgContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());

        // Act
        var encodedPayload = new Str("Default").Encode();
        var activationReply = await this.remoting.ActivateAsync(
            codeId,
            salt: BitConverter.GetBytes(Random.NextInt64()),
            encodedPayload,
            CancellationToken.None);

        // Assert
        var activationResult = await activationReply.ReadAsync(CancellationToken.None);

        var programIdStr = activationResult.ProgramId.ToHexString(); // Should be asserted against logs produced by node

        activationResult.Payload.Should().BeEquivalentTo(encodedPayload, options => options.WithStrictOrdering());
    }

    [Fact]
    public async Task Sending_Message_To_Program_Works()
    {
        // Arrange
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());
        var activationReply = await this.remoting.ActivateAsync(
            codeId,
            salt: BitConverter.GetBytes(Random.NextInt64()),
            new Str("Default").Encode(),
            CancellationToken.None);
        var activationResult = await activationReply.ReadAsync(CancellationToken.None);

        // Act
        var encodedPayload = new Str("Counter").Encode()
            .Concat(new Str("Add").Encode())
            .Concat(new U32(42).Encode())
            .ToArray();
        var messageReply = await this.remoting.MessageAsync(
            activationResult.ProgramId,
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
        var codeBytes = await this.sailsFixture.GetDemoContractWasmAsync();
        var codeId = await this.UploadCodeAsync(codeBytes.AsReadOnlyCollection());
        var activationReply = await this.remoting.ActivateAsync(
            codeId,
            salt: BitConverter.GetBytes(Random.NextInt64()),
            new Str("Default").Encode(),
            CancellationToken.None);
        var activationResult = await activationReply.ReadAsync(CancellationToken.None);
        var messageReply = await this.remoting.MessageAsync(
            activationResult.ProgramId,
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
            activationResult.ProgramId,
            encodedPayload,
            CancellationToken.None);

        // Assert
        queryResult.Should().BeEquivalentTo(
            encodedPayload.Concat(new U32(42).Encode()).ToArray(),
            options => options.WithStrictOrdering());
    }

    private async Task<CodeId> UploadCodeAsync(IReadOnlyCollection<byte> codeBytes)
    {
        using (var nodeClient = new SubstrateClientExt(
            this.sailsFixture.GearNodeWsUrl,
            ChargeTransactionPayment.Default()))
        {
            await nodeClient.ConnectAsync();

            return await nodeClient.UploadCodeAsync(
                AliceAccount,
                codeBytes,
                CancellationToken.None);
        }
    }
}
