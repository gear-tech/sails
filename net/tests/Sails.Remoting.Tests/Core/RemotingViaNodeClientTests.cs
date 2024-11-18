using System.Threading;
using System.Threading.Tasks;
using Sails.Remoting.Tests._Infra.XUnit.Fixtures;
using Sails.Tests.Shared.XUnit;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Storage;
using Substrate.Gear.Client;
using Substrate.Gear.Client.Model.Types.Base;
using Substrate.Gear.Client.Model.Types.Primitive;
using Substrate.NET.Schnorrkel.Keys;
using Substrate.NetApi;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;
using Xunit;

namespace Sails.Remoting.Tests.Core;

public sealed class RemotingViaNodeClientTests : IAssemblyFixture<SailsFixture>
{
    public RemotingViaNodeClientTests(SailsFixture sailsFixture)
    {
        this.sailsFixture = sailsFixture;
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

    private readonly SailsFixture sailsFixture;

    [Fact]
    public async Task Test()
    {
        var nodeWsUrl = this.sailsFixture.GearNodeWsUrl;

        using (var nodeClient = new SubstrateClientExt(nodeWsUrl, ChargeTransactionPayment.Default()))
        {
            await nodeClient.ConnectAsync();

            var codeBytes = await this.sailsFixture.GetNoSvcsProgContractWasmAsync();

            var uploadCode = GearCalls.UploadCode(codeBytes.ToBaseVecOfU8());
            var extrinsicResult = await nodeClient.ExecuteExtrinsicAsync(
                AliceAccount,
                uploadCode,
                64,
                CancellationToken.None);
        }
    }
}
